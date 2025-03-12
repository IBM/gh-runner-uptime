use anyhow::{ensure, Context, Result};

use crate::{
    inbound::send_inbound,
    structs::{Config, Runner, RunnerMap, RunnerStateChange},
};

pub async fn alert_all_changes_and_update_grace_period(
    cfg: &Config,
    old_runners: &RunnerMap,
    new_runners: &mut RunnerMap,
) -> Result<()> {
    for (old_key, old_runner) in old_runners {
        let new_runner = match new_runners.get_mut(old_key) {
            Some(r) => r,
            None => {
                // the runner doesn't exist no more
                send_alert(cfg, RunnerStateChange::Removed, Some(old_runner), None).await?;
                continue;
            }
        };

        if old_runner
            .interpret_online
            .context("the old runner needs to have interpret_online set")?
            == new_runner.online_for_github_api
        {
            // reset immediately once the old state has reappeared
            new_runner.online_state_change_since = 0;
        } else {
            new_runner.online_state_change_since = old_runner.online_state_change_since + 1;
        }

        if new_runner.online_state_change_since <= cfg.grace_period {
            // all still fine, keep it the way it was before
            new_runner.interpret_online = old_runner.interpret_online;
            // If there has been a state change, it has been noted and will case an event
            // once the grace period runs out.
            continue;
        }

        // consider the state changed now
        new_runner.interpret_online = Some(new_runner.online_for_github_api);
        new_runner.online_state_change_since = 0;

        send_alert(
            cfg,
            if new_runner.online_for_github_api {
                RunnerStateChange::Online
            } else {
                RunnerStateChange::Offline
            },
            // after this old_runner isn't needed any longer
            Some(old_runner),
            Some(&new_runner),
        )
        .await?;
    }

    for (new_key, new_runner) in new_runners {
        if !old_runners.contains_key(new_key) {
            // the runner hasn't existed before
            // initialize the interpreted state to what GitHub thinks
            // this needs to be done before sending the alert
            new_runner.interpret_online = Some(new_runner.online_for_github_api);
            send_alert(cfg, RunnerStateChange::Created, None, Some(new_runner)).await?;
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
    let (endpoint, msg, summary) = match change {
        RunnerStateChange::Created => {
            ensure!(old_runner.is_none());
            let new_runner =
                new_runner.context("new_runner needs to be defined when runner created")?;
            let msg = format!(
                "Now created Runner:\n{}",
                serde_json::to_string_pretty(new_runner)?
            );
            (&new_runner.webhook_endpoint, msg, "Created new Runner")
        }
        RunnerStateChange::Removed => {
            let old_runner =
                old_runner.context("old_runner needs to be defined when runner removed")?;
            ensure!(new_runner.is_none());
            let msg = format!(
                "Now removed Runner:\n{}",
                serde_json::to_string_pretty(old_runner)?,
            );
            (&old_runner.webhook_endpoint, msg, "Removed Runner")
        }
        RunnerStateChange::Offline => {
            let old_runner =
                old_runner.context("old_runner needs to be defined when runner went offline")?;
            let new_runner =
                new_runner.context("new_runner needs to be defined when runner went offline")?;
            let msg = format!(
                "Old Runner:\n{}\n\nNew Runner:\n{}",
                serde_json::to_string_pretty(old_runner)?,
                serde_json::to_string_pretty(new_runner)?
            );
            (&new_runner.webhook_endpoint, msg, "Runner went Offline")
        }
        RunnerStateChange::Online => {
            let old_runner =
                old_runner.context("old_runner needs to be defined when runner came online")?;
            let new_runner =
                new_runner.context("new_runner needs to be defined when runner came online")?;
            let msg = format!(
                "Old Runner:\n{}\n\nNew Runner:\n{}",
                serde_json::to_string_pretty(old_runner)?,
                serde_json::to_string_pretty(new_runner)?
            );
            (&new_runner.webhook_endpoint, msg, "Runner came Online")
        }
    };

    send_inbound(cfg, endpoint, &msg, summary).await
}
