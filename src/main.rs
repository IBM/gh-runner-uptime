use anyhow::{bail, Context, Result};
use http_body_util::BodyExt;

mod config;

async fn is_runner_set_online(runner_set: &mut config::RunnerSetConfig) -> Result<bool> {
    println!("{}", &runner_set.github_endpoint);
    let resp = runner_set
        .octocrab
        ._get(&runner_set.github_endpoint)
        .await
        .with_context(|| format!("runner api request failed for {}", runner_set.name))?;

    if !resp.status().is_success() {
        bail!(
            "github runner api request returned status {} for {}",
            resp.status(),
            runner_set.name
        );
    }
    let t = resp.into_body().collect().await?.to_bytes();
    println!("{:?}", t);

    Ok(false)
}

async fn are_all_online(cfg: &mut config::Config) -> bool {
    for runner_set in &mut cfg.runner_sets {
        match is_runner_set_online(runner_set).await {
            Err(e) => {
                eprintln!("{:#}", e);
                return false;
            }
            Ok(false) => return false,
            Ok(true) => {}
        }
    }
    true
}

#[tokio::main]
async fn main() {
    let mut cfg = config::load_cfg("./config.yaml")
        .await
        .unwrap_or_else(|e| panic!("{:#}", e));

    match are_all_online(&mut cfg).await {
        true => println!("all runners are online"),
        false => println!("not all runners are online"),
    }
}
