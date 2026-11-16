// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

#![allow(missing_docs, clippy::unwrap_used)]

mod common;

use common::id;
use gvm_gmp::commands::agent_groups::ModifyAgentGroupRequest;
use gvm_gmp::{CollectionUpdate, GmpRequestCodec, GmpVersion};

fn request_xml(request: &impl GmpRequestCodec) -> String {
    String::from_utf8(request.encode(GmpVersion(22, 8)).unwrap()).unwrap()
}

#[test]
fn modify_agent_group_membership_preserve_omits_agents_exactly() {
    let request = ModifyAgentGroupRequest::new(id("group-1"), "0 */5 * * *");

    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time></modify_agent_group>"
    );
}

#[test]
fn modify_agent_group_membership_replacement_emits_exact_agents_container() {
    let mut request = ModifyAgentGroupRequest::new(id("group-1"), "0 */5 * * *");
    request.agent_ids = CollectionUpdate::replace([id("agent-1"), id("agent-2")]);

    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time><agents><agent id=\"agent-1\"/><agent id=\"agent-2\"/></agents></modify_agent_group>"
    );
}

#[test]
fn modify_agent_group_membership_clear_emits_exact_empty_agents_container() {
    let mut request = ModifyAgentGroupRequest::new(id("group-1"), "0 */5 * * *");
    request.agent_ids = CollectionUpdate::Clear;

    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time><agents/></modify_agent_group>"
    );
}
