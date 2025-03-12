use anyhow::{ensure, Context, Result};

use crate::structs::{Config, Runner, RunnerMap, RunnerStateChange};

pub async fn alert_all_changes<'a>(
    cfg: &Config,
    old_runners: RunnerMap,
    new_runners: &RunnerMap,
) -> Result<()> {
    for (old_key, old_runner) in &old_runners {
        let new_runner = match new_runners.get(old_key) {
            Some(r) => r,
            None => {
                // the runner doesn't exist no more
                send_alert(&cfg, RunnerStateChange::Removed, Some(old_runner), None).await?;
                continue;
            }
        };
        if old_runner.online == new_runner.online {
            // all fine, nothing changed
            continue;
        }

        send_alert(
            &cfg,
            if new_runner.online {
                RunnerStateChange::Online
            } else {
                RunnerStateChange::Offline
            },
            Some(old_runner),
            Some(new_runner),
        )
        .await?;
    }

    for (new_key, new_runner) in new_runners {
        if !old_runners.contains_key(new_key) {
            // the runner hasn't existed before
            send_alert(&cfg, RunnerStateChange::Created, None, Some(new_runner)).await?;
        }
    }
    Ok(())
}

async fn send_alert(
    cfg: &Config,
    change: RunnerStateChange,
    old_runner: Option<&Runner>,
    new_runner: Option<&Runner>,
) -> Result<()> {
    match change {
        RunnerStateChange::Created => {
            ensure!(old_runner.is_none());
            let new_runner =
                &new_runner.context("new_runner needs to be defined when runner created")?;

            let msg = format!(
                "Now created Runner:\n{}",
                serde_json::to_string_pretty(&new_runner)?
            );

            send_inbound(cfg, &msg, "Created new Runner").await?
        }
        RunnerStateChange::Removed => {
            let old_runner =
                &old_runner.context("old_runner needs to be defined when runner removed")?;
            ensure!(new_runner.is_none());

            let msg = format!(
                "Now removed Runner:\n{}",
                serde_json::to_string_pretty(&old_runner)?,
            );

            send_inbound(cfg, &msg, "Removed Runner").await?
        }
        RunnerStateChange::Offline => {
            let old_runner =
                &old_runner.context("old_runner needs to be defined when runner went offline")?;
            let new_runner =
                &new_runner.context("new_runner needs to be defined when runner went offline")?;

            let msg = format!(
                "Old Runner:\n{}\n\nNew Runner:\n{}",
                serde_json::to_string_pretty(&old_runner)?,
                serde_json::to_string_pretty(&new_runner)?
            );

            send_inbound(cfg, &msg, "Runner went Offline").await?
        }
        RunnerStateChange::Online => {
            let old_runner =
                &old_runner.context("old_runner needs to be defined when runner came online")?;
            let new_runner =
                &new_runner.context("new_runner needs to be defined when runner came online")?;

            let msg = format!(
                "Old Runner:\n{}\n\nNew Runner:\n{}",
                serde_json::to_string_pretty(&old_runner)?,
                serde_json::to_string_pretty(&new_runner)?
            );

            send_inbound(cfg, &msg, "Runner came Online").await?
        }
    }

    Ok(())
}

async fn send_inbound(cfg: &Config, description: &str, summary: &str) -> Result<()> {
    println!("Sending: {}\n{}", summary, description);
    Ok(())
}
