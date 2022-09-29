mod binance;
mod limiter;
mod postgresql;

use clap::App;
use clap::Arg;
use cprices::data::KLine;
use cprices::data::Limiter;
use cprices::WorkingData;
use cprices::{get_last_time, run, saver_start};
use reqwest::Error;
use std::process;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use binance::Binance;
use cprices::Config;
use postgresql::PostgresClient;

use crate::limiter::RateLimiter;
use crate::postgresql::PostgresClientRetryable;
use cprices::data::DBSaver;
use futures::future::join_all;

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
                .help("Crypto pairs separated by comma, e.g. : BTCUSDT,ETHUSDT")
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
    log::info!("Pair     {}", config.pairs.join(","));
    log::info!("Interval {}", config.interval);

    let db_saver = PostgresClient::new(&config.db_url).unwrap_or_else(|err| {
        log::error!("postgres client init: {err}");
        process::exit(1)
    });
    let db_saver = PostgresClientRetryable::new(db_saver);
    let boxed_db_saver: Box<dyn DBSaver> = Box::new(db_saver);
    log::info!("Test Postgres is live ...");
    boxed_db_saver.live().await.unwrap();
    log::info!("Postgresql OK");

    let mut imports = Vec::new();
    let limiter = RateLimiter::new().unwrap();
    let boxed_limiter: Box<dyn Limiter> = Box::new(limiter);
    let limiter = Arc::new(Mutex::new(boxed_limiter));

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    for pair in config.pairs {
        let loader = Binance::new().unwrap();

        let interval = config.interval.clone();
        let int_limiter = limiter.clone();
        let int_tx = tx.clone();
        let start_from = get_last_time(&boxed_db_saver, &pair).await.unwrap();
        let w_data = WorkingData {
            loader: Box::new(loader),
            pair,
            interval,
            start_from,
            sender: int_tx,
            limiter: int_limiter,
        };

        imports.push(run(w_data));
    }
    tokio::spawn(async move { start_saver_loop(&config.db_url, &mut rx).await });
    // imports.push(saver_start(boxed_db_saver, &mut rx));

    join_all(imports).await.iter().for_each(|err| {
        if let Err(e) = err {
            log::error!("Problem importing {e}");
        }
    });

    // if let Err(e) = run(&w_data).await {
    //         log::error!("Problem importing arguments: {e}");
    //         process::exit(1);
    //     }

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

async fn start_saver_loop(db_url: &str, receiver: &mut Receiver<KLine>) -> Result<(), String> {
    log::info!("start saver loop");
    let db_saver = PostgresClient::new(db_url).unwrap_or_else(|err| {
        log::error!("postgres client init: {err}");
        process::exit(1)
    });
    let db_saver = PostgresClientRetryable::new(db_saver);
    let boxed_db_saver: Box<dyn DBSaver + Send + Sync> = Box::new(db_saver);
    log::info!("Test Postgres is live ...");
    boxed_db_saver.live().await.unwrap();
    log::info!("Postgresql OK");

    saver_start(boxed_db_saver, receiver).await.unwrap();

    log::info!("exit loop");
    Ok(())
}
