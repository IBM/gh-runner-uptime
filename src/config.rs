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
    pub github_endpoint: String,
    pub webhook_endpoint: String,
    pub octocrab: Octocrab,
}

pub async fn load_cfg(cfg_path: &str) -> Config {
    println!("Parsing configuration");

    let file =
        File::open(cfg_path).unwrap_or_else(|e| panic!("Unable to open config file: {:?}", e));
    let yml_cfg: YAMLConfig = from_reader(file)
        .unwrap_or_else(|e| panic!("Failed to parse yaml config {:?}: {:?}", cfg_path, e));

    let mut cfg = Config {
        runner_sets: vec![],
    };

    cfg.runner_sets.extend(yml_cfg.orgs.into_iter().map(|org| {
        let octocrab = Octocrab::builder()
            .base_uri(org.github_base_uri)
            .unwrap_or_else(|e| panic!("Invalid base uri: {:?}", e))
            .personal_token(org.github_pat)
            .build()
            .unwrap_or_else(|e| {
                panic!(
                    "failed to build octocrab instance for org {:?}: {:?}",
                    org.name, e
                )
            });
        let github_endpoint = format!("/orgs/{:?}/actions/runners", org.name);
        RunnerSetConfig {
            github_endpoint,
            webhook_endpoint: org.webhook_endpoint,
            octocrab,
        }
    }));
    cfg.runner_sets
        .extend(yml_cfg.repos.into_iter().map(|repo| {
            let octocrab = Octocrab::builder()
                .base_uri(repo.github_base_uri)
                .unwrap_or_else(|e| panic!("Invalid base uri: {:?}", e))
                .personal_token(repo.github_pat)
                .build()
                .unwrap_or_else(|e| {
                    panic!(
                        "failed to build octocrab instance for repo {:?}: {:?}",
                        repo.name, e
                    )
                });
            let github_endpoint = format!("/repo/{:?}/actions/runners", repo.name);
            RunnerSetConfig {
                github_endpoint,
                webhook_endpoint: repo.webhook_endpoint,
                octocrab,
            }
        }));

    assert!(
        cfg.runner_sets.len() != 0,
        "at least one repo or org needs to be defined"
    );

    cfg
}
