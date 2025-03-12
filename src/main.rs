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
    let (cfg, mut runners) = config::load_cfg("./config.yaml")
        .await
        .unwrap_or_else(|e| panic!("{:#}", e));

    // wait for sighup from docker_cron container
    let mut stream = signal(SignalKind::hangup()).unwrap();
    loop {
        stream.recv().await;

        // TODO: remove
        runners = RunnerMap::new();

        let new_runners = get_all_runners(&cfg).await.unwrap();
        alert_all_changes(&cfg, runners, &new_runners)
            .await
            .unwrap();
        runners = new_runners;
    }
}
