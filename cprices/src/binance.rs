use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cprices::data::{KLine, Loader};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

#[derive(Debug)]
pub struct Binance {
    url: String,
    client: ClientWithMiddleware,
}

impl Binance {
    pub fn new() -> Result<Binance, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Binance {
            url: "https://api.binance.com".to_string(),
            client,
        })
    }
}

#[async_trait]
impl Loader for Binance {
    async fn live(&self) -> std::result::Result<String, Box<dyn Error>> {
        let url = format!("{}/{}", self.url, "api/v3/ping");
        log::debug!("Calling... {} ", url);
        let content = self.client.get(url).send().await?.text().await?;
        log::debug!("response: {}", content);
        Ok(content)
    }
    async fn retrieve(
        &self,
        pair: &str,
        interval: &str,
        from: DateTime<Utc>,
    ) -> std::result::Result<Vec<KLine>, Box<dyn Error>> {
        let url = format!(
            "{}/{}?symbol={}&interval={}&startTime={}&limit={}",
            self.url,
            "api/v3/klines",
            pair,
            interval,
            from.timestamp_millis(),
            100
        );
        log::debug!("Calling... {} ", url);
        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Vec<BinanceKLine>>()
            .await?;
        let res = resp.iter().map(|d| to_kline(d, pair)).collect();
        Ok(res)
    }
}

fn to_kline(d: &BinanceKLine, pair: &str) -> KLine {
    KLine {
        open_time: d.open_time,
        open_price: d.open_price,
        high_price: d.high_price,
        low_price: d.low_price,
        close_price: d.close_price,
        volume: d.volume,
        close_time: d.close_time,
        pair: pair.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct BinanceKLine {
    // 1499040000000,      // Open time
    // "0.01634790",       // Open
    // "0.80000000",       // High
    // "0.01575800",       // Low
    // "0.01577100",       // Close
    // "148976.11427815",  // Volume
    // 1499644799999,      // Close time
    // "2434.19055334",    // Quote asset volume
    // 308,                // Number of trades
    // "1756.87402397",    // Taker buy base asset volume
    // "28.46694368",      // Taker buy quote asset volume
    // "17928899.62484339" // Ignore
    pub open_time: i64,
    #[serde(deserialize_with = "string_as_f64")]
    pub open_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub high_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub low_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub close_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub volume: f64,
    pub close_time: i64,
    #[serde(deserialize_with = "string_as_f64")]
    pub asset_volume: f64,
    pub trades: i64,
    #[serde(deserialize_with = "string_as_f64")]
    pub base_asset_volume: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub quote_asset_volume: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub other: f64,
}

fn string_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(F64Visitor)
}

struct F64Visitor;
impl<'de> Visitor<'de> for F64Visitor {
    type Value = f64;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("aaa string representation of a f64")
    }
    fn visit_str<E>(self, value: &str) -> Result<f64, E>
    where
        E: de::Error,
    {
        value.parse::<f64>().map_err(|_err| {
            E::invalid_value(
                Unexpected::Str(value),
                &"abb string representation of a f64",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::binance::{to_kline, BinanceKLine};

    fn one_sample() -> &'static str {
        r#"[1502942400000,
        "4261.48000000",
        "4313.62000000",
        "4261.32000000",
        "4308.83000000",
        "47.18100900",
        1502945999999,
        "202366.13839304",
        171,
        "35.16050300",
        "150952.47794304",
    "0"
    ]"#
    }

    #[test]
    fn deseriarelize_binance_kline() {
        let deserialized: BinanceKLine = serde_json::from_str(one_sample()).unwrap();
        assert_eq!(deserialized.open_time, 1502942400000);
        assert_eq!(deserialized.close_time, 1502945999999);
        assert_relative_eq!(deserialized.open_price, 4261.48);
    }

    #[test]
    fn deseriarelize_binance_vec_kline() {
        let deserialized: Vec<BinanceKLine> =
            serde_json::from_str(&("[".to_owned() + one_sample() + "]")).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0].open_time, 1502942400000);
        assert_eq!(deserialized[0].close_time, 1502945999999);
        assert_relative_eq!(deserialized[0].open_price, 4261.48);
    }
    #[test]
    fn deseriarelize_binance_vec_several_kline() {
        let deserialized: Vec<BinanceKLine> =
            serde_json::from_str(&("[".to_owned() + one_sample() + "," + one_sample() + "]"))
                .unwrap();
        assert_eq!(deserialized.len(), 2);
    }
    #[test]
    fn mapt_to_kline() {
        let deserialized: BinanceKLine = serde_json::from_str(one_sample()).unwrap();
        let kline = to_kline(&deserialized, "olia");
        assert_eq!(kline.open_time, 1502942400000);
        assert_eq!(kline.close_time, 1502945999999);
        assert_eq!(kline.pair, "olia");
    }
}
