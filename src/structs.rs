use reqwest::Client;
use serde::Serialize;
use std::{collections::HashMap, time::Duration};

#[derive(Debug)]
pub struct Config {
    pub runner_sets: Vec<RunnerSetConfig>,
    pub github_timeout: Duration,
    pub inbound_timeout: Duration,
    // The grace period does not affect the created and removed state changes.
    // It only allows runners to briefly go offline and come back
    // without raising the alarm.
    //
    // A grace period of 0 means the online state from the GitHub API is
    // used directly for events.
    pub grace_period: u8,
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
#[derive(Debug, Serialize, PartialEq)]
pub struct Runner {
    pub utc_ping_time: String,
    // the state reflected in the inbound webhook events
    // only once the grace period has passed does this change
    pub interpret_online: Option<bool>,
    // the state the GitHub API provides
    // it does not include the grace period
    pub online_for_github_api: bool,
    // for how long is interpret_online != online_for_github_api
    pub online_state_change_since: u8,
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
