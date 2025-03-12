use anyhow::{bail, ensure, Context, Result};
use octocrab::Octocrab;
use std::fs::File;

use serde::Deserialize;
use serde_yaml::from_reader;

#[derive(Debug, Deserialize)]
struct YAMLConfig {
    #[serde(default)]
    pub orgs: Vec<RunnerSetYAMLConfig>,
    #[serde(default)]
    pub repos: Vec<RunnerSetYAMLConfig>,
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

#[derive(Debug)]
pub struct Config {
    pub runner_sets: Vec<RunnerSetConfig>,
}
#[derive(Debug)]
pub struct RunnerSetConfig {
    pub name: String,
    pub github_endpoint: String,
    pub webhook_endpoint: String,
    pub octocrab: Octocrab,
    pub last_online: bool,
}

pub async fn load_cfg(cfg_path: &str) -> Result<Config> {
    println!("Parsing configuration");

    let file = File::open(cfg_path).context("Unable to open config file")?;
    let yml_cfg: YAMLConfig =
        from_reader(file).with_context(|| format!("Failed to parse yaml config {}", cfg_path))?;

    let org_runner_sets = yml_cfg
        .orgs
        .into_iter()
        .map(|org| -> Result<RunnerSetConfig> {
            let octocrab = Octocrab::builder()
                .base_uri(&org.github_base_uri)
                .with_context(|| format!("Invalid base uri for org {}", org.name))?
                .personal_token(org.github_pat)
                .build()
                .with_context(|| {
                    format!("failed to build octocrab instance for org {}", org.name)
                })?;
            Ok(RunnerSetConfig {
                name: format!("org: {}/{}", org.github_base_uri, org.name),
                github_endpoint: format!(
                    "{}/orgs/{}/actions/runners",
                    org.github_base_uri, org.name
                ),
                webhook_endpoint: org.webhook_endpoint,
                octocrab,
                last_online: true,
            })
        });
    let repo_runner_sets = yml_cfg
        .repos
        .into_iter()
        .map(|repo| -> Result<RunnerSetConfig> {
            let octocrab = Octocrab::builder()
                .base_uri(&repo.github_base_uri)
                .with_context(|| format!("Invalid base uri for repo {}", repo.name))?
                .personal_token(repo.github_pat)
                .build()
                .with_context(|| {
                    format!("failed to build octocrab instance for repo {}", repo.name)
                })?;
            Ok(RunnerSetConfig {
                name: format!("repo: {}/{}", repo.github_base_uri, repo.name),
                github_endpoint: format!(
                    "{}/repos/{}/actions/runners",
                    repo.github_base_uri, repo.name
                ),
                webhook_endpoint: repo.webhook_endpoint,
                octocrab,
                last_online: true,
            })
        });
    let runner_sets = org_runner_sets
        .chain(repo_runner_sets)
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        !runner_sets.is_empty(),
        "At least one repo or org needs to be defined."
    );

    Ok(Config { runner_sets })
}
