use std::collections::hash_set::HashSet;
use std::time::Duration;

use crate::structs::RunnerStateChange;
use crate::test_alert_handler::TestAlertHandler;
use crate::{
    alert::alert_all_changes_and_update_grace_period,
    github::key_runner,
    structs::{Config, Runner, RunnerMap},
};

// test the tests
#[tokio::test]
#[should_panic]
async fn alert_grace_0_should_panic_test() {
    let grace_period = 0;
    let cfg = Config {
        runner_sets: vec![],
        github_timeout: Duration::from_millis(30),
        inbound_timeout: Duration::from_millis(30),
        grace_period,
        allow_http: false,
    };

    // initial setup
    let mut runners = RunnerMap::from([key_runner(Runner {
        utc_ping_time: String::from("1"),
        // the initial interpret_online needs to be set
        interpret_online: Some(true),
        online_for_github_api: true,
        online_state_change_since: 0,
        runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
        id: 69,
        name: String::from("runner-01"),
        os: String::from("linux"),
        labels: vec![String::from("some label")],
        webhook_endpoint: String::from("https://chris-besch.com"),
    })]);

    // runner went offline
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("2"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 69,
            name: String::from("runner-01"),
            os: String::from("linux"),
            labels: vec![String::from("some label")],
            webhook_endpoint: String::from("https://chris-besch.com"),
        })]),
        HashSet::from([RunnerStateChange::Offline(
            Runner {
                utc_ping_time: String::from("1"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 69,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
            Runner {
                utc_ping_time: String::from("2"),
                interpret_online: Some(false),
                // this should be false and thus causes a panic
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 69,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
        )]),
    )
    .await;
}

#[tokio::test]
async fn alert_grace_0_test() {
    let grace_period = 0;
    let cfg = Config {
        runner_sets: vec![],
        github_timeout: Duration::from_millis(30),
        inbound_timeout: Duration::from_millis(30),
        grace_period,
        allow_http: false,
    };

    // initial setup
    let mut runners = RunnerMap::from([key_runner(Runner {
        utc_ping_time: String::from("1"),
        // the initial interpret_online needs to be set
        interpret_online: Some(true),
        online_for_github_api: true,
        online_state_change_since: 0,
        runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
        id: 69,
        name: String::from("runner-01"),
        os: String::from("linux"),
        labels: vec![String::from("some label")],
        webhook_endpoint: String::from("https://chris-besch.com"),
    })]);

    // runner went offline
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("2"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 69,
            name: String::from("runner-01"),
            os: String::from("linux"),
            labels: vec![String::from("some label")],
            webhook_endpoint: String::from("https://chris-besch.com"),
        })]),
        HashSet::from([RunnerStateChange::Offline(
            Runner {
                utc_ping_time: String::from("1"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 69,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
            Runner {
                utc_ping_time: String::from("2"),
                interpret_online: Some(false),
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 69,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
        )]),
    )
    .await;

    // runner removed
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([]),
        HashSet::from([RunnerStateChange::Removed(Runner {
            utc_ping_time: String::from("2"),
            interpret_online: Some(false),
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 69,
            name: String::from("runner-01"),
            os: String::from("linux"),
            labels: vec![String::from("some label")],
            webhook_endpoint: String::from("https://chris-besch.com"),
        })]),
    )
    .await;
    // this doesn't change anything
    perform_alert_test_step(&cfg, &mut runners, RunnerMap::from([]), HashSet::from([])).await;
    perform_alert_test_step(&cfg, &mut runners, RunnerMap::from([]), HashSet::from([])).await;
    perform_alert_test_step(&cfg, &mut runners, RunnerMap::from([]), HashSet::from([])).await;
    perform_alert_test_step(&cfg, &mut runners, RunnerMap::from([]), HashSet::from([])).await;
    perform_alert_test_step(&cfg, &mut runners, RunnerMap::from([]), HashSet::from([])).await;

    // runner added
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([
            key_runner(Runner {
                utc_ping_time: String::from("3"),
                interpret_online: None,
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
            key_runner(Runner {
                utc_ping_time: String::from("3"),
                interpret_online: None,
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
        ]),
        HashSet::from([
            RunnerStateChange::Created(Runner {
                utc_ping_time: String::from("3"),
                interpret_online: Some(false),
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
            RunnerStateChange::Created(Runner {
                utc_ping_time: String::from("3"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
        ]),
    )
    .await;

    // nothing changed
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([
            key_runner(Runner {
                utc_ping_time: String::from("4"),
                interpret_online: None,
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                // os changes don't cause an alert
                os: String::from("TempleOS"),
                // label changes don't cause an alert
                labels: vec![String::from("some other label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
            key_runner(Runner {
                utc_ping_time: String::from("4"),
                interpret_online: None,
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
        ]),
        HashSet::from([]),
    )
    .await;

    // one runner went offline
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([
            key_runner(Runner {
                utc_ping_time: String::from("5"),
                interpret_online: None,
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
            key_runner(Runner {
                utc_ping_time: String::from("5"),
                interpret_online: None,
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
        ]),
        HashSet::from([RunnerStateChange::Offline(
            Runner {
                utc_ping_time: String::from("4"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                // os changes don't cause an alert
                os: String::from("TempleOS"),
                // label changes don't cause an alert
                labels: vec![String::from("some other label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
            Runner {
                utc_ping_time: String::from("5"),
                interpret_online: Some(false),
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
        )]),
    )
    .await;

    // other runner came online
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([
            key_runner(Runner {
                utc_ping_time: String::from("6"),
                interpret_online: None,
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 420,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
            key_runner(Runner {
                utc_ping_time: String::from("6"),
                interpret_online: None,
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
        ]),
        HashSet::from([RunnerStateChange::Online(
            Runner {
                utc_ping_time: String::from("5"),
                interpret_online: Some(false),
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
            Runner {
                utc_ping_time: String::from("6"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3",
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            },
        )]),
    )
    .await;

    assert_eq!(
        runners,
        RunnerMap::from([
            key_runner(Runner {
                utc_ping_time: String::from("6"),
                interpret_online: Some(false),
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3"
                ),
                id: 420,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
            key_runner(Runner {
                utc_ping_time: String::from("6"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from(
                    "org: chrisOtherTestOrg; github: https://github.com/api/v3"
                ),
                id: 12,
                name: String::from("runner-01"),
                os: String::from("linux"),
                labels: vec![String::from("some label")],
                webhook_endpoint: String::from("https://chris-besch.com"),
            }),
        ])
    )
}

#[tokio::test]
async fn alert_grace_3_test() {
    let grace_period = 3;
    let cfg = Config {
        runner_sets: vec![],
        github_timeout: Duration::from_millis(30),
        inbound_timeout: Duration::from_millis(30),
        grace_period,
        allow_http: false,
    };

    // initial setup
    let mut runners = RunnerMap::from([key_runner(Runner {
        utc_ping_time: String::from("1"),
        // the initial interpret_online needs to be set
        interpret_online: Some(true),
        online_for_github_api: true,
        online_state_change_since: 0,
        runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
        id: 2845,
        name: String::from("runner-69"),
        os: String::from("todd-linux"),
        labels: vec![String::from("16x the detail")],
        webhook_endpoint: String::from("https://example.com"),
    })]);

    // runner goes offline for GitHub for three heartbeats
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("2"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("3"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("4"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;

    // runner goes back online for GitHub
    // for now nothing has changed yet as the grace period wasn't completetd
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("5"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;

    // runner goes offline for GitHub for four heartbeats
    // this causes an alert
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("6"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("7"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("8"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("9"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([RunnerStateChange::Offline(
            Runner {
                utc_ping_time: String::from("8"),
                interpret_online: Some(true),
                online_for_github_api: false,
                online_state_change_since: 3,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 2845,
                name: String::from("runner-69"),
                os: String::from("todd-linux"),
                labels: vec![String::from("16x the detail")],
                webhook_endpoint: String::from("https://example.com"),
            },
            Runner {
                utc_ping_time: String::from("9"),
                interpret_online: Some(false),
                online_for_github_api: false,
                online_state_change_since: 0,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 2845,
                name: String::from("runner-69"),
                os: String::from("todd-linux"),
                labels: vec![String::from("16x the detail")],
                webhook_endpoint: String::from("https://example.com"),
            },
        )]),
    )
    .await;

    // nothing changes here
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("10"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;

    // go back online for GitHub for three heartbeats
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("11"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("12"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("13"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;

    // go back offline
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("14"),
            interpret_online: None,
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;

    // go back online for GitHub for four heartbeats
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("15"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("16"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("17"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([]),
    )
    .await;
    perform_alert_test_step(
        &cfg,
        &mut runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("18"),
            interpret_online: None,
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })]),
        HashSet::from([RunnerStateChange::Online(
            Runner {
                utc_ping_time: String::from("17"),
                interpret_online: Some(false),
                online_for_github_api: true,
                online_state_change_since: 3,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 2845,
                name: String::from("runner-69"),
                os: String::from("todd-linux"),
                labels: vec![String::from("16x the detail")],
                webhook_endpoint: String::from("https://example.com"),
            },
            Runner {
                utc_ping_time: String::from("18"),
                interpret_online: Some(true),
                online_for_github_api: true,
                online_state_change_since: 0,
                runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
                id: 2845,
                name: String::from("runner-69"),
                os: String::from("todd-linux"),
                labels: vec![String::from("16x the detail")],
                webhook_endpoint: String::from("https://example.com"),
            },
        )]),
    )
    .await;

    assert_eq!(
        runners,
        RunnerMap::from([key_runner(Runner {
            utc_ping_time: String::from("18"),
            interpret_online: Some(true),
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
            id: 2845,
            name: String::from("runner-69"),
            os: String::from("todd-linux"),
            labels: vec![String::from("16x the detail")],
            webhook_endpoint: String::from("https://example.com"),
        })])
    );
}

async fn perform_alert_test_step(
    cfg: &Config,
    runners: &mut RunnerMap,
    mut new_runners: RunnerMap,
    expected_changes: HashSet<RunnerStateChange>,
) {
    let mut alert_handler = TestAlertHandler::new(expected_changes);
    alert_all_changes_and_update_grace_period(&cfg, &runners, &mut new_runners, &mut alert_handler)
        .await
        .unwrap();
    alert_handler.assert_all_received();
    *runners = new_runners;
}
