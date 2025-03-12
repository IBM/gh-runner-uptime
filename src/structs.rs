use reqwest::Client;
use serde::Serialize;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// config //
#[derive(Debug)]
pub struct Config {
    pub runner_sets: Vec<RunnerSetConfig>,
    pub github_timeout: Duration,
}
#[derive(Debug)]
pub struct RunnerSetConfig {
    pub name: String,
    pub github_endpoint: String,
    pub webhook_endpoint: String,
    pub github_client: Client,
}

#[derive(Debug, Serialize)]
pub struct Runner {
    pub ping_time: String,
    pub online: bool,
    pub runner_set: String,
    pub id: i64,
    pub name: String,
    pub os: String,
    pub labels: Vec<String>,
}

pub enum RunnerStateChange {
    // a new runner just popped up
    Created,
    Removed,
    Offline,
    Online,
}

pub type RunnerMap = HashMap<String, Runner>;
