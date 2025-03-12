use std::collections::hash_set::HashSet;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;

use crate::inbound_alert_handler::InboundAlertHandler;
use crate::{
    alert::alert_all_changes_and_update_grace_period,
    github::key_runner,
    structs::{Config, Runner, RunnerMap},
};

// these tests use the utc_ping_time field in the Runner struct to differentiate different
// simulated pings and check their order
//
// A simple vector can't be used because the order in which the alerts are sent for a single ping is
// undefined.

#[tokio::test]
async fn alert_grace_0_test() {
    let grace_period = 0;
    let inbound_address_tcp = String::from("127.0.0.1:9000");
    let inbound_address_http = format!("http://{}", inbound_address_tcp);
    let states: Vec<RunnerMap> = vec![
        // initial setup
        RunnerMap::from([key_runner(Runner {
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
            webhook_endpoint: inbound_address_http.clone(),
        })]),
        // runner went offline
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
            webhook_endpoint: inbound_address_http.clone(),
        })]),
        // runner removed
        RunnerMap::from([]),
        // runner added
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
                webhook_endpoint: inbound_address_http.clone(),
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
                webhook_endpoint: inbound_address_http.clone(),
            }),
        ]),
        // nothing changed
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
                webhook_endpoint: inbound_address_http.clone(),
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
                // label changes don't cause an alert
                labels: vec![String::from("some other label")],
                webhook_endpoint: inbound_address_http.clone(),
            }),
        ]),
        // one runner went offline
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
                webhook_endpoint: inbound_address_http.clone(),
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
                webhook_endpoint: inbound_address_http.clone(),
            }),
        ]),
        // other runner came online
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
                webhook_endpoint: inbound_address_http.clone(),
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
                webhook_endpoint: inbound_address_http.clone(),
            }),
        ]),
    ];
    let expected_final_state = RunnerMap::from([
        key_runner(Runner {
            utc_ping_time: String::from("6"),
            interpret_online: Some(false),
            online_for_github_api: false,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisOtherTestOrg; github: https://github.com/api/v3"),
            id: 420,
            name: String::from("runner-01"),
            os: String::from("linux"),
            labels: vec![String::from("some label")],
            webhook_endpoint: inbound_address_http.clone(),
        }),
        key_runner(Runner {
            utc_ping_time: String::from("6"),
            interpret_online: Some(true),
            online_for_github_api: true,
            online_state_change_since: 0,
            runner_set: String::from("org: chrisOtherTestOrg; github: https://github.com/api/v3"),
            id: 12,
            name: String::from("runner-01"),
            os: String::from("linux"),
            labels: vec![String::from("some label")],
            webhook_endpoint: inbound_address_http.clone(),
        }),
    ]);
    let awaited_messages = HashSet::from([
        r#"[{"summary":"Runner went Offline","event_body":"Old Runner:\n{\n  \"utc_ping_time\": \"1\",\n  \"interpret_online\": true,\n  \"online_for_github_api\": true,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisTestOrg; github: https://github.com/api/v3\",\n  \"id\": 69,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}\n\nNew Runner:\n{\n  \"utc_ping_time\": \"2\",\n  \"interpret_online\": false,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisTestOrg; github: https://github.com/api/v3\",\n  \"id\": 69,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
        r#"[{"summary":"Removed Runner","event_body":"Now removed Runner:\n{\n  \"utc_ping_time\": \"2\",\n  \"interpret_online\": false,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisTestOrg; github: https://github.com/api/v3\",\n  \"id\": 69,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
        r#"[{"summary":"Created new Runner","event_body":"Now created Runner:\n{\n  \"utc_ping_time\": \"3\",\n  \"interpret_online\": false,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisOtherTestOrg; github: https://github.com/api/v3\",\n  \"id\": 12,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
        r#"[{"summary":"Created new Runner","event_body":"Now created Runner:\n{\n  \"utc_ping_time\": \"3\",\n  \"interpret_online\": true,\n  \"online_for_github_api\": true,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisOtherTestOrg; github: https://github.com/api/v3\",\n  \"id\": 420,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
        r#"[{"summary":"Runner went Offline","event_body":"Old Runner:\n{\n  \"utc_ping_time\": \"4\",\n  \"interpret_online\": true,\n  \"online_for_github_api\": true,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisOtherTestOrg; github: https://github.com/api/v3\",\n  \"id\": 420,\n  \"name\": \"runner-01\",\n  \"os\": \"TempleOS\",\n  \"labels\": [\n    \"some other label\"\n  ]\n}\n\nNew Runner:\n{\n  \"utc_ping_time\": \"5\",\n  \"interpret_online\": false,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisOtherTestOrg; github: https://github.com/api/v3\",\n  \"id\": 420,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
        r#"[{"summary":"Runner came Online","event_body":"Old Runner:\n{\n  \"utc_ping_time\": \"5\",\n  \"interpret_online\": false,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisOtherTestOrg; github: https://github.com/api/v3\",\n  \"id\": 12,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}\n\nNew Runner:\n{\n  \"utc_ping_time\": \"6\",\n  \"interpret_online\": true,\n  \"online_for_github_api\": true,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisOtherTestOrg; github: https://github.com/api/v3\",\n  \"id\": 12,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
    ]);

    tokio::join!(
        alert_test_uptime(states, grace_period, expected_final_state),
        inbound_mock(&inbound_address_tcp, awaited_messages)
    );
}

