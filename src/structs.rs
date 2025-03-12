use reqwest::Client;
use serde::Serialize;
use std::{collections::HashMap, time::Duration};

#[derive(Debug)]
pub struct Config {
    pub runner_sets: Vec<RunnerSetConfig>,
    pub github_timeout: Duration,
    pub inbound_timeout: Duration,
}
// a runner set is all the runners that belong to a repo or org or enterprise
// one runner set will be queried per github request
#[derive(Debug)]
pub struct RunnerSetConfig {
    pub name: String,
    pub github_endpoint: String,
    pub webhook_endpoint: String,
    pub github_client: Client,
}

// this runner struct will be serialized for the webhook message body
#[derive(Debug, Serialize)]
pub struct Runner {
    pub utc_ping_time: String,
    pub online: bool,
    pub runner_set: String,
    pub id: i64,
    pub name: String,
    pub os: String,
    pub labels: Vec<String>,

    // this contains a secret key
    #[serde(skip_serializing)]
    pub webhook_endpoint: String,
}

pub enum RunnerStateChange {
    // a new runner just popped up
    Created,
    // a known runner isn't there anymore
    Removed,
    // a runner was online the last time and is now offline
    Offline,
    // a runner was offline the last time and is now online
    Online,
}

pub type RunnerMap = HashMap<String, Runner>;
