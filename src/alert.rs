use anyhow::{Context, Result};

use crate::structs::{Config, RunnerMap, RunnerStateChange};

pub trait AlertHandler {
    async fn send_alert(&self, cfg: &Config, change: RunnerStateChange) -> Result<()>;
}

pub async fn alert_all_changes_and_update_grace_period(
    cfg: &Config,
    old_runners: &RunnerMap,
    new_runners: &mut RunnerMap,
    alert_handler: &impl AlertHandler,
) -> Result<()> {
    for (old_key, old_runner) in old_runners {
        let new_runner = match new_runners.get_mut(old_key) {
            Some(r) => r,
            None => {
                // the runner doesn't exist no more
                alert_handler
                    .send_alert(cfg, RunnerStateChange::Removed(old_runner))
                    .await?;
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

        alert_handler
            .send_alert(
                cfg,
                if new_runner.online_for_github_api {
                    RunnerStateChange::Online(old_runner, new_runner)
                } else {
                    RunnerStateChange::Offline(old_runner, new_runner)
                },
            )
            .await?;
    }

    for (new_key, new_runner) in new_runners {
        if !old_runners.contains_key(new_key) {
            // the runner hasn't existed before
            // initialize the interpreted state to what GitHub thinks
            // this needs to be done before sending the alert
            new_runner.interpret_online = Some(new_runner.online_for_github_api);
            alert_handler
                .send_alert(cfg, RunnerStateChange::Created(new_runner))
                .await?;
        }
    }
    Ok(())
}
