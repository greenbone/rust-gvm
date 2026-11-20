// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG
#![allow(clippy::unwrap_used, missing_docs)]

use gvm_gmp::commands::agents::GetAgentInventoryTokenRequest;
use gvm_gmp::responses::GetAgentInventoryTokenResponse;
use gvm_gmp::{GmpCommand, GmpRequestCodec, GmpVersion};
use gvm_protocol::Response;

#[test]
fn inventory_token_request_has_no_arguments() {
    let request = GetAgentInventoryTokenRequest;
    assert_eq!(
        request.command(),
        Some(GmpCommand::new("get_agent_inventory_token"))
    );
    assert_eq!(
        request.encode(GmpVersion(22, 8)).unwrap(),
        b"<get_agent_inventory_token/>"
    );
}

#[test]
fn inventory_token_decodes_escaped_content_and_redacts_debug() {
    let response = Response::from(
        r#"<get_agent_inventory_token_response status="200" status_text="OK"><token>secret&amp;&lt;value&gt;</token></get_agent_inventory_token_response>"#,
    );
    let parsed = GetAgentInventoryTokenResponse::from_response(&response).unwrap();
    assert_eq!(parsed.token.as_str(), "secret&<value>");
    assert!(!format!("{parsed:?}").contains("secret"));
    assert!(!format!("{:?}", parsed.token).contains("secret"));
}

#[test]
fn inventory_token_missing_payload_is_not_a_success() {
    let response =
        Response::from(r#"<get_agent_inventory_token_response status="200" status_text="OK"/>"#);
    assert!(GetAgentInventoryTokenResponse::from_response(&response).is_err());
}

// Successful status must not turn an absent credential into successful issuance.
#[test]
fn inventory_token_rejects_empty_and_whitespace_payloads_with_redacted_typed_error() {
    for payload in [
        "<token/>",
        "<token></token>",
        "<token> </token>",
        "<token>\t\r\n </token>",
        "<token>&#x20;&#x9;&#xD;&#xA;</token>",
    ] {
        let xml = format!(
            r#"<get_agent_inventory_token_response status="200" status_text="OK">{payload}</get_agent_inventory_token_response>"#
        );
        let error = GetAgentInventoryTokenResponse::from_response(&Response::from(xml.as_str()))
            .unwrap_err();
        assert!(
            matches!(&error, gvm_gmp::responses::ParseError::InvalidValue { field, value }
            if field == "token" && value == "[REDACTED]")
        );
        assert!(!format!("{error:?}").contains(payload));
    }
}

// Validation must not interpret the contents of a nonempty opaque credential.
#[test]
fn inventory_token_preserves_opaque_content_after_existing_xml_decoding() {
    let response = Response::from(
        r#"<get_agent_inventory_token_response status="200" status_text="OK"><token>opaque +/=&#x9;middle&amp;&lt;&gt;&quot;&apos;end</token></get_agent_inventory_token_response>"#,
    );
    let parsed = GetAgentInventoryTokenResponse::from_response(&response).unwrap();
    assert_eq!(parsed.token.as_str(), "opaque +/=\tmiddle&<>\"'end");
    assert!(!format!("{parsed:?}").contains("opaque"));
}

// Server failures take precedence even when their payload is missing or invalid.
#[test]
fn inventory_token_checks_server_status_before_payload() {
    for payload in ["", "<token/>", "<token> </token>", "<token>secret</token>"] {
        let xml = format!(
            r#"<get_agent_inventory_token_response status="503" status_text="Unavailable">{payload}</get_agent_inventory_token_response>"#
        );
        let error = GetAgentInventoryTokenResponse::from_response(&Response::from(xml.as_str()))
            .unwrap_err();
        assert!(matches!(
            error,
            gvm_gmp::responses::ParseError::ServerError { status: 503, .. }
        ));
        assert!(!format!("{error:?}").contains("secret"));
    }
}
