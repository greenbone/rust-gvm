// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

//! Socket-level decoding of bounded built-in inventory projections.
#![cfg(feature = "unix-socket-tests")]
#![allow(missing_docs, clippy::unwrap_used)]

use gvm_client::{GmpClient, GvmError};
use gvm_connection::{GvmConnection, UnixSocketConnection};
use gvm_gmp::commands::{
    agents::{GetAgentSupportBundleRequest, GetAgentsRequest},
    authentication::AuthenticateRequest,
    reports::GetReportApplicationsRequest,
    results::GetResultsRequest,
    scanners::GetScannersRequest,
};
use gvm_gmp::{responses::GetAgentsResponse, AgentSchedule, EntityId, GmpVersion};
use gvm_mock_server::{GmpVersion as MockVersion, MockGmpServer, Resource, ServerMode};
use gvm_protocol::Response;

fn connection(server: &MockGmpServer) -> UnixSocketConnection {
    UnixSocketConnection::with_path(server.socket_path().unwrap())
}

async fn fixture_server(version: MockVersion) -> MockGmpServer {
    MockGmpServer::builder()
        .mode(ServerMode::Fixture)
        .version(version)
        .unix_socket_auto()
        .build()
        .await
        .unwrap()
}

#[tokio::test]
async fn builtins_decode_os_metadata_and_structured_scheduler_defaults() {
    let server = fixture_server(MockVersion::V22_8).await;
    let mut client = GmpClient::connect(connection(&server)).await.unwrap();
    let agents = client.execute(GetAgentsRequest::default()).await.unwrap();
    assert_eq!(agents.items.len(), 3);
    assert_eq!(agents.counts.total, Some(3));
    assert_eq!(agents.counts.filtered, Some(3));
    let agent = &agents.items[0];
    assert_eq!(
        agent.operating_system.as_deref(),
        Some("Debian GNU/Linux 12")
    );
    assert_eq!(agent.architecture.as_deref(), Some("x86_64"));
    assert_eq!(
        agent.os_cpe.as_deref(),
        Some("cpe:/o:debian:debian_linux:12")
    );
    assert_eq!(agent.os_status.as_deref(), Some("supported"));
    for legacy in &agents.items[1..] {
        assert_eq!(legacy.operating_system, None);
        assert_eq!(legacy.architecture, None);
        assert_eq!(legacy.os_cpe, None);
        assert_eq!(legacy.os_status, None);
        assert_eq!(legacy.config, None);
    }
    let config = agent.config.as_ref().unwrap();
    let entries = &config
        .agent_script_executor
        .as_ref()
        .unwrap()
        .scheduler_cron_time;
    assert_eq!(
        entries,
        &vec![
            AgentSchedule {
                cron: "0 */6 * * *".into(),
                max_delay_seconds: Some(30)
            },
            AgentSchedule {
                cron: "15 * * * *".into(),
                max_delay_seconds: Some(0)
            },
            "legacy cron".into(),
            "".into(),
        ]
    );
    let scanners = client.execute(GetScannersRequest::default()).await.unwrap();
    assert_eq!(scanners.items.len(), 2);
    assert_eq!(scanners.items[0].agent_control_config_defaults, None);
    assert_eq!(scanners.items[1].scanner_type.as_deref(), Some("9"));
    let defaults = scanners.items[1]
        .agent_control_config_defaults
        .as_ref()
        .unwrap();
    assert_eq!(defaults.agent_defaults.as_ref(), Some(config));
    assert_eq!(defaults.update_to_latest, Some(true));
    server.shutdown().await;
}

#[tokio::test]
async fn builtin_report_applications_preserve_missing_ids_and_independent_counts() {
    let server = fixture_server(MockVersion::V22_8).await;
    let mut client = GmpClient::connect(connection(&server)).await.unwrap();
    let applications = client
        .execute(GetReportApplicationsRequest::new(
            EntityId::new("report-fixture").unwrap(),
        ))
        .await
        .unwrap();
    assert_eq!(applications.items.len(), 4);
    assert_eq!(applications.counts.total, Some(4));
    assert_eq!(applications.counts.filtered, Some(4));
    assert!(applications.items.iter().all(|item| item.id.is_none()));
    let first = &applications.items[0];
    assert_eq!(first.name.as_deref(), Some("OpenSSH"));
    assert_eq!(first.hosts_count, Some(2));
    assert_eq!(first.occurrences, Some(5));
    assert_eq!(first.severity.as_deref(), Some("9.8"));
    assert_eq!(first.threat.as_deref(), Some("High"));
    for item in &applications.items[1..3] {
        assert_eq!(item.hosts_count, None);
        assert_eq!(item.occurrences, None);
        assert_eq!(item.threat, None);
    }
    assert_eq!(applications.items[3].hosts_count, Some(0));
    assert_eq!(applications.items[3].occurrences, Some(0));
    assert_eq!(applications.items[3].threat.as_deref(), Some("Log"));
    server.shutdown().await;
}

