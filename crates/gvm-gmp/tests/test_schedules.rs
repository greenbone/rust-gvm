// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

#![allow(missing_docs, clippy::unwrap_used)]

mod common;

use common::id;
use gvm_gmp::commands::schedules::{
    CloneScheduleRequest, CreateScheduleRequest, DeleteScheduleRequest, GetScheduleRequest,
    GetSchedulesRequest, ModifyScheduleRequest,
};
use gvm_gmp::schedule::{parse_icalendar, to_icalendar};
use gvm_gmp::{GmpRequestCodec, GmpVersion};
use gvm_gmp::{
    ScheduleDefinition, ScheduleRecurrence, ScheduleStartObservation, ScheduleTimestamp,
};

const ICALENDAR: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR";

fn xml(request: &impl GmpRequestCodec) -> String {
    String::from_utf8(request.encode(GmpVersion(22, 8)).unwrap()).unwrap()
}

#[test]
fn create_schedule_with_complete_request() {
    let mut request = CreateScheduleRequest::new("daily-scan", ICALENDAR);
    request.comment = Some("run daily".into());
    request.timezone = Some("UTC".into());

    assert_eq!(
        xml(&request),
        "<create_schedule><name>daily-scan</name><comment>run daily</comment><icalendar>BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR</icalendar><timezone>UTC</timezone></create_schedule>"
    );
}

#[test]
fn schedule_lifecycle_requests_have_exact_xml() {
    assert_eq!(
        xml(&GetSchedulesRequest {
            details: Some(true),
            ..Default::default()
        }),
        "<get_schedules details=\"1\"/>"
    );
    assert_eq!(
        xml(&GetScheduleRequest::new(id("sc1"))),
        "<get_schedules details=\"1\" schedule_id=\"sc1\"/>"
    );
    assert_eq!(
        xml(&CloneScheduleRequest::new(id("sc1"))),
        "<create_schedule><copy>sc1</copy></create_schedule>"
    );

    let mut modify = ModifyScheduleRequest::new(id("sc1"), ICALENDAR);
    modify.name = Some("updated".into());
    modify.timezone = Some("Europe/Berlin".into());
    assert_eq!(
        xml(&modify),
        "<modify_schedule schedule_id=\"sc1\"><name>updated</name><icalendar>BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR</icalendar><timezone>Europe/Berlin</timezone></modify_schedule>"
    );
    assert_eq!(
        xml(&DeleteScheduleRequest::new(id("sc1"), false)),
        "<delete_schedule schedule_id=\"sc1\" ultimate=\"0\"/>"
    );
}

#[test]
fn schedule_timestamp_preserves_rfc3339_validation_utc_normalization_and_second_truncation() {
    for (input, expected) in [
        (
            "2026-08-10T12:30:00.987654321123+02:00",
            "2026-08-10T10:30:00Z",
        ),
        ("1969-12-31T23:59:59.9Z", "1969-12-31T23:59:59Z"),
        ("2016-12-31T23:59:60.5+01:00", "2016-12-31T22:59:60Z"),
        ("2026-08-10 12:30:00z", "2026-08-10T12:30:00Z"),
    ] {
        assert_eq!(
            ScheduleTimestamp::parse(input)
                .expect("previously accepted RFC 3339 input should parse")
                .as_str(),
            expected
        );
    }

    for invalid in [
        "2026-08-10T12:30:00-04",
        "2026-08-10T12:30:00-04[America/New_York]",
        "2026-08-10T12:30Z",
        "2026-08-10T12:30:00+02:30:15",
        "20260810T123000Z",
        "2026-08-10T12:30:00,5Z",
        "2026-08-10T12:30:00+24:00",
    ] {
        assert!(
            ScheduleTimestamp::parse(invalid).is_err(),
            "non-RFC input should remain rejected: {invalid}"
        );
    }
}

#[test]
fn compact_utc_icalendar_formatting_and_parsing_rejects_malformed_timestamps() {
    let definition = ScheduleDefinition {
        first_run: ScheduleTimestamp::parse("2026-08-10T12:30:00.9+02:00")
            .expect("valid offset timestamp"),
        recurrence: ScheduleRecurrence::Daily,
    };
    let calendar = to_icalendar(&definition);
    assert!(calendar.contains("DTSTART:20260810T103000Z"));
    assert_eq!(
        parse_icalendar(&calendar)
            .expect("generated compact UTC timestamp should parse")
            .first_run,
        ScheduleStartObservation::Supported(definition.first_run)
    );

    let leap_definition = ScheduleDefinition {
        first_run: ScheduleTimestamp::parse("2016-12-31T23:59:60.5+01:00")
            .expect("valid RFC 3339 leap second"),
        recurrence: ScheduleRecurrence::Once,
    };
    assert!(to_icalendar(&leap_definition).contains("DTSTART:20161231T225960Z"));

    for malformed in [
        "BEGIN:VEVENT\r\nDTSTART:20300230T080000Z\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nDTSTART:20300101T0800Z\r\nEND:VEVENT\r\n",
    ] {
        assert!(
            parse_icalendar(malformed).is_err(),
            "malformed compact UTC timestamp should be rejected: {malformed:?}"
        );
    }
}
