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
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use binance::Binance;
use cprices::Config;
use postgresql::PostgresClient;

use crate::limiter::RateLimiter;
use crate::postgresql::PostgresClientRetryable;
use cprices::data::DBSaver;
use futures::future::join_all;
use tokio::signal;
use tokio::sync::broadcast;

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
    let boxed_db_saver: Box<dyn DBSaver + Send + Sync> = Box::new(db_saver);
    log::info!("Test Postgres is live ...");
    boxed_db_saver.live().await.unwrap();
    log::info!("Postgresql OK");

    let mut imports = Vec::new();
    let limiter = RateLimiter::new().unwrap();
    let boxed_limiter: Box<dyn Limiter> = Box::new(limiter);
    let limiter = Arc::new(Mutex::new(boxed_limiter));

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let (tx_close, _) = broadcast::channel(2);
    let (tx_wait_exit, mut rx_wait_exit) = tokio::sync::mpsc::channel(1);

    for pair in config.pairs {
        let loader = Binance::new().unwrap();

        let interval = config.interval.clone();
        let int_limiter = limiter.clone();
        let start_from = get_last_time(boxed_db_saver.as_ref(), &pair).await.unwrap();
        let w_data = WorkingData {
            loader: Box::new(loader),
            pair,
            interval,
            start_from,
            sender: tx.clone(),
            limiter: int_limiter,
        };

        imports.push(run(w_data, tx_close.clone()));
    }
    let int_exit = tx_wait_exit.clone();
    tokio::spawn(async move { start_saver_loop(boxed_db_saver, &mut rx, int_exit).await });

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                log::debug!("Exit event");
            }
            Err(err) => {
                log::error!("Unable to listen for shutdown signal: {}", err);
            }
        }
        log::debug!("sending exit event");
        tx_close.send(0).unwrap();
    });

    drop(tx_wait_exit);
    drop(tx);

    join_all(imports).await.iter().for_each(|err| {
        if let Err(e) = err {
            log::error!("problem importing: {e}");
        }
    });

    log::info!("wait jobs to finish");
    let _ = rx_wait_exit.recv().await;

    log::info!("Bye");
    Ok(())
}

async fn start_saver_loop(
    db_saver: Box<dyn DBSaver + Send + Sync>,
    receiver: &mut Receiver<KLine>,
    _tx_exit: Sender<()>,
) -> Result<(), String> {
    log::info!("Test Postgres is live ...");
    db_saver.live().await.unwrap();
    log::info!("Postgresql OK");

    saver_start(db_saver, receiver).await.unwrap();

    log::info!("exit loop");
    Ok(())
}
