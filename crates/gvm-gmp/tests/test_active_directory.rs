// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

#![allow(missing_docs, clippy::unwrap_used)]

use gvm_gmp::commands::active_directory::{
    ActiveDirectoryObjectType as Kind, ActiveDirectorySearchScope as Scope,
    GetActiveDirectoryObjectsRequest,
};
use gvm_gmp::responses::active_directory::GetActiveDirectoryObjectsResponse;
use gvm_gmp::responses::{GmpServerErrorKind, ParseError};
use gvm_gmp::{EntityId, GmpCommand, GmpRequest, GmpRequestCodec, GmpResponse, GmpVersion};
use gvm_protocol::Response;

fn request() -> GetActiveDirectoryObjectsRequest {
    GetActiveDirectoryObjectsRequest::new(EntityId::new("ad-credential").unwrap())
}

fn xml(request: &GetActiveDirectoryObjectsRequest) -> String {
    // Encoding does not assert historical availability; support is discovery-gated.
    String::from_utf8(request.encode(GmpVersion(22, 8)).unwrap()).unwrap()
}

fn decode(xml: &str) -> Result<GetActiveDirectoryObjectsResponse, ParseError> {
    GetActiveDirectoryObjectsResponse::decode(&Response::from(xml), GmpVersion(22, 8))
}

#[test]
fn default_request_preserves_server_defaults_and_response_association() {
    fn associated<R: GmpRequest<Response = GetActiveDirectoryObjectsResponse>>(_: &R) {}
    let req = request();
    associated(&req);
    assert_eq!(req.validate(), Ok(()));
    assert_eq!(
        req.command(),
        Some(GmpCommand::new("get_active_directory_objects"))
    );
    assert_eq!(
        xml(&req),
        "<get_active_directory_objects credential_id=\"ad-credential\"/>"
    );
}

#[test]
fn complete_request_serializes_all_controls_and_escaped_dns() {
    let mut req = request();
    req.scope = Some(Scope::Subtree);
    req.max_results = Some(1000);
    req.include_disabled_computers = Some(false);
    req.targets = Some(true);
    req.tasks = Some(true);
    req.details = Some(true);
    req.base_dn = Some("OU=R&D,DC=example".into());
    req.group_dn = Some("CN=<team>,DC=example".into());
    req.object_types = Some(vec![
        Kind::Ou,
        Kind::Container,
        Kind::Computer,
        Kind::Group,
        Kind::Container,
    ]);
    assert_eq!(xml(&req), "<get_active_directory_objects credential_id=\"ad-credential\" details=\"1\" include_disabled_computers=\"0\" max_results=\"1000\" scope=\"subtree\" targets=\"1\" tasks=\"1\"><base_dn>OU=R&amp;D,DC=example</base_dn><group_dn>CN=&lt;team&gt;,DC=example</group_dn><object_types><object_type>ou</object_type><object_type>container</object_type><object_type>computer</object_type><object_type>group</object_type></object_types></get_active_directory_objects>");
    for (scope, wire) in [
        (Scope::Base, "base"),
        (Scope::OneLevel, "onelevel"),
        (Scope::Subtree, "subtree"),
    ] {
        req.scope = Some(scope);
        assert!(xml(&req).contains(&format!("scope=\"{wire}\"")));
    }
    req.tasks = Some(false);
    req.targets = Some(false);
    req.details = Some(false);
    req.include_disabled_computers = Some(true);
    assert!(xml(&req).contains("tasks=\"0\""));
    assert!(xml(&req).contains("targets=\"0\""));
    assert!(xml(&req).contains("details=\"0\""));
    assert!(xml(&req).contains("include_disabled_computers=\"1\""));
}

