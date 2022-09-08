mod binance;

use chrono::Duration;
use chrono::Utc;
use clap::App;
use clap::Arg;
use cprices::run;
use cprices::WorkingData;
use reqwest::Error;
use std::process;
use tokio::signal;

use binance::Binance;
use cprices::Config;

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
        .get_matches();
    log::info!("Starting Crypto importer");

    let config = Config::build(&matches).unwrap_or_else(|err| {
        log::error!("Problem parsing arguments: {err}");
        process::exit(1)
    });
    log::info!("Pair     {}", config.pair);
    log::info!("Interval {}", config.interval);

    let loader = Binance::new();
    let w_data = WorkingData {
        loader: Box::new(loader),
        config: config,
    };

    if let Err(e) = run(&w_data).await {
        log::error!("Problem parsing arguments: {e}");
        process::exit(1);
    }
    // let resp = reqwest::get("https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=15m&startTime=1403367258000&limit=10")
    //     .await?
    //     .json::<serde_json::Value>()
    //     .await?;
    // for r in resp.as_array() {
    //     for r1 in r {
    //         println!("arr {:#?}", r1);
    //     }
    // }
    // let now = Utc::now();
    // println!("{}", now);

    // let time = now.checked_add_signed(Duration::days(-3000)).unwrap();
    // println!("{}", time);

    // let timestamp: i64 = time.timestamp() * 1000;

    // println!("Current timestamp is {}", timestamp);

    // match signal::ctrl_c().await {
    //     Ok(()) => {
    //         log::debug!("Exit event");
    //     }
    //     Err(err) => {
    //         eprintln!("Unable to listen for shutdown signal: {}", err);
    //         // we also shut down in case of error
    //     }
    // }
    // log::info!("Bye");
    Ok(())
}
