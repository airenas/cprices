use std::error::Error;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
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
    // pub fn from_arr(line: &str) -> Result<Self, String> {
    //     let strs = line.split_whitespace().collect::<Vec<_>>();
    //     if strs.len() != 4 {
    //         return Err(format!("'{}' is not of len 4", line));
    //     }
    //     //todo: parse as hexadecimal string
    //     match u32::from_str_radix(strs[1], 16) {
    //         Ok(n) => Ok(Event {
    //             id: strs[0].to_string(),
    //             repeat: n,
    //             name: strs[2].to_string(),
    //             device: strs[3].to_string(),
    //         }),
    //         Err(e) => Err(format!("can't parse '{}': {}", strs[1], e)),
    //     }
    // }

    // pub fn to_str(&self) -> String {
    //     format!(
    //         "{} {:x} {} {}",
    //         self.id, self.repeat, self.name, self.device
    //     )
    // }
}

#[async_trait]
pub trait Loader {
    async fn live(&self) -> Result<String, Box<dyn Error>>;
    async fn retrieve(&self, pair: &str, interval: &str, from: i64) -> Result<Vec<KLine>, Box<dyn Error>>;
}

#[cfg(test)]
mod tests {}
