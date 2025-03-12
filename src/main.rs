use alert::alert_all_changes;
use structs::RunnerMap;

use crate::github::get_all_runners;

mod alert;
mod config;
mod github;
mod structs;

#[tokio::main]
async fn main() {
    let cfg = config::load_cfg("./config.yaml")
        .await
        .unwrap_or_else(|e| panic!("{:#}", e));

    let mut map = RunnerMap::new();

    let new_map = get_all_runners(&cfg).await.unwrap();
    alert_all_changes(&cfg, map, &new_map).await.unwrap();
    map = new_map;

    let new_map = get_all_runners(&cfg).await.unwrap();
    alert_all_changes(&cfg, map, &new_map).await.unwrap();
    map = new_map;
}
