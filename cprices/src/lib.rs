pub mod data;

use clap::ArgMatches;
use data::Loader;
use std::error::Error;

pub struct Config {
    pub pair: String,
    pub interval: String,
}

impl Config {
    pub fn build(args: &ArgMatches) -> Result<Config, &'static str> {
        let pair = args.value_of("pair").unwrap_or("BTCUSDT");
        let interval = args.value_of("interval").unwrap_or("15m");
        Ok(Config {
            pair: pair.to_string(),
            interval: interval.to_string(),
        })
    }
}

pub struct WorkingData {
    pub config: Config,
    pub loader: Box<dyn Loader>,
}

pub async fn run(w_data: &WorkingData) -> Result<(), Box<dyn Error>> {
    log::info!("Test Binance is live");
    match w_data.loader.live().await {
        Ok(_) => {
            log::info!("Binance OK");
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(err);
        }
    }
    log::info!(
        "Getting klines {} for {}",
        w_data.config.interval,
        w_data.config.pair
    );

    let klines = w_data
        .loader
        .retrieve(
            w_data.config.pair.as_str(),
            w_data.config.interval.as_str(),
            0,
        )
        .await?;

    Ok(())
}
