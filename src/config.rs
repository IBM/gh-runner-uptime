use anyhow::{ensure, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use serde_yaml::from_reader;
use std::fs::File;
use std::time::Duration;

use crate::github::get_all_runners;
use crate::structs::Config;
use crate::structs::RunnerSetConfig;

#[derive(Debug, Deserialize)]
struct YAMLConfig {
    #[serde(default)]
    pub orgs: Vec<RunnerSetYAMLConfig>,
    #[serde(default)]
    pub repos: Vec<RunnerSetYAMLConfig>,
    #[serde(default = "default_timeout_millis")]
    pub github_timeout_millis: u64,
}
#[derive(Debug, Deserialize)]
struct RunnerSetYAMLConfig {
    // org or repo string
    pub name: String,
    pub github_base_uri: String,
    pub github_pat: String,
    // when using the inbound_parser, the access token should be added here
    pub webhook_endpoint: String,
}
fn default_timeout_millis() -> u64 {
    30000
}

pub async fn load_cfg(cfg_path: &str) -> Result<Config> {
    println!("Parsing configuration");

    let file = File::open(cfg_path).context("Unable to open config file")?;
    let yml_cfg: YAMLConfig =
        from_reader(file).with_context(|| format!("Failed to parse yaml config {}", cfg_path))?;

    let github_timeout = Duration::from_millis(yml_cfg.github_timeout_millis);

    let org_runner_sets = yml_cfg
        .orgs
        .into_iter()
        .map(|org| -> Result<RunnerSetConfig> {
            Ok(RunnerSetConfig {
                name: format!("org: {}; github: {}", org.name, org.github_base_uri),
                github_endpoint: format!(
                    "{}/orgs/{}/actions/runners",
                    org.github_base_uri, org.name
                ),
                webhook_endpoint: org.webhook_endpoint,
                github_client: get_github_client(github_timeout, &org.github_pat)?,
            })
        });
    let repo_runner_sets = yml_cfg
        .repos
        .into_iter()
        .map(|repo| -> Result<RunnerSetConfig> {
            Ok(RunnerSetConfig {
                name: format!("repo: {}; github: {}", repo.name, repo.github_base_uri),
                github_endpoint: format!(
                    "{}/repos/{}/actions/runners",
                    repo.github_base_uri, repo.name
                ),
                webhook_endpoint: repo.webhook_endpoint,
                github_client: get_github_client(github_timeout, &repo.github_pat)?,
            })
        });
    let runner_sets = org_runner_sets
        .chain(repo_runner_sets)
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        !runner_sets.is_empty(),
        "At least one repo or org needs to be defined."
    );

    let cfg = Config {
        runner_sets,
        github_timeout,
    };
    println!("attempting GitHub connections");
    get_all_runners(&cfg).await?;
    Ok(cfg)
}

fn get_github_client(timeout: Duration, pat: &str) -> Result<Client> {
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
