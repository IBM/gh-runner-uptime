use reqwest::Client;
use serde::Serialize;
use std::{collections::HashMap, time::Duration};

#[derive(Debug)]
pub struct Config {
    pub runner_sets: Vec<RunnerSetConfig>,
    // this is used before constructing the config
    #[allow(dead_code)]
    pub github_timeout: Duration,
    pub inbound_timeout: Duration,

    // templates for inbound request
    pub created_template_path: String,
    pub removed_template_path: String,
    pub online_template_path: String,
    pub offline_template_path: String,

    // The grace period does not affect the created and removed state changes.
    // It only allows runners to briefly go offline and come back
    // without raising the alarm.
    //
    // A grace period of 0 means the online state from the GitHub API is
    // used directly for events.
    pub grace_period: u32,
    // used for testing
    pub allow_http: bool,
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
#[derive(Debug, Serialize, PartialEq, Clone, Hash, Eq)]
pub struct Runner {
    pub utc_ping_time: String,
    // the state reflected in the inbound webhook events
    // only once the grace period has passed does this change
    pub interpret_online: Option<bool>,
    // the state the GitHub API provides
    // it does not include the grace period
    pub online_for_github_api: bool,
    // for how long is interpret_online != online_for_github_api
    pub online_state_change_since: u32,
    pub runner_set: String,
    pub id: i64,
    pub name: String,
    pub os: String,
    pub labels: Vec<String>,

    // this contains a secret key
    #[serde(skip_serializing)]
    pub webhook_endpoint: String,
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub enum RunnerStateChange {
    // a new runner just popped up
    Created(Runner),
    // a known runner isn't there anymore
    Removed(Runner),
    // a runner was online the last time and is now offline
    // old runner, new runner
    Offline(Runner, Runner),
    // a runner was offline the last time and is now online
    // old runner, new runner
    Online(Runner, Runner),
}

pub type RunnerMap = HashMap<String, Runner>;
