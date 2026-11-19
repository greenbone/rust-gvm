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
fn modify_agent_group_requires_name_even_for_delay_only_update() {
    let mut request = ModifyAgentGroupRequest::new(id("group-1"), "Group & name", "0 */5 * * *");
    request.max_delay_seconds = Some(30);
    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time><max_delay_seconds>30</max_delay_seconds><name>Group &amp; name</name></modify_agent_group>"
    );
    for name in [None, Some(String::new())] {
        request.name = name;
        assert!(request.validate().is_err());
        assert!(request.encode(GmpVersion(22, 8)).is_err());
    }
    let empty = ModifyAgentGroupRequest::new(id("group-1"), "", "0 */5 * * *");
    assert!(empty.validate().is_err());
}

#[test]
fn modify_agent_group_membership_preserve_omits_agents_exactly() {
    let request = ModifyAgentGroupRequest::new(id("group-1"), "Group", "0 */5 * * *");

    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time><name>Group</name></modify_agent_group>"
    );
}

#[test]
fn modify_agent_group_membership_replacement_emits_exact_agents_container() {
    let mut request = ModifyAgentGroupRequest::new(id("group-1"), "Group", "0 */5 * * *");
    request.agent_ids = CollectionUpdate::replace([id("agent-1"), id("agent-2")]);

    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time><name>Group</name><agents><agent id=\"agent-1\"/><agent id=\"agent-2\"/></agents></modify_agent_group>"
    );
}

#[test]
fn modify_agent_group_membership_clear_emits_exact_empty_agents_container() {
    let mut request = ModifyAgentGroupRequest::new(id("group-1"), "Group", "0 */5 * * *");
    request.agent_ids = CollectionUpdate::Clear;

    assert_eq!(
        request_xml(&request),
        "<modify_agent_group agent_group_id=\"group-1\"><scheduler_cron_time>0 */5 * * *</scheduler_cron_time><name>Group</name><agents/></modify_agent_group>"
    );
}
