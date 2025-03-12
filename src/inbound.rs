use anyhow::{bail, Context, Result};
use reqwest::ClientBuilder;
use serde::Serialize;

use crate::structs::Config;

#[derive(Debug, Serialize)]
struct JSONInboundEvent {
    summary: String,
    event_body: String,
    #[serde(rename = "type")]
    type_field: String,
}

pub async fn send_inbound(
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
        .https_only(true)
        .user_agent("github uptime monitor")
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
            "webhook request returned status {} for {}; return body: {}",
            resp.status(),
            summary,
            resp.text().await?
        );
    }

    Ok(())
}
