// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG
//! Target provenance create/modify/copy/filter regression coverage.
#![cfg(feature = "unix-socket-tests")]
#![allow(missing_docs)]

use gvm_gmp::{GmpResponse, GmpVersion as WireVersion};
use gvm_mock_server::{MockGmpServer, ServerMode};
use gvm_protocol::Response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

async fn exchange(stream: &mut UnixStream, xml: &str) -> Response {
    stream
        .write_all(xml.as_bytes())
        .await
        .expect("write request");
    let mut bytes = Vec::new();
    let root = xml
        .trim_start_matches('<')
        .split([' ', '/', '>'])
        .next()
        .expect("command");
    let closing = format!("</{root}_response>");
    loop {
        let mut buf = [0u8; 4096];
        let count = stream.read(&mut buf).await.expect("read response");
        assert!(count > 0, "unexpected EOF");
        bytes.extend_from_slice(&buf[..count]);
        if (bytes.ends_with(b"/>")
            && bytes.iter().position(|byte| *byte == b'>') == Some(bytes.len() - 1))
            || bytes.ends_with(closing.as_bytes())
        {
            break;
        }
    }
    Response::new(bytes)
}

#[tokio::test]
async fn target_source_round_trips_omission_copy_filter_and_atomic_invalid_updates() {
    let server = match MockGmpServer::builder()
        .mode(ServerMode::Stateful)
        .credentials("admin", "admin")
        .unix_socket_auto()
        .build()
        .await
    {
        Ok(server) => server,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return,
        Err(e) => panic!("start mock: {e}"),
    };
    let mut stream = UnixStream::connect(server.socket_path().expect("socket"))
        .await
        .expect("connect");
    assert_eq!(exchange(&mut stream, "<authenticate><credentials><username>admin</username><password>admin</password></credentials></authenticate>").await.status_code(), Some(200));
    let created = exchange(&mut stream, "<create_target><name>AD</name><source>active_directory</source><hosts>192.0.2.1</hosts><port_range>T:1-65535</port_range></create_target>").await;
    assert_eq!(created.status_code(), Some(201));
    let id = created.id().expect("id");
    assert_eq!(
        exchange(
            &mut stream,
            &format!(
                "<modify_target target_id=\"{id}\"><comment>keep source</comment></modify_target>"
            )
        )
        .await
        .status_code(),
        Some(200)
    );
    let cloned = exchange(
        &mut stream,
        &format!("<create_target><copy>{id}</copy></create_target>"),
    )
    .await;
    assert_eq!(cloned.status_code(), Some(201));
    let cloned_id = cloned.id().expect("copy id");
    for selected in [&id, &cloned_id] {
        let result = exchange(
            &mut stream,
            &format!("<get_targets target_id=\"{selected}\"/>"),
        )
        .await;
        let parsed = gvm_gmp::responses::GetTargetsResponse::decode(&result, WireVersion(22, 8))
            .expect("typed target");
        assert_eq!(parsed.items[0].source.as_deref(), Some("active_directory"));
    }
    let manual = exchange(&mut stream, "<create_target><name>manual target</name><hosts>192.0.2.2</hosts><port_range>T:1-65535</port_range></create_target>").await;
    let manual_id = manual.id().expect("manual id");
    let result = exchange(
        &mut stream,
        &format!("<get_targets target_id=\"{manual_id}\"/>"),
    )
    .await;
    assert_eq!(
        gvm_gmp::responses::GetTargetsResponse::decode(&result, WireVersion(22, 8))
            .expect("manual response")
            .items[0]
            .source
            .as_deref(),
        Some("manual")
    );
    for source in ["", "invalid"] {
        let response = exchange(&mut stream, &format!("<modify_target target_id=\"{id}\"><name>must not change</name><source>{source}</source></modify_target>")).await;
        assert_eq!(response.status_code(), Some(400));
    }
    let result = exchange(
        &mut stream,
        "<get_targets filter=\"source=active_directory\"/>",
    )
    .await;
    let parsed = gvm_gmp::responses::GetTargetsResponse::decode(&result, WireVersion(22, 8))
        .expect("filtered targets");
    assert_eq!(parsed.items.len(), 2);
    assert!(parsed
        .items
        .iter()
        .all(|target| target.source.as_deref() == Some("active_directory")));
    assert!(parsed.items.iter().any(|target| target.meta.name == "AD"));
    assert_eq!(
        exchange(
            &mut stream,
            &format!("<modify_target target_id=\"{id}\"><source>manual</source></modify_target>")
        )
        .await
        .status_code(),
        Some(200)
    );
    let result = exchange(&mut stream, &format!("<get_targets target_id=\"{id}\"/>")).await;
    assert_eq!(
        gvm_gmp::responses::GetTargetsResponse::decode(&result, WireVersion(22, 8))
            .expect("updated target")
            .items[0]
            .source
            .as_deref(),
        Some("manual")
    );
    server.shutdown().await;
}
