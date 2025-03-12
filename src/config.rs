use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use serde_yaml::from_reader;
use std::fs::File;
use std::time::Duration;

use crate::github::{
    get_all_runners, get_github_client, get_github_enterprise_endpoint, get_github_org_endpoint,
    get_github_repo_endpoint,
};
use crate::structs::RunnerSetConfig;
use crate::structs::{Config, RunnerMap};

#[derive(Debug, Deserialize)]
struct YAMLConfig {
    #[serde(default)]
    pub orgs: Vec<RunnerSetYAMLConfig>,
    #[serde(default)]
    pub repos: Vec<RunnerSetYAMLConfig>,
    #[serde(default)]
    pub enterprises: Vec<RunnerSetYAMLConfig>,
    pub grace_period: u32,
    #[serde(default = "default_timeout_millis")]
    pub github_timeout_millis: u64,
    #[serde(default = "default_timeout_millis")]
    pub inbound_timeout_millis: u64,

    pub created_template_path: String,
    pub removed_template_path: String,
    pub online_template_path: String,
    pub offline_template_path: String,
}
#[derive(Debug, Deserialize)]
struct RunnerSetYAMLConfig {
    // org, repo or enterprise slug name
    pub name: String,
    pub github_base_uri: String,
    pub github_pat: String,
    // when using the inbound_parser, the access token should be added here
    pub webhook_endpoint: String,
}
fn default_timeout_millis() -> u64 {
    30000
}

pub async fn load_cfg(cfg_path: &str) -> Result<(Config, RunnerMap)> {
    println!("Parsing configuration");

    let file = File::open(cfg_path).context("Unable to open config file")?;
    let yml_cfg: YAMLConfig =
        from_reader(file).with_context(|| format!("Failed to parse yaml config {}", cfg_path))?;

    let github_timeout = Duration::from_millis(yml_cfg.github_timeout_millis);
    let inbound_timeout = Duration::from_millis(yml_cfg.inbound_timeout_millis);

    let org_runner_sets = yml_cfg
        .orgs
        .into_iter()
        .map(|org| -> Result<RunnerSetConfig> {
            Ok(RunnerSetConfig {
                name: format!("org: {}; github: {}", org.name, org.github_base_uri),
                github_endpoint: get_github_org_endpoint(&org.github_base_uri, &org.name),
                webhook_endpoint: org.webhook_endpoint,
                github_client: get_github_client(github_timeout, &org.github_pat, false)?,
            })
        });
    let repo_runner_sets = yml_cfg
        .repos
        .into_iter()
        .map(|repo| -> Result<RunnerSetConfig> {
            Ok(RunnerSetConfig {
                name: format!("repo: {}; github: {}", repo.name, repo.github_base_uri),
                github_endpoint: get_github_repo_endpoint(&repo.github_base_uri, &repo.name),
                webhook_endpoint: repo.webhook_endpoint,
                github_client: get_github_client(github_timeout, &repo.github_pat, false)?,
            })
        });
    let enterprise_runner_sets =
        yml_cfg
            .enterprises
            .into_iter()
            .map(|enterprise| -> Result<RunnerSetConfig> {
                Ok(RunnerSetConfig {
                    name: format!(
                        "enterprise: {}; github: {}",
                        enterprise.name, enterprise.github_base_uri
                    ),
                    github_endpoint: get_github_enterprise_endpoint(
                        &enterprise.github_base_uri,
                        &enterprise.name,
                    ),
                    webhook_endpoint: enterprise.webhook_endpoint,
                    github_client: get_github_client(
                        github_timeout,
                        &enterprise.github_pat,
                        false,
                    )?,
                })
            });
    let runner_sets = org_runner_sets
        .chain(repo_runner_sets)
        .chain(enterprise_runner_sets)
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        !runner_sets.is_empty(),
        "At least one repo, org or enterprise needs to be defined."
    );

    let cfg = Config {
        runner_sets,
        github_timeout,
        inbound_timeout,
        created_template_path: yml_cfg.created_template_path,
        removed_template_path: yml_cfg.removed_template_path,
        online_template_path: yml_cfg.online_template_path,
        offline_template_path: yml_cfg.offline_template_path,

        grace_period: yml_cfg.grace_period,
        allow_http: false,
    };
    println!("Attempting GitHub connections");
    let runners = get_all_runners(&cfg, true).await?;
    Ok((cfg, runners))
}
