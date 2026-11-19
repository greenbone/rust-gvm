// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

#![allow(missing_docs, clippy::unwrap_used)]

use gvm_gmp::responses::{GetAuditReportResponse, GetScanReportResponse, StructuredReportTask};
use gvm_protocol::Response;

fn parse_task(kind: &str, task: &str) -> StructuredReportTask {
    let xml = format!(
        r#"<get_{kind}_report_response status="200" status_text="OK"><report id="report"><name>Report</name>{task}</report></get_{kind}_report_response>"#
    );
    let response = Response::from(xml.as_str());
    if kind == "scan" {
        GetScanReportResponse::from_response(&response)
            .unwrap()
            .report
            .task
            .unwrap()
    } else {
        GetAuditReportResponse::from_response(&response)
            .unwrap()
            .report
            .task
            .unwrap()
    }
}

#[test]
fn scan_and_audit_preserve_report_config_and_existing_task_fields() {
    for kind in ["scan", "audit"] {
        let task = parse_task(
            kind,
            r#"<task id="task-id"><name>Task &amp; name</name><comment>Comment</comment><scan_config id="daba56c8-73ec-11df-a475-002264764cea"><name>Full &amp; fast</name></scan_config></task>"#,
        );
        assert_eq!(task.id.as_ref().unwrap().as_str(), "task-id");
        assert_eq!(task.name, "Task & name");
        assert_eq!(task.comment.as_deref(), Some("Comment"));
        let config = task.scan_config.as_ref().unwrap();
        assert_eq!(config.id.as_str(), "daba56c8-73ec-11df-a475-002264764cea");
        assert_eq!(config.name, "Full & fast");
        #[cfg(feature = "serde")]
        {
            let json = serde_json::to_string(&task).unwrap();
            let restored: StructuredReportTask = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, task);
        }
    }
}

#[test]
fn older_and_configless_report_tasks_do_not_synthesize_configs() {
    for kind in ["scan", "audit"] {
        for name in [
            "Older server task",
            "CVE task",
            "Container task",
            "Agent task",
        ] {
            let task = parse_task(
                kind,
                &format!("<task id=\"task-id\"><name>{name}</name><comment>Kept</comment></task>"),
            );
            assert!(task.scan_config.is_none());
            assert_eq!(task.name, name);
            assert_eq!(task.comment.as_deref(), Some("Kept"));
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn older_serialized_task_without_scan_config_remains_readable() {
    let task: StructuredReportTask = serde_json::from_str(
        r#"{"id":"task-id","name":"Legacy","comment":null,"target":null,"progress":null}"#,
    )
    .unwrap();
    assert!(task.scan_config.is_none());
    assert_eq!(task.name, "Legacy");
}