// #[tokio::test]
// async fn alert_grace_3_test() {
//     let grace_period = 3;
//     let inbound_address_tcp = String::from("127.0.0.1:9001");
//     let inbound_address_http = format!("http://{}", inbound_address_tcp);
//     let states: Vec<RunnerMap> = vec![
//         // initial setup
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("1"),
//             // the initial interpret_online needs to be set
//             interpret_online: Some(true),
//             online_for_github_api: true,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         // runner went offline for GitHub
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("2"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("3"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("4"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         // runner back online for GitHub
//         // none of this causes an alert
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("5"),
//             interpret_online: None,
//             online_for_github_api: true,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         // don't do anything
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("6"),
//             interpret_online: None,
//             online_for_github_api: true,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         // runner goes ofline for GitHub
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("7"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("8"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("9"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//         // now an alert will be sent
//         RunnerMap::from([key_runner(Runner {
//             utc_ping_time: String::from("10"),
//             interpret_online: None,
//             online_for_github_api: false,
//             online_state_change_since: 0,
//             runner_set: String::from("org: chrisTestOrg; github: https://github.com/api/v3"),
//             id: 69,
//             name: String::from("runner-01"),
//             os: String::from("linux"),
//             labels: vec![String::from("some label")],
//             webhook_endpoint: inbound_address_http.clone(),
//         })]),
//     ];
//     let expected_final_state = RunnerMap::from([]);
//     let awaited_messages = HashSet::from([
//         r#"[{"summary":"Runner went Offline","event_body":"Old Runner:\n{\n  \"utc_ping_time\": \"9\",\n  \"interpret_online\": true,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 3,\n  \"runner_set\": \"org: chrisTestOrg; github: https://github.com/api/v3\",\n  \"id\": 69,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}\n\nNew Runner:\n{\n  \"utc_ping_time\": \"10\",\n  \"interpret_online\": false,\n  \"online_for_github_api\": false,\n  \"online_state_change_since\": 0,\n  \"runner_set\": \"org: chrisTestOrg; github: https://github.com/api/v3\",\n  \"id\": 69,\n  \"name\": \"runner-01\",\n  \"os\": \"linux\",\n  \"labels\": [\n    \"some label\"\n  ]\n}","type":"github_monitor"}]"#,
//     ]);

//     tokio::join!(
//         alert_test_uptime(states, grace_period, expected_final_state),
//         inbound_mock(&inbound_address_tcp, awaited_messages)
//     );
// }

// takes the RunnerMaps as get_all_runners would produce them
async fn alert_test_uptime(
    states: Vec<RunnerMap>,
    grace_period: u8,
    expected_final_state: RunnerMap,
) {
    // wait for server
    sleep(Duration::from_millis(100)).await;
    let mut states = states.into_iter();
    // only used for send_alert
    let cfg = Config {
        runner_sets: vec![],
        github_timeout: Duration::from_millis(30),
        inbound_timeout: Duration::from_millis(30),
        grace_period,
        allow_http: true,
    };

    let mut runners = states.next().unwrap();
    for mut new_runners in states {
        let alert_handler = InboundAlertHandler::new();
        alert_all_changes_and_update_grace_period(&cfg, &runners, &mut new_runners, &alert_handler)
            .await
            .unwrap();
        runners = new_runners;
    }
    assert_eq!(runners, expected_final_state);
}

fn get_expected_payload(msg: &str) -> String {
    format!(
        "POST / HTTP/1.1\r
accept: */*\r
user-agent: gh_runner_uptime v{}\r
host: 127.0.0.1:9000\r
content-length: {}\r
\r
{}",
        env!("CARGO_PKG_VERSION"),
        msg.chars().count(),
        msg
    )
}

async fn inbound_mock(address: &str, expected_messages: HashSet<&str>) {
    let listener = TcpListener::bind(address).await.unwrap();
    let mut expected_payloads: HashSet<String> = expected_messages
        .into_iter()
        .map(get_expected_payload)
        .collect();

    println!("starting mock inbound server");
    while !expected_payloads.is_empty() {
        let mut stream = listener.accept().await.unwrap().0;
        let mut buf: Vec<u8> = vec![0; 4000];
        stream.read(&mut buf).await.unwrap();
        let payload = String::from_utf8(buf).unwrap();
        let payload = payload.trim_matches(char::from(0));
        println!(
            "\nreceived payload:\n> {:#?}\n",
            payload.replace("\n", "\n> ")
        );
        assert!(expected_payloads.remove(payload));

        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write(response.as_bytes()).await.unwrap();
    }
    println!("stopping mock inbound server");
}