#[tokio::test]
async fn agent_fixture_does_not_bypass_client_version_gate_but_raw_fixture_policy_is_unchanged() {
    let server = fixture_server(MockVersion::V22_7).await;
    let mut client = GmpClient::connect(connection(&server)).await.unwrap();
    server.clear_history();
    assert!(matches!(client.execute(GetAgentsRequest::default()).await,
        Err(GvmError::UnsupportedCommand { command, version: GmpVersion(22, 7), required: "22.8" })
            if command == "get_agents"));
    assert!(server.command_history().is_empty());
    let scanners = client.execute(GetScannersRequest::default()).await.unwrap();
    assert_eq!(scanners.items.len(), 1);
    assert_eq!(scanners.items[0].agent_control_config_defaults, None);

    // Fixture mode is programmable playback, not stateful version enforcement.
    // A direct socket request intentionally bypasses the client's command gate.
    let mut raw = connection(&server);
    raw.connect().await.unwrap();
    raw.send(b"<get_agents/>").await.unwrap();
    let agents =
        GetAgentsResponse::from_response(&Response::from(raw.read().await.unwrap())).unwrap();
    assert_eq!(agents.items.len(), 3);
    assert_eq!(agents.status, 200);
    server.shutdown().await;
}

#[tokio::test]
async fn support_bundle_options_are_recorded_without_claiming_dynamic_encryption_or_anonymization()
{
    let server = MockGmpServer::builder()
        .mode(ServerMode::Stateful)
        .version(MockVersion::V22_8)
        .credentials("admin", "admin")
        .unix_socket_auto()
        .build()
        .await
        .unwrap();
    let mut client = GmpClient::connect(connection(&server)).await.unwrap();
    client
        .execute(AuthenticateRequest::new("admin", "admin"))
        .await
        .unwrap();
    server.clear_history();
    for option in [None, Some(false), Some(true)] {
        let mut request =
            GetAgentSupportBundleRequest::new(EntityId::new("agent-fixture").unwrap(), Some(7));
        request.encryption = option;
        request.anonymized = option;
        let bundle = client.execute(request).await.unwrap();
        // Fixed inert bytes: these options neither encrypt nor transform an archive.
        assert_eq!(bundle.file.content, b"hello-mock");
        assert_eq!(bundle.file.name, "mock-agent-support-bundle.tar.gz");
        assert_eq!(
            bundle.file.content_type.as_deref(),
            Some("application/octet-stream")
        );
    }
    let history = server.command_history();
    assert_eq!(history.len(), 3);
    for (entry, expected) in history.iter().zip([
        r#"<get_agent_support_bundle agent_uuid="agent-fixture" days="7"/>"#,
        r#"<get_agent_support_bundle agent_uuid="agent-fixture" anonymized="0" days="7" encryption="0"/>"#,
        r#"<get_agent_support_bundle agent_uuid="agent-fixture" anonymized="1" days="7" encryption="1"/>"#,
    ]) {
        assert_eq!(entry.raw_xml(), expected.as_bytes());
    }
    server.shutdown().await;
}

#[tokio::test]
async fn seeded_results_preserve_unknown_nvt_type_and_opaque_metadata() {
    const METADATA: &str = r#"{"future": ["<&>", 42], "opaque": true}"#;
    let server = MockGmpServer::builder()
        .mode(ServerMode::Stateful)
        .version(MockVersion::V22_8)
        .credentials("admin", "admin")
        .unix_socket_auto()
        .seed(|store| {
            let mut result = Resource::new("result", "Future typed finding");
            result.set_attr("nvt_oid", "1.3.6.1.4.1.25623.1.0.12345");
            result.set_attr("nvt_type", "future-vt-kind");
            result.set_attr("type_metadata", METADATA);
            store.seed(result);
        })
        .build()
        .await
        .unwrap();
    let mut client = GmpClient::connect(connection(&server)).await.unwrap();
    client
        .execute(AuthenticateRequest::new("admin", "admin"))
        .await
        .unwrap();
    let results = client.execute(GetResultsRequest::default()).await.unwrap();
    assert_eq!(results.items.len(), 1);
    let nvt = results.items[0].nvt.as_ref().unwrap();
    assert_eq!(nvt.type_.as_deref(), Some("future-vt-kind"));
    assert_eq!(nvt.type_metadata.as_deref(), Some(METADATA));
    server.shutdown().await;
}
