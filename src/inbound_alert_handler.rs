use anyhow::{bail, Context, Result};
use reqwest::ClientBuilder;
use serde::Serialize;

use crate::{
    alert::AlertHandler,
    structs::{Config, RunnerStateChange},
};

// the struct to be serialized and sent to the inbound webhook
#[derive(Debug, Serialize)]
struct JSONInboundEvent {
    summary: String,
    event_body: String,
    #[serde(rename = "type")]
    type_field: String,
}

pub struct InboundAlertHandler {}

impl AlertHandler for InboundAlertHandler {
    async fn send_alert(&mut self, cfg: &Config, change: RunnerStateChange) -> Result<()> {
        let (endpoint, msg, summary) = match change {
            RunnerStateChange::Created(new_runner) => {
                let msg = format!(
                    "Now created Runner:\n{}",
                    serde_json::to_string_pretty(&new_runner)?
                );
                (new_runner.webhook_endpoint, msg, "Created new Runner")
            }
            RunnerStateChange::Removed(old_runner) => {
                let msg = format!(
                    "Now removed Runner:\n{}",
                    serde_json::to_string_pretty(&old_runner)?,
                );
                (old_runner.webhook_endpoint, msg, "Removed Runner")
            }
            RunnerStateChange::Offline(old_runner, new_runner) => {
                let msg = format!(
                    "Old Runner:\n{}\n\nNew Runner:\n{}",
                    serde_json::to_string_pretty(&old_runner)?,
                    serde_json::to_string_pretty(&new_runner)?
                );
                (new_runner.webhook_endpoint, msg, "Runner went Offline")
            }
            RunnerStateChange::Online(old_runner, new_runner) => {
                let msg = format!(
                    "Old Runner:\n{}\n\nNew Runner:\n{}",
                    serde_json::to_string_pretty(&old_runner)?,
                    serde_json::to_string_pretty(&new_runner)?
                );
                (new_runner.webhook_endpoint, msg, "Runner came Online")
            }
        };

        self.send_inbound(cfg, &endpoint, &msg, summary).await
    }
}

impl InboundAlertHandler {
    pub fn new() -> Self {
        InboundAlertHandler {}
    }

    async fn send_inbound(
        &self,
        cfg: &Config,
        endpoint: &str,
        event_body: &str,
        summary: &str,
    ) -> Result<()> {
        println!("Sending: {}\n{}", summary, event_body);

        // multiple events could be batched but aren't as of now
        let event_body = serde_json::to_string(&vec![JSONInboundEvent {
            summary: summary.to_string(),
            event_body: event_body.to_string(),
            type_field: "github_monitor".to_string(),
        }])?;

        // TODO: reuse client
        let client = ClientBuilder::new()
            .https_only(!cfg.allow_http)
            .user_agent(format!("gh_runner_uptime v{}", env!("CARGO_PKG_VERSION")))
            .timeout(cfg.inbound_timeout)
            .build()?;

        let resp = client
            .post(endpoint)
            .body(event_body)
            .send()
            .await
            .context("webhook request failed")?;
        if !resp.status().is_success() {
            bail!(
                "webhook request returned status {} for '{}'; return body: {}",
                resp.status(),
                summary,
                resp.text().await?
            );
        }

        Ok(())
    }
}
