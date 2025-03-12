use alert::alert_all_changes;
use structs::RunnerMap;
use tokio::signal::unix::{signal, SignalKind};

use crate::github::get_all_runners;

mod alert;
mod config;
mod github;
mod inbound;
mod structs;

#[tokio::main]
async fn main() {
    let (cfg, mut _runners) = config::load_cfg("./config.yaml")
        .await
        .unwrap_or_else(|e| panic!("{:#}", e));

    println!("awaiting sighup");
    // wait for sighup from docker_cron container
    let mut stream = signal(SignalKind::hangup()).unwrap();
    loop {
        // all errors in this loop only restart the loop, the program doesn't crash any more
        stream.recv().await;

        println!("received sighup; starting scan");
        // TODO: remove
        let runners = RunnerMap::new();

        let new_runners = match get_all_runners(&cfg).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{:#}", e);
                continue;
            }
        };
        match alert_all_changes(&cfg, runners, &new_runners).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:#}", e);
                continue;
            }
        }
        _runners = new_runners;
        println!("scan complete");
    }
}
