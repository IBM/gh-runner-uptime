use anyhow::Result;
use std::collections::hash_set::HashSet;

use crate::alert::AlertHandler;
use crate::structs::{Config, RunnerStateChange};

pub struct TestAlertHandler {
    expect_changes: HashSet<RunnerStateChange>,
}

impl TestAlertHandler {
    pub fn new(expect_changes: HashSet<RunnerStateChange>) -> Self {
        TestAlertHandler { expect_changes }
    }
    pub fn assert_all_received(&self) {
        assert!(self.expect_changes.is_empty());
    }
}

impl AlertHandler for TestAlertHandler {
    async fn send_alert(&mut self, _cfg: &Config, change: RunnerStateChange) -> Result<()> {
        if !self.expect_changes.remove(&change) {
            panic!(
                "expected one of:\n{:?}\ngot:\n{:?}",
                self.expect_changes, change
            );
        }
        Ok(())
    }
}
