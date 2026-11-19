// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG
#![cfg(feature = "serde")]
#![allow(missing_docs, clippy::unwrap_used)]

use gvm_gmp::AgentSchedule;
use serde_json::json;

#[test]
fn legacy_and_structured_entries_can_be_loaded_together() {
    let entries: Vec<AgentSchedule> = serde_json::from_value(json!([
        "0 */6 * * *",
        {"cron": "15 * * * *"},
        {"cron": "30 * * * *", "max_delay_seconds": null},
        {"cron": "45 * * * *", "max_delay_seconds": 0},
        {"cron": "* * * * *", "max_delay_seconds": 30}
    ]))
    .unwrap();

    assert_eq!(entries[0], AgentSchedule::from("0 */6 * * *"));
    assert_eq!(entries[1], AgentSchedule::from("15 * * * *"));
    assert_eq!(entries[2], AgentSchedule::from("30 * * * *"));
    assert_eq!(entries[3].max_delay_seconds, Some(0));
    assert_eq!(entries[4].max_delay_seconds, Some(30));

    // Legacy input upgrades to an object, preserving omitted versus zero delay.
    let serialized = serde_json::to_value(&entries).unwrap();
    assert_eq!(
        serialized,
        json!([
            {"cron": "0 */6 * * *", "max_delay_seconds": null},
            {"cron": "15 * * * *", "max_delay_seconds": null},
            {"cron": "30 * * * *", "max_delay_seconds": null},
            {"cron": "45 * * * *", "max_delay_seconds": 0},
            {"cron": "* * * * *", "max_delay_seconds": 30}
        ])
    );
    assert_eq!(
        serde_json::from_value::<Vec<AgentSchedule>>(serialized).unwrap(),
        entries
    );
}

#[test]
fn structured_delay_preserves_the_full_observation_range() {
    let value = json!({"cron": "* * * * *", "max_delay_seconds": u32::MAX});
    let schedule: AgentSchedule = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(schedule.max_delay_seconds, Some(u32::MAX));
    assert_eq!(serde_json::to_value(schedule).unwrap(), value);
}

#[test]
fn malformed_schedule_values_are_not_accepted_as_legacy_entries() {
    for value in [
        json!(null),
        json!(42),
        json!(true),
        json!({}),
        json!({"max_delay_seconds": 30}),
        json!({"cron": null}),
        json!({"cron": 42}),
        json!({"cron": "* * * * *", "max_delay_seconds": -1}),
        json!({"cron": "* * * * *", "max_delay_seconds": 4294967296_u64}),
        json!({"cron": "* * * * *", "max_delay_seconds": "30"}),
        json!({"cron": "* * * * *", "max_delay_seconds": 0.5}),
    ] {
        assert!(
            serde_json::from_value::<AgentSchedule>(value.clone()).is_err(),
            "unexpectedly accepted {value}"
        );
    }
}
