use async_trait::async_trait;
use cprices::data::{KLine, Loader};
use std::error::Error;

#[derive(Debug)]
pub struct Binance {
    url: String,
}

impl Binance {
    pub fn new() -> Binance {
        Binance {
            url: "https://api.binance.com".to_string(),
        }
    }
}

#[async_trait]
impl Loader for Binance {
    async fn live(&self) -> std::result::Result<String, Box<dyn Error>> {
        let url = format!("{}/{}", self.url, "api/v3/ping");
        log::debug!("Calling... {} ", url);
        let content = reqwest::get(url).await?.text().await?;
        log::debug!("response: {}", content);
        Ok(content)
    }
    async fn retrieve(
        &self,
        pair: &str,
        interval: &str,
        from: i64,
    ) -> std::result::Result<Vec<KLine>, Box<dyn Error>> {
        let url = format!("{}/{}?symbol={}&interval={}&startTime={}&limit={}", self.url, "api/v3/klines", pair, interval, from, 10);
        log::debug!("Calling... {} ", url);
        let content = reqwest::get(url).await?.text().await?;
        log::debug!("response: {}", content);
        todo!()
    }
}
