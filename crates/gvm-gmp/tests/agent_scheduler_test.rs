// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG
#![allow(missing_docs, clippy::unwrap_used)]

use gvm_gmp::commands::agent_groups::{CreateAgentGroupRequest, ModifyAgentGroupRequest};
use gvm_gmp::commands::agents::{
    AgentConfigOpts, AgentScriptExecutorConfig, ModifyAgentControlScanConfigRequest,
    ModifyAgentRequest,
};
use gvm_gmp::responses::{GetAgentsResponse, GetScannersResponse};
use gvm_gmp::{AgentSchedule, EntityId, GmpRequestCodec, GmpVersion};
use gvm_protocol::Response;

fn config(delay: Option<u32>) -> AgentConfigOpts {
    AgentConfigOpts {
        agent_script_executor: Some(AgentScriptExecutorConfig {
            scheduler_cron_time: vec![AgentSchedule {
                cron: "0 */6 * * *".into(),
                max_delay_seconds: delay,
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn scheduler_writes_nested_items_and_validates_delay_at_both_entry_points() {
    for delay in [
        None,
        Some(0),
        Some(30),
        Some(i32::MAX as u32),
        Some(i32::MAX as u32 + 1),
    ] {
        let mut defaults =
            ModifyAgentControlScanConfigRequest::new(EntityId::new("scanner-1").unwrap());
        defaults.agent_defaults = Some(config(delay));
        let mut agent = ModifyAgentRequest::new(vec![EntityId::new("agent-1").unwrap()]);
        agent.config = Some(config(delay));
        for request in [&defaults as &dyn GmpRequestCodec, &agent] {
            let result = request.encode(GmpVersion(22, 8));
            if delay == Some(i32::MAX as u32 + 1) {
                assert!(request.validate().is_err());
                assert!(result.is_err());
            } else {
                let xml = String::from_utf8(result.unwrap()).unwrap();
                let expected = delay.map_or_else(|| "<item><cron>0 */6 * * *</cron></item>".into(), |delay| format!("<item><cron>0 */6 * * *</cron><max_delay_seconds>{delay}</max_delay_seconds></item>"));
                assert!(xml.contains(&expected));
            }
        }
    }
}

#[test]
fn agent_groups_validate_and_encode_zero_and_maximum_delays() {
    for delay in [
        None,
        Some(0),
        Some(30),
        Some(i32::MAX as u32),
        Some(u32::MAX),
    ] {
        let mut create = CreateAgentGroupRequest::new("Group", vec![], "0 */6 * * *");
        create.max_delay_seconds = delay;
        let mut modify =
            ModifyAgentGroupRequest::new(EntityId::new("group-1").unwrap(), "Group", "0 */6 * * *");
        modify.max_delay_seconds = delay;
        for request in [&create as &dyn GmpRequestCodec, &modify] {
            let encoded = request.encode(GmpVersion(22, 8));
            if delay == Some(u32::MAX) {
                assert!(encoded.is_err());
                assert!(request.validate().is_err());
            } else {
                let xml = String::from_utf8(encoded.unwrap()).unwrap();
                assert_eq!(xml.contains("<max_delay_seconds>"), delay.is_some());
                if let Some(delay) = delay {
                    assert!(
                        xml.contains(&format!("<max_delay_seconds>{delay}</max_delay_seconds>"))
                    );
                }
            }
        }
    }
}

#[test]
fn agent_and_scanner_defaults_decode_actual_nested_schedule_shape() {
    let executor = r#"<agent_script_executor><scheduler_cron_time is_list="1"><item><cron>0 */6 * * *</cron><max_delay_seconds>30</max_delay_seconds></item><item><cron>15 * * * *</cron><max_delay_seconds>0</max_delay_seconds></item><item>legacy cron</item><item><cron/></item></scheduler_cron_time></agent_script_executor>"#;
    let agents = GetAgentsResponse::from_response(&Response::from(format!(r#"<get_agents_response status="200" status_text="OK"><agent id="agent-1"><name>Agent</name><config>{executor}</config></agent></get_agents_response>"#).into_bytes())).unwrap();
    let scanners = GetScannersResponse::from_response(&Response::from(format!(r#"<get_scanners_response status="200" status_text="OK"><scanner id="scanner-1"><name>Control</name><agent_control_config_defaults><agent_defaults>{executor}</agent_defaults><agent_control_defaults><update_to_latest>1</update_to_latest></agent_control_defaults></agent_control_config_defaults></scanner></get_scanners_response>"#).into_bytes())).unwrap();
    let agent_entries = &agents.items[0]
        .config
        .as_ref()
        .unwrap()
        .agent_script_executor
        .as_ref()
        .unwrap()
        .scheduler_cron_time;
    let defaults = scanners.items[0]
        .agent_control_config_defaults
        .as_ref()
        .unwrap();
    assert_eq!(defaults.update_to_latest, Some(true));
    assert_eq!(
        agent_entries,
        &defaults
            .agent_defaults
            .as_ref()
            .unwrap()
            .agent_script_executor
            .as_ref()
            .unwrap()
            .scheduler_cron_time
    );
    assert_eq!(
        agent_entries,
        &vec![
            AgentSchedule {
                cron: "0 */6 * * *".into(),
                max_delay_seconds: Some(30)
            },
            AgentSchedule {
                cron: "15 * * * *".into(),
                max_delay_seconds: Some(0)
            },
            "legacy cron".into(),
            "".into()
        ]
    );
}

#[test]
fn malformed_response_delay_is_not_silently_discarded() {
    for delay in ["-1", "invalid", "4294967296"] {
        let response = Response::from(format!(
            r#"<get_agents_response status="200" status_text="OK"><agent id="agent-1"><name>Agent</name><config><agent_script_executor><scheduler_cron_time><item><cron>* * * * *</cron><max_delay_seconds>{delay}</max_delay_seconds></item></scheduler_cron_time></agent_script_executor></config></agent></get_agents_response>"#
        ).into_bytes());
        assert!(GetAgentsResponse::from_response(&response).is_err());
    }
}
