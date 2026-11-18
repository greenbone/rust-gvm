// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG
#![allow(missing_docs, clippy::unwrap_used)]
mod common;
use common::id;
use gvm_gmp::commands::credentials::*;
use gvm_gmp::responses::{
    CredentialKind, GetCredentialsResponse, VerifyActiveDirectoryCredentialResponse,
};
use gvm_gmp::{CredentialType, GmpRequestCodec, GmpResponse, GmpVersion};
use gvm_protocol::Response;

fn fields() -> ActiveDirectoryCredentialFields {
    ActiveDirectoryCredentialFields {
        host: Some("dc.example".into()),
        bind_dn: Some("CN=svc&ops,DC=example".into()),
        bind_password: Some("secret<&".into()),
        base_dn: Some("DC=example".into()),
        tls_mode: Some(ActiveDirectoryTlsMode::StartTls),
        ..Default::default()
    }
}
fn xml(request: &impl GmpRequestCodec) -> String {
    String::from_utf8(request.encode(GmpVersion(22, 4)).unwrap()).unwrap()
}
#[test]
fn create_modify_inline_and_saved_use_canonical_fields() {
    let mut create = CreateCredentialRequest::new("AD");
    create.credential_type = Some(CredentialType::ActiveDirectory);
    create.active_directory = Some(fields());
    let value = xml(&create);
    assert!(value.contains("<type>ad</type>"));
    assert!(value.contains("<bind_dn>CN=svc&amp;ops,DC=example</bind_dn>"));
    assert!(value.contains("<bind_password>secret&lt;&amp;</bind_password>"));
    assert!(!value.contains("<port>"));
    assert!(!format!("{create:?}").contains("secret"));
    let inline = VerifyActiveDirectoryCredentialRequest::Inline(fields());
    assert_eq!(xml(&inline), "<verify_active_directory_credential><host>dc.example</host><bind_dn>CN=svc&amp;ops,DC=example</bind_dn><bind_password>secret&lt;&amp;</bind_password><base_dn>DC=example</base_dn><tls_mode>starttls</tls_mode></verify_active_directory_credential>");
    assert!(!format!("{inline:?}").contains("secret"));
    let saved = VerifyActiveDirectoryCredentialRequest::Saved {
        credential_id: id("credential-1"),
    };
    assert_eq!(
        xml(&saved),
        "<verify_active_directory_credential credential_id=\"credential-1\"/>"
    );
    let mut modify = ModifyCredentialRequest::new(id("credential-1"));
    modify.active_directory = Some(ActiveDirectoryCredentialFields {
        bind_password: Some("replacement".into()),
        ca_certificate: Some(String::new()),
        port: Some(636),
        tls_mode: Some(ActiveDirectoryTlsMode::Ldaps),
        ..Default::default()
    });
    let wire = xml(&modify);
    assert!(wire.contains("<port>636</port>"));
    assert!(wire.contains("<ca_certificate></ca_certificate>"));
    assert!(!wire.contains("<host>"));
    assert!(!format!("{modify:?}").contains("replacement"));
    modify.active_directory = Some(ActiveDirectoryCredentialFields {
        clear_port: true,
        ..Default::default()
    });
    assert!(xml(&modify).contains("<port/>"));
    let fields = ActiveDirectoryCredentialFields {
        clear_port: true,
        ..fields()
    };
    assert!(VerifyActiveDirectoryCredentialRequest::Inline(fields)
        .validate()
        .is_err());
}
#[test]
fn complete_forms_require_fields_but_modify_is_partial() {
    assert!(
        VerifyActiveDirectoryCredentialRequest::Inline(Default::default())
            .validate()
            .is_err()
    );
    for name in [
        "host",
        "bind_dn",
        "bind_password",
        "base_dn",
        "tls_mode",
        "port",
    ] {
        let mut value = fields();
        match name {
            "host" => value.host = None,
            "bind_dn" => value.bind_dn = None,
            "bind_password" => value.bind_password = None,
            "base_dn" => value.base_dn = None,
            "tls_mode" => value.tls_mode = None,
            _ => value.port = Some(0),
        }
        let error = VerifyActiveDirectoryCredentialRequest::Inline(value)
            .validate()
            .unwrap_err();
        assert!(!format!("{error:?}").contains("secret"));
    }
    let mut create = CreateCredentialRequest::new("AD");
    create.active_directory = Some(fields());
    assert!(create.validate().is_err());
    create.credential_type = Some(CredentialType::ActiveDirectory);
    assert!(create.validate().is_ok());
}
#[test]
fn response_preserves_non_secret_metadata_only() {
    let response = Response::new(br#"<get_credentials_response status="200" status_text="OK"><credential id="credential-1"><name>AD</name><type>ad</type><host>dc.example</host><port>636</port><bind_dn>CN=svc&amp;ops</bind_dn><base_dn>DC=example</base_dn><tls_mode>ldaps</tls_mode><ca_certificate>PEM</ca_certificate><bind_password>must-not-retain</bind_password></credential></get_credentials_response>"#.to_vec());
    let parsed = GetCredentialsResponse::decode(&response, GmpVersion(22, 8)).unwrap();
    assert_eq!(parsed.items[0].kind, CredentialKind::ActiveDirectory);
    let ad = parsed.items[0].active_directory.as_ref().unwrap();
    assert_eq!(ad.port, Some(636));
    assert_eq!(ad.bind_dn.as_deref(), Some("CN=svc&ops"));
    assert!(!format!("{parsed:?}").contains("must-not-retain"));
    assert_eq!(
        CredentialKind::from(CredentialType::ActiveDirectory),
        CredentialKind::ActiveDirectory
    );
}
#[test]
fn server_failures_are_not_reinterpreted() {
    for (status, message) in [
        (400, "Permission denied"),
        (400, "Command unavailable"),
        (400, "Destination not allowed"),
        (503, "Directory unavailable"),
    ] {
        let response = Response::new(format!("<verify_active_directory_credential_response status=\"{status}\" status_text=\"{message}\"/>").into_bytes());
        let error = VerifyActiveDirectoryCredentialResponse::decode(&response, GmpVersion(22, 8))
            .unwrap_err();
        assert!(
            matches!(error, gvm_gmp::responses::ParseError::ServerError { status: got, message: text } if got == status && text == message)
        );
    }
}