#[test]
fn validation_rejects_invalid_limits_empty_selectors_and_task_dependencies() {
    for limit in [0, 1001, u32::MAX] {
        let mut req = request();
        req.max_results = Some(limit);
        assert!(req.validate().is_err());
        assert!(req.encode(GmpVersion(22, 8)).is_err());
    }
    for limit in [1, 1000] {
        let mut req = request();
        req.max_results = Some(limit);
        assert!(req.validate().is_ok());
    }
    let mut req = request();
    req.tasks = Some(true);
    assert!(req.validate().is_err());
    req.targets = Some(false);
    assert!(req.validate().is_err());
    req.targets = Some(true);
    assert!(req.validate().is_ok());
    let mut req = request();
    req.base_dn = Some(String::new());
    assert!(req.validate().is_err());
    req.base_dn = None;
    req.group_dn = Some(String::new());
    assert!(req.validate().is_err());
    req.group_dn = None;
    req.object_types = Some(vec![]);
    assert!(req.validate().is_err());
}

const FULL: &str = r#"<get_active_directory_objects_response status="200" status_text="OK">
<objects truncated="1"><count><ou>1</ou><container>1</container><group>1</group><computer><total>2</total><disabled>1</disabled></computer></count>
<object type="ou"><name>R&amp;D</name><distinguished_name>OU=R&amp;D,DC=example</distinguished_name><parent_dn>DC=example</parent_dn><description>Research &lt;team&gt;</description></object>
<object type="container"><name>Computers</name><distinguished_name>CN=Computers,DC=example</distinguished_name><parent_dn>DC=example</parent_dn><description/></object>
<object type="group"><name>Scan group</name><distinguished_name>CN=Scan group,DC=example</distinguished_name><parent_dn/><description/></object>
<object type="computer"><name>PC1</name><distinguished_name>CN=PC1,OU=R&amp;D,DC=example</distinguished_name><parent_dn>OU=R&amp;D,DC=example</parent_dn><description>Desktop</description><dns_hostname>pc1.example</dns_hostname><operating_system>Windows</operating_system><operating_system_version>10.0</operating_system_version><service_principal_names><service_principal_name>HOST/pc1</service_principal_name><service_principal_name>HOST/pc1.example</service_principal_name></service_principal_names><disabled>1</disabled><targets><target id="target-1"><name>PC1 &amp; friends</name><tasks><task id="task-1"><name>Daily</name><config id="config-1"><name>Full &amp; fast</name></config></task><task id="task-2"><name>Hidden config</name></task></tasks></target><target id="target-2"><name>No tasks</name><tasks/></target></targets></object>
<object type="computer"><name>PC2</name><distinguished_name>CN=PC2,DC=example</distinguished_name><parent_dn>DC=example</parent_dn><description/><service_principal_names/><disabled>0</disabled><targets/></object>
</objects></get_active_directory_objects_response>"#;

