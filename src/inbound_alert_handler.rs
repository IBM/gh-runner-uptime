use anyhow::{bail, Context, Error, Result};
use reqwest::ClientBuilder;
use tera::Tera;

use crate::{
    alert::AlertHandler,
    structs::{Config, RunnerStateChange},
};

pub struct InboundAlertHandler {
    pub templates: Tera,
}

impl InboundAlertHandler {
    const CREATED_TEMPLATE: &'static str = "created";
    const REMOVED_TEMPLATE: &'static str = "removed";
    const OFFLINE_TEMPLATE: &'static str = "offline";
    const ONLINE_TEMPLATE: &'static str = "online";

    pub fn new(cfg: &Config) -> Result<Self, Error> {
        let mut templates = Tera::default();
        templates.add_template_file(&cfg.created_template_path, Some(Self::CREATED_TEMPLATE))?;
        templates.add_template_file(&cfg.removed_template_path, Some(Self::REMOVED_TEMPLATE))?;
        templates.add_template_file(&cfg.offline_template_path, Some(Self::OFFLINE_TEMPLATE))?;
        templates.add_template_file(&cfg.online_template_path, Some(Self::ONLINE_TEMPLATE))?;
        Ok(InboundAlertHandler { templates })
    }

    async fn send_inbound(&self, cfg: &Config, endpoint: &str, request_body: String) -> Result<()> {
        println!("Sending:\n{}", request_body);
        // TODO: reuse client
        let client = ClientBuilder::new()
            .https_only(!cfg.allow_http)
            .user_agent(format!("gh_runner_uptime v{}", env!("CARGO_PKG_VERSION")))
            .timeout(cfg.inbound_timeout)
            .build()?;

        let resp = client
            .post(endpoint)
            .body(request_body)
            .send()
            .await
            .context("webhook request failed")?;
        if !resp.status().is_success() {
            bail!(
                "webhook request returned status {}; return body: {}",
                resp.status(),
                resp.text().await?
            );
        }
        Ok(())
    }
}

impl AlertHandler for InboundAlertHandler {
    async fn send_alert(&mut self, cfg: &Config, change: RunnerStateChange) -> Result<()> {
        let mut ctx = tera::Context::new();
        let (endpoint, request_body) = match change {
            RunnerStateChange::Created(new_runner) => {
                ctx.insert("new_runner", &new_runner);
                ctx.insert(
                    "new_runner_json",
                    &serde_json::to_string_pretty(&new_runner)?,
                );
                (
                    new_runner.webhook_endpoint,
                    self.templates.render(Self::CREATED_TEMPLATE, &ctx)?,
                )
            }
            RunnerStateChange::Removed(old_runner) => {
                ctx.insert("old_runner", &old_runner);
                ctx.insert(
                    "old_runner_json",
                    &serde_json::to_string_pretty(&old_runner)?,
                );
                (
                    old_runner.webhook_endpoint,
                    self.templates.render(Self::REMOVED_TEMPLATE, &ctx)?,
                )
            }
            RunnerStateChange::Offline(old_runner, new_runner) => {
                ctx.insert("old_runner", &old_runner);
                ctx.insert(
                    "old_runner_json",
                    &serde_json::to_string_pretty(&old_runner)?,
                );
                ctx.insert("new_runner", &new_runner);
                ctx.insert(
                    "new_runner_json",
                    &serde_json::to_string_pretty(&new_runner)?,
                );
                (
                    new_runner.webhook_endpoint,
                    self.templates.render(Self::OFFLINE_TEMPLATE, &ctx)?,
                )
            }
            RunnerStateChange::Online(old_runner, new_runner) => {
                ctx.insert("old_runner", &old_runner);
                ctx.insert(
                    "old_runner_json",
                    &serde_json::to_string_pretty(&old_runner)?,
                );
                ctx.insert("new_runner", &new_runner);
                ctx.insert(
                    "new_runner_json",
                    &serde_json::to_string_pretty(&new_runner)?,
                );
                (
                    new_runner.webhook_endpoint,
                    self.templates.render(Self::ONLINE_TEMPLATE, &ctx)?,
                )
            }
        };
        self.send_inbound(cfg, &endpoint, request_body).await
    }
}
