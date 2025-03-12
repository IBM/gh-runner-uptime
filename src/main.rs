use alert::alert_all_changes_and_update_grace_period;
use anyhow::Result;
use structs::{Config, RunnerMap};
use tokio::signal::unix::{signal, SignalKind};

use crate::{github::get_all_runners, inbound_alert_handler::InboundAlertHandler};

mod alert;
mod config;
mod github;
mod inbound_alert_handler;
mod structs;

#[cfg(test)]
#[path = "./tests/alert_test.rs"]
mod alert_test;
// #[path = "./tests/test_alert_handler.rs"]
// mod test_alert_handler;

async fn perform_scan(cfg: &Config, runners: &mut RunnerMap) -> Result<()> {
    println!("Received sighup; starting scan");
    let mut new_runners = get_all_runners(cfg, false).await?;
    let alert_handler = InboundAlertHandler::new();
    alert_all_changes_and_update_grace_period(cfg, runners, &mut new_runners, &alert_handler)
        .await?;
    // only update runners when changes got transmitted successfully
    // -> retry next time when the service remains in the same new state
    *runners = new_runners;
    println!("Scan complete; {} runners found", runners.len());
    Ok(())
}

#[tokio::main]
async fn main() {
    let (cfg, mut runners) = config::load_cfg("./config.yaml")
        .await
        .unwrap_or_else(|e| panic!("{:#}", e));

    println!("Awaiting sighup");
    // wait for sighup from docker_cron container
    let mut stream = signal(SignalKind::hangup()).unwrap();
    loop {
        // all errors in this loop only restart the loop, the program doesn't crash any more
        stream.recv().await;
        perform_scan(&cfg, &mut runners)
            .await
            .unwrap_or_else(|e| eprintln!("{:#}", e));
    }
}
