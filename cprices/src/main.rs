mod binance;
mod limiter;
mod postgresql;

use clap::App;
use clap::Arg;
use cprices::run;
use cprices::WorkingData;
use reqwest::Error;
use std::process;

use binance::Binance;
use cprices::Config;
use postgresql::PostgresClient;

use crate::limiter::RateLimiter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let matches = App::new("importer")
        .version("0.1")
        .author("Airenas V.<airenass@gmail.com>")
        .about("Imports Binance crypto Klines to local timescaleDB")
        .arg(
            Arg::new("pair")
                .short('p')
                .long("pair")
                .value_name("PAIR")
                .help("Crypto pair, e.g. : BTCUSDT")
                .takes_value(true),
        )
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .value_name("INTERVAL")
                .help("KLine interval, e.g. : 15m")
                .takes_value(true),
        )
        .arg(
            Arg::new("db_url")
                .short('u')
                .long("db-url")
                .value_name("URL")
                .help("TimescaleDb URL e.g.: postgres://postgres:pass@localhost/crypto")
                .takes_value(true),
        )
        .get_matches();
    log::info!("Starting Crypto importer");

    let config = Config::build(&matches).unwrap_or_else(|err| {
        log::error!("Problem parsing arguments: {err}");
        process::exit(1)
    });
    log::info!("Pair     {}", config.pair);
    log::info!("Interval {}", config.interval);

    let loader = Binance::new().unwrap();
    let db_saver = PostgresClient::new(&config.db_url).unwrap_or_else(|err| {
        log::error!("postgres client init: {err}");
        process::exit(1)
    });
    let limiter = RateLimiter::new().unwrap();
    let w_data = WorkingData {
        loader: Box::new(loader),
        config,
        saver: Box::new(db_saver),
        limiter: Box::new(limiter),
    };

    if let Err(e) = run(&w_data).await {
        log::error!("Problem parsing arguments: {e}");
        process::exit(1);
    }
    // match signal::ctrl_c().await {
    //     Ok(()) => {
    //         log::debug!("Exit event");
    //     }
    //     Err(err) => {
    //         eprintln!("Unable to listen for shutdown signal: {}", err);
    //         // we also shut down in case of error
    //     }
    // }
    log::info!("Bye");
    Ok(())
}
