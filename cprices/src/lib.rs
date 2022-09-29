pub mod data;

use chrono::{DateTime, Utc};
use clap::ArgMatches;
use data::{DBSaver, KLine, Limiter, Loader};
use std::error::Error;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

pub struct Config {
    pub pairs: Vec<String>,
    pub interval: String,
    pub db_url: String,
}

impl Config {
    pub fn build(args: &ArgMatches) -> Result<Config, &'static str> {
        let pair = args.value_of("pair").unwrap_or("BTCUSDT");
        let interval = args.value_of("interval").unwrap_or("1h");
        let db_url = args.value_of("db_url").expect("no db_url provided");
        let pairs = pair.split(',').map(String::from).collect();
        Ok(Config {
            pairs,
            interval: interval.to_string(),
            db_url: db_url.to_string(),
        })
    }
}

type LimiterM = std::sync::Arc<Mutex<Box<dyn Limiter>>>;
type ResultM = Result<(), Box<dyn Error>>;

pub struct WorkingData {
    pub pair: String,
    pub interval: String,
    pub start_from: DateTime<Utc>,
    pub loader: Box<dyn Loader>,
    pub limiter: LimiterM,
    pub sender: Sender<KLine>,
}

pub async fn run(w_data: WorkingData) -> ResultM {
    log::info!("Importing: {}, from {}", w_data.pair, w_data.start_from);
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
    let mut last_time = w_data.start_from;
    let dur = chrono::Duration::from_std(
        duration_str::parse(&w_data.interval).map_err(|e| format!("duration  parse: {}", e))?,
    )
    .map_err(|e| format!("duration parse: {}", e))?;
    loop{
        let td = last_time - (Utc::now() - dur);
        if td < chrono::Duration::zero() {
            last_time = import(&w_data, last_time).await?;
        } else {
            log::info!("sleep {}", td.to_string());
            tokio::time::sleep(td.to_std()?).await;
        }
    }
    Ok(())
}

pub async fn get_last_time(
    db: &Box<dyn DBSaver>,
    pair: &str,
) -> Result<DateTime<Utc>, Box<dyn Error>> {
    log::info!("Get last value in DB for {}", pair);
    db.get_last_time(pair)
        .await
        .map_err(|e| format!("get pair's '{}' from: {}", pair, e).into())
}

async fn import(
    w_data: &WorkingData,
    from: DateTime<Utc>,
) -> Result<DateTime<Utc>, Box<dyn Error>> {
    {
        log::info!("wait for import");
        let wait = w_data.limiter.lock().await;
        wait.wait().await?;
        log::info!("let's go");
    }
    log::info!(
        "Getting klines {} for {} from {}",
        w_data.interval,
        w_data.pair,
        from
    );

    let klines = w_data
        .loader
        .retrieve(w_data.pair.as_str(), w_data.interval.as_str(), from)
        .await?;
    klines.iter().for_each(|f| {
        log::trace!("{}", f.to_str());
        // w_data.saver.save(f).await;
    });
    log::info!("got {} lines", klines.len());
    let mut res = from;
    for line in klines {
        if res < line.open_time() {
            res = line.open_time();
        }
        w_data.sender.send(line).await?;
    }
    log::debug!("send lines to save");
    Ok(res)
}

pub async fn saver_start(
    db: Box<dyn DBSaver + Send + Sync>,
    receiver: &mut Receiver<KLine>,
) -> Result<(), String> {
    log::info!("start db saver loop");
    loop {
        let line = receiver.recv().await;
        log::debug!("got line");
        match line {
            Some(line) => db.save(&line).await.map(|_v| ()).map_err(|err| format!("save err: {}", err))?,
            None => todo!(),
        }
    }
}
