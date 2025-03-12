use anyhow::{bail, Result};
use chrono::Utc;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, ClientBuilder,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::structs::{Config, Runner, RunnerMap, RunnerSetConfig};

#[derive(Debug, Deserialize, Serialize)]
pub struct JSONRunnerSetResponse {
    // not needed but supplied by github
    // pub total_count: i64,
    pub runners: Vec<JSONRunnerResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JSONRunnerResponse {
    pub id: i64,
    pub name: String,
    pub os: String,
    pub status: String,
    pub busy: bool,
    pub labels: Vec<JSONLabelResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JSONLabelResponse {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

async fn get_runners_for_set(runner_set: &RunnerSetConfig) -> Result<RunnerMap> {
    let resp = runner_set
        .github_client
        .get(&runner_set.github_endpoint)
        .send()
        .await?;
    if !resp.status().is_success() {
        bail!(
            "github runner api request returned status {} for {}; return body: {}",
            resp.status(),
            runner_set.name,
            resp.text().await?
        );
    }
    let json_resp = resp.json::<JSONRunnerSetResponse>().await?;

    json_resp
        .runners
        .into_iter()
        .map(|r| parse_runner(r, runner_set))
        .collect::<Result<RunnerMap>>()
}

// return key and value for hashmap
fn parse_runner(
    json_runner: JSONRunnerResponse,
    runner_set: &RunnerSetConfig,
) -> Result<(String, Runner)> {
    let runner = Runner {
        utc_ping_time: Utc::now().to_rfc3339(),
        online: json_runner.status == "online",
        runner_set: runner_set.name.clone(),
        id: json_runner.id,
        name: json_runner.name,
        os: json_runner.os,
        labels: json_runner.labels.into_iter().map(|l| l.name).collect(),
        webhook_endpoint: runner_set.webhook_endpoint.clone(),
    };
    Ok((
        format!("{}; runner id: {}", runner.runner_set, runner.id),
        runner,
    ))
}

pub async fn get_all_runners(cfg: &Config) -> Result<RunnerMap> {
    let mut map = RunnerMap::new();
    for runner_set in &cfg.runner_sets {
        map.extend(get_runners_for_set(runner_set).await?);
    }
    Ok(map)
}

pub fn get_github_client(timeout: Duration, pat: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", pat))?,
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    let client = ClientBuilder::new()
        .https_only(true)
        .user_agent("github uptime monitor")
        .default_headers(headers)
        .timeout(timeout)
        .build()?;

    Ok(client)
}
// https://docs.github.com/en/enterprise-server@3.12/rest/actions/self-hosted-runners?apiVersion=2022-11-28#list-runner-applications-for-an-organization
pub fn get_github_repo_endpoint(base_uri: &str, repo_name: &str) -> String {
    format!("{}/repos/{}/actions/runners", base_uri, repo_name)
}
// https://docs.github.com/en/enterprise-server@3.12/rest/actions/self-hosted-runners?apiVersion=2022-11-28#list-self-hosted-runners-for-a-repository
pub fn get_github_org_endpoint(base_uri: &str, org_name: &str) -> String {
    format!("{}/orgs/{}/actions/runners", base_uri, org_name)
}
