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