#[test]
fn current_collection_decodes_counts_containers_details_and_resource_references() {
    let response = decode(FULL).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.status_text, "OK");
    let objects = response.objects.unwrap();
    assert!(objects.truncated);
    assert_eq!(objects.counts.ou, 1);
    assert_eq!(objects.counts.container, 1);
    assert_eq!(objects.counts.group, 1);
    assert_eq!(objects.counts.computer.total, 2);
    assert_eq!(objects.counts.computer.disabled, Some(1));
    assert_eq!(objects.items.len(), 5);
    let ou = &objects.items[0];
    assert_eq!(ou.object_type, Kind::Ou);
    assert_eq!(ou.name, "R&D");
    assert_eq!(ou.distinguished_name, "OU=R&D,DC=example");
    assert_eq!(ou.description, "Research <team>");
    assert!(ou.targets.is_none());
    assert!(ou.disabled.is_none());
    assert!(ou.service_principal_names.is_none());
    assert_eq!(objects.items[1].object_type, Kind::Container);
    assert_eq!(objects.items[1].description, "");
    assert_eq!(objects.items[2].object_type, Kind::Group);
    assert_eq!(objects.items[2].parent_dn, "");
    let pc = &objects.items[3];
    assert_eq!(pc.object_type, Kind::Computer);
    assert_eq!(pc.dns_hostname.as_deref(), Some("pc1.example"));
    assert_eq!(pc.operating_system.as_deref(), Some("Windows"));
    assert_eq!(pc.operating_system_version.as_deref(), Some("10.0"));
    assert_eq!(pc.disabled, Some(true));
    assert_eq!(
        pc.service_principal_names.as_ref().unwrap(),
        &["HOST/pc1", "HOST/pc1.example"]
    );
    let targets = pc.targets.as_ref().unwrap();
    assert_eq!(targets[0].id.as_str(), "target-1");
    assert_eq!(targets[0].name, "PC1 & friends");
    let tasks = targets[0].tasks.as_ref().unwrap();
    assert_eq!(tasks[0].id.as_str(), "task-1");
    assert_eq!(tasks[0].name, "Daily");
    let config = tasks[0].config.as_ref().unwrap();
    assert_eq!(config.id.as_str(), "config-1");
    assert_eq!(config.name, "Full & fast");
    assert!(tasks[1].config.is_none());
    assert_eq!(targets[1].tasks, Some(vec![]));
    let pc = &objects.items[4];
    assert!(pc.operating_system_version.is_none());
    assert!(pc.dns_hostname.is_none());
    assert_eq!(pc.disabled, Some(false));
    assert_eq!(pc.service_principal_names, Some(vec![]));
    assert_eq!(pc.targets, Some(vec![]));
}

#[test]
fn empty_absent_and_truncated_collections_remain_distinguishable() {
    for truncated in ["0", "1"] {
        let xml = format!("<get_active_directory_objects_response status=\"200\" status_text=\"OK\"><objects truncated=\"{truncated}\"><count><ou>0</ou><container>0</container><group>0</group><computer><total>0</total></computer></count></objects></get_active_directory_objects_response>");
        let objects = decode(&xml).unwrap().objects.unwrap();
        assert_eq!(objects.truncated, truncated == "1");
        assert!(objects.items.is_empty());
        assert_eq!(objects.counts.computer.total, 0);
        assert_eq!(objects.counts.computer.disabled, None);
    }
    assert!(
        decode("<get_active_directory_objects_response status=\"200\" status_text=\"OK\"/>")
            .unwrap()
            .objects
            .is_none()
    );
    let without_tasks = FULL.replace("<tasks/>", "");
    let objects = decode(&without_tasks).unwrap().objects.unwrap();
    assert!(objects.items[3].targets.as_ref().unwrap()[1]
        .tasks
        .is_none());
}

#[test]
fn malformed_counts_flags_types_and_references_are_not_silently_defaulted() {
    for xml in [
        FULL.replace("<total>2</total>", "<total>no</total>"),
        FULL.replace("<container>1</container>", ""),
        FULL.replace("<disabled>1</disabled>", "<disabled>invalid</disabled>"),
        FULL.replace("truncated=\"1\"", "truncated=\"invalid\""),
        FULL.replace("type=\"container\"", "type=\"unknown\""),
        FULL.replace("id=\"task-1\"", ""),
        FULL.replace("id=\"config-1\"", "id=\"\""),
    ] {
        assert!(decode(&xml).is_err(), "{xml}");
    }
}

#[test]
fn feature_disabled_denied_and_destination_errors_preserve_server_evidence() {
    for message in [
        "Active Directory integration is disabled",
        "Permission denied",
        "Active Directory destination is not allowed.",
        "Stored Active Directory credential configuration is invalid",
    ] {
        let xml = format!(
            "<get_active_directory_objects_response status=\"400\" status_text=\"{message}\"/>"
        );
        let err = decode(&xml).unwrap_err();
        if message == "Permission denied" {
            assert_eq!(
                err.server_error_kind(),
                Some(GmpServerErrorKind::PermissionDenied)
            );
        }
        assert!(
            matches!(err, ParseError::ServerError {status: 400, message: text} if text == message)
        );
    }
}
