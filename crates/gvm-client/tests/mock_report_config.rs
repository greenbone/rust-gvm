// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG
//! Ensure structured report metadata survives the real mock/client transport.
#![cfg(feature = "unix-socket-tests")]
#![allow(missing_docs)]

use gvm_client::GmpClient;
use gvm_connection::UnixSocketConnection;
use gvm_gmp::commands::authentication::AuthenticateRequest;
use gvm_gmp::commands::reports::{GetAuditReportRequest, GetScanReportRequest};
use gvm_gmp::EntityId;
use gvm_mock_server::{GmpVersion, MockGmpServer, Resource, ServerMode};

#[tokio::test]
async fn structured_scan_and_audit_reports_keep_optional_config_references() {
    // Configured tasks preserve escaped names; configless task reports must not
    // acquire a default configuration merely because one exists in the store.
    for audit in [false, true] {
        for configured in [false, true] {
            let config = Resource::new("config", "Full & fast <policy>");
            let config_id = config.id.to_string();
            let expected_id = config_id.clone();
            let mut task = Resource::new("task", "Report-producing task");
            if configured {
                task.set_attr("config_id", &config_id);
            }
            let mut report = Resource::new("report", "Structured report");
            report.set_attr("task_id", &task.id.to_string());
            report.set_attr("status", "Done");
            report.set_attr("usage_type", if audit { "audit" } else { "scan" });
            let report_id = EntityId::new(report.id.to_string()).expect("UUID");
            let server = MockGmpServer::builder()
                .mode(ServerMode::Stateful)
                .version(GmpVersion::V22_8)
                .seed(move |store| {
                    store.seed(config);
                    store.seed(task);
                    store.seed(report);
                })
                .unix_socket_auto()
                .build()
                .await
                .expect("start mock server");
            let mut client = GmpClient::connect(UnixSocketConnection::with_path(
                server.socket_path().expect("socket"),
            ))
            .await
            .expect("connect");
            client
                .authenticate(AuthenticateRequest::new("admin", "admin"))
                .await
                .expect("authenticate");
            let task = if audit {
                client
                    .get_audit_report(GetAuditReportRequest::new(report_id))
                    .await
                    .expect("decode audit report")
                    .report
                    .task
            } else {
                client
                    .get_scan_report(GetScanReportRequest::new(report_id))
                    .await
                    .expect("decode scan report")
                    .report
                    .task
            }
            .expect("linked task");
            if configured {
                let config = task.scan_config.expect("report configuration reference");
                assert_eq!(config.id.as_str(), expected_id.as_str());
                assert_eq!(config.name, "Full & fast <policy>");
            } else {
                assert!(task.scan_config.is_none(), "do not invent a configuration");
            }
            server.shutdown().await;
        }
    }
}
