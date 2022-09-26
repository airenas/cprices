use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct KLine {
    pub open_time: i64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub close_price: f64,
    pub volume: f64,
    pub close_time: i64,
    pub pair: String,
}

impl KLine {
    pub fn to_str(&self) -> String {
        format!(
            "pair: {}, time: {}, price {}",
            self.pair, self.open_time, self.open_price
        )
    }
}
#[async_trait]
pub trait Loader {
    async fn live(&self) -> Result<String, Box<dyn Error>>;
    async fn retrieve(
        &self,
        pair: &str,
        interval: &str,
        from: i64,
    ) -> Result<Vec<KLine>, Box<dyn Error>>;
}

#[async_trait]
pub trait DBSaver {
    async fn live(&self) -> Result<String, Box<dyn Error>>;
    async fn get_last_time(&self, pair: &str) -> Result<i64, Box<dyn Error>>;
    async fn save(&self, data: &KLine) -> Result<bool, Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use crate::data::KLine;
    #[test]
    fn to_string() {
        assert_eq!(
            KLine {
                open_time: 10,
                open_price: 1.0,
                high_price: 2.0,
                low_price: 0.1,
                close_price: 1.5,
                volume: 10.0,
                close_time: 15,
                pair: "olia".to_string(),
            }
            .to_str(),
            "pair: olia, time: 10, price 1"
        );
    }
}
