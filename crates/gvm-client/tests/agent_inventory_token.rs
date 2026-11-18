// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

#![allow(clippy::unwrap_used, missing_docs)]
#![cfg(feature = "unix-socket-tests")]

use std::sync::{Arc, Mutex};

use gvm_client::{CommandSupport, GmpClient, GvmError, WireTraceEvent};
use gvm_connection::UnixSocketConnection;
use gvm_gmp::commands::agents::GetAgentInventoryTokenRequest;
use gvm_mock_server::{GmpVersion, MockGmpServer, ServerMode};

const COMMAND: &str = "get_agent_inventory_token";
const HELP: &str = r#"<help_response status="200" status_text="OK"><schema format="XML"><command><name>get_agent_inventory_token</name></command></schema></help_response>"#;

async fn server(response: Option<&str>, help: &str, version: GmpVersion) -> MockGmpServer {
    let mut builder = MockGmpServer::builder()
        .mode(ServerMode::Fixture)
        .version(version)
        .unix_socket_auto()
        .override_response("help", help);
    if let Some(response) = response {
        builder = builder.override_response(COMMAND, response);
    }
    builder.build().await.expect("mock server")
}

#[tokio::test]
async fn inventory_token_is_typed_and_redacted_in_debug_and_wire_trace() {
    let server = server(None, HELP, GmpVersion::V22_8).await;
    let events = Arc::new(Mutex::new(Vec::<WireTraceEvent>::new()));
    let sink = Arc::clone(&events);
    let mut client = GmpClient::connect_with_wire_trace(
        UnixSocketConnection::with_path(server.socket_path().unwrap()),
        move |event| sink.lock().unwrap().push(event),
    )
    .await
    .unwrap();
    assert_eq!(
        client.command_support(COMMAND),
        CommandSupport::RequiresDiscovery
    );
    assert!(client
        .get_agent_inventory_token(GetAgentInventoryTokenRequest)
        .await
        .is_err());
    assert!(!server
        .command_history()
        .iter()
        .any(|entry| entry.command_name() == COMMAND));
    client.discover_commands().await.unwrap();
    let response = client
        .get_agent_inventory_token(GetAgentInventoryTokenRequest)
        .await
        .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.token.as_str(), "mock-agent-inventory-token");
    assert!(!format!("{response:?}").contains(response.token.as_str()));
    assert!(!format!("{:?}", response.token).contains(response.token.as_str()));
    {
        let traces = events.lock().unwrap();
        assert!(traces
            .iter()
            .any(|event| String::from_utf8_lossy(&event.bytes)
                .contains("<token><redacted/></token>")));
        assert!(traces
            .iter()
            .all(|event| !String::from_utf8_lossy(&event.bytes).contains(response.token.as_str())));
    }
    server.shutdown().await;
}

#[tokio::test]
async fn inventory_token_preserves_disabled_unavailable_denied_and_server_errors() {
    for (status, message) in [
        (503, "Command unavailable"),
        (503, "Agent inventory token issuance is unavailable"),
        (400, "Permission denied"),
        (500, "Internal error"),
    ] {
        let xml = format!(
            "<get_agent_inventory_token_response status=\"{status}\" status_text=\"{message}\"/>"
        );
        let server = server(Some(&xml), HELP, GmpVersion::V22_8).await;
        let mut client = GmpClient::connect(UnixSocketConnection::with_path(
            server.socket_path().unwrap(),
        ))
        .await
        .unwrap();
        client.discover_commands().await.unwrap();
        let error = client
            .get_agent_inventory_token(GetAgentInventoryTokenRequest)
            .await
            .unwrap_err();
        assert!(
            matches!(error, GvmError::Server { status: actual, message: ref text } if actual == status && text == message)
        );
        server.shutdown().await;
    }
}

#[tokio::test]
async fn inventory_token_rejects_missing_payload_and_negative_discovery() {
    let server = server(
        Some(r#"<get_agent_inventory_token_response status="200" status_text="OK"/>"#),
        HELP,
        GmpVersion::V22_8,
    )
    .await;
    let mut client = GmpClient::connect(UnixSocketConnection::with_path(
        server.socket_path().unwrap(),
    ))
    .await
    .unwrap();
    client.discover_commands().await.unwrap();
    assert!(client
        .get_agent_inventory_token(GetAgentInventoryTokenRequest)
        .await
        .is_err());
    server.shutdown().await;

    for version in [GmpVersion::V22_7, GmpVersion::V22_8] {
        let server = self::server(None, r#"<help_response status="200" status_text="OK"><schema format="XML"/></help_response>"#, version).await;
        let mut client = GmpClient::connect(UnixSocketConnection::with_path(
            server.socket_path().unwrap(),
        ))
        .await
        .unwrap();
        client.discover_commands().await.unwrap();
        assert!(client
            .get_agent_inventory_token(GetAgentInventoryTokenRequest)
            .await
            .is_err());
        server.shutdown().await;
    }
}

#[tokio::test]
async fn inventory_token_authenticated_stateful_fixture_has_canonical_wire_shape() {
    let server = MockGmpServer::builder()
        .mode(ServerMode::Stateful)
        .version(GmpVersion::V22_8)
        .credentials("admin", "admin")
        .unix_socket_auto()
        .build()
        .await
        .unwrap();
    let mut client = GmpClient::connect(UnixSocketConnection::with_path(
        server.socket_path().unwrap(),
    ))
    .await
    .unwrap();
    client
        .authenticate(gvm_gmp::commands::authentication::AuthenticateRequest::new(
            "admin", "admin",
        ))
        .await
        .unwrap();
    client.discover_commands().await.unwrap();
    let response = client
        .get_agent_inventory_token(GetAgentInventoryTokenRequest)
        .await
        .unwrap();
    assert_eq!(response.token.as_str(), "mock-agent-inventory-token");
    let requests = server.command_history();
    let request = requests
        .iter()
        .find(|entry| entry.command_name() == COMMAND)
        .unwrap();
    assert_eq!(request.raw_xml(), b"<get_agent_inventory_token/>");
    server.shutdown().await;
}
