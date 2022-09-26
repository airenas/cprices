pub mod data;

use clap::ArgMatches;
use data::{DBSaver, Loader};
use std::error::Error;

pub struct Config {
    pub pair: String,
    pub interval: String,
    pub db_url: String,
}

impl Config {
    pub fn build(args: &ArgMatches) -> Result<Config, &'static str> {
        let pair = args.value_of("pair").unwrap_or("BTCUSDT");
        let interval = args.value_of("interval").unwrap_or("15m");
        let db_url = args.value_of("db_url").expect("no db_url provided");
        Ok(Config {
            pair: pair.to_string(),
            interval: interval.to_string(),
            db_url: db_url.to_string(),
        })
    }
}

pub struct WorkingData {
    pub config: Config,
    pub loader: Box<dyn Loader>,
    pub saver: Box<dyn DBSaver>,
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

    log::info!("Test Postgres is live");
    match w_data.saver.live().await {
        Ok(_) => {
            log::info!("Postgresql OK");
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(err);
        }
    }

    log::info!("Get last value in DB");
    let last_time = match w_data.saver.get_last_time(&w_data.config.pair).await {
        Ok(v) => {
            log::info!("Got last time: {}", v);
            v
        }
        Err(err) => {
            log::error!("{}", err);
            return Err(err);
        }
    };

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
            last_time,
        )
        .await?;
    klines.iter().for_each(|f| {
        log::info!("{}", f.to_str());
        // w_data.saver.save(f).await;
    });

    for line in klines {
        w_data.saver.save(&line).await?;
    }

    Ok(())
}
