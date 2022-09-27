use async_trait::async_trait;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use chrono::{DateTime, TimeZone, Utc};
use cprices::data::{DBSaver, KLine};
use deadpool_postgres::{tokio_postgres::NoTls, Pool};
use reqwest::Url;
use std::{error::Error, path::Path, time::Duration};

#[derive(serde::Deserialize)]
pub struct DbConfig {
    pub pg: deadpool_postgres::Config,
}
impl DbConfig {
    pub fn from_url(db_url: &str) -> Result<Self, Box<dyn Error>> {
        let mut pg = deadpool_postgres::Config::new();
        let parsed = Url::parse(db_url).map_err(|e| format!("db-url parse: {}", e))?;
        if parsed.scheme() != "postgres" {
            Err(format!("wrong postgres url scheme '{}'", parsed.scheme()))?
        }
        pg.dbname = Path::new(parsed.path())
            .strip_prefix("/")?
            .to_str()
            .map(String::from);
        pg.host = parsed.host_str().map(String::from);
        pg.user = Some(parsed.username().to_string());
        pg.password = parsed.password().map(String::from);
        pg.port = parsed.port();
        Ok(DbConfig { pg })
    }
}

#[derive()]
pub struct PostgresClient {
    pool: Pool,
}

impl PostgresClient {
    pub fn new(db_url: &str) -> Result<PostgresClient, Box<dyn Error>> {
        let cfg = DbConfig::from_url(db_url).map_err(|e| format!("init DbConfig: {}", e))?;
        let pool = cfg
            .pg
            .create_pool(NoTls)
            .map_err(|e| format!("init db pool: {}", e))?;
        Ok(PostgresClient { pool })
    }
}

#[async_trait]
impl DBSaver for PostgresClient {
    async fn live(&self) -> std::result::Result<String, Box<dyn Error>> {
        log::debug!("invoke live");
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("connect db: {}", e))?;
        let stmt = client.prepare_cached("SELECT 1").await?;
        let rows = client.query(&stmt, &[]).await?;
        let value: i32 = rows[0].get(0);
        Ok(format!("{}", value))
    }

    async fn get_last_time(&self, pair: &str) -> Result<DateTime<Utc>, Box<dyn Error>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("connect db: {}", e))?;
        let stmt = client
            .prepare_cached("SELECT MAX(time) from crypto_prices WHERE currency_pair=$1")
            .await?;
        let rows = client.query(&stmt, &[&pair]).await?;
        let value: chrono::DateTime<chrono::offset::Utc> = match rows[0].try_get(0) {
            Ok(ok) => ok,
            Err(_) => Utc.timestamp(0, 0),
        };
        Ok(value)
    }

    async fn save(&self, kline: &KLine) -> Result<bool, Box<dyn Error>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("connect db: {}", e))?;
        let stmt = client
            .prepare_cached("INSERT INTO crypto_prices (time, opening_price, highest_price, lowest_price, closing_price, volume_crypto, currency_pair)
                VALUES ($1, $2, $3, $4, $5, $6, $7)").await?;
        match client
            .execute(
                &stmt,
                &[
                    &kline.open_time(),
                    &kline.open_price,
                    &kline.high_price,
                    &kline.low_price,
                    &kline.close_price,
                    &kline.volume,
                    &kline.pair,
                ],
            )
            .await
        {
            Ok(ok) => Ok(ok),
            Err(err) => {
                if match err.as_db_error() {
                    Some(err) => err.code().code() == "23505",
                    None => false,
                } {
                    log::warn!("postgres err: {err}");
                    return Ok(true);
                }
                Err(err)
            }
        }?;
        Ok(true)
    }
}

#[derive()]
pub struct PostgresClientRetryable {
    client: PostgresClient,
}

impl PostgresClientRetryable {
    pub fn new(client: PostgresClient) -> PostgresClientRetryable {
        PostgresClientRetryable { client }
    }
    fn get_backoff() -> ExponentialBackoff {
        backoff::ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some(Duration::from_secs(15)))
            .build()
    }
}

#[async_trait]
impl DBSaver for PostgresClientRetryable {
    async fn live(&self) -> std::result::Result<String, Box<dyn Error>> {
        retry(
            backoff::ExponentialBackoffBuilder::default()
                .with_max_elapsed_time(Some(Duration::from_secs(3)))
                .build(),
            || async { Ok(self.client.live().await?) },
        )
        .await
    }

    async fn get_last_time(&self, pair: &str) -> Result<DateTime<Utc>, Box<dyn Error>> {
        retry(PostgresClientRetryable::get_backoff(), || async {
            Ok(self.client.get_last_time(pair).await?)
        })
        .await
    }

    async fn save(&self, kline: &KLine) -> Result<bool, Box<dyn Error>> {
        retry(PostgresClientRetryable::get_backoff(), || async {
            Ok(self.client.save(kline).await?)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_url() {
        let cfg =
            DbConfig::from_url("postgres://aa:pass@localhost:1234/db?sslmode=disable").unwrap();
        assert_eq!(cfg.pg.dbname, Some("db".to_string()));
        assert_eq!(cfg.pg.user, Some("aa".to_string()));
        assert_eq!(cfg.pg.password, Some("pass".to_string()));
        assert_eq!(cfg.pg.host, Some("localhost".to_string()));
        assert_eq!(cfg.pg.port, Some(1234));
    }
    #[test]
    #[should_panic]
    fn test_fails_parse_url() {
        DbConfig::from_url("olia://aa:pass@localhost:1234/db?sslmode=disable").unwrap();
    }
}
