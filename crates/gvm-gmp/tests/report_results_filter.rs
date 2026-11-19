// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Greenbone AG

#![allow(missing_docs)]
use gvm_gmp::commands::results::GetResultsRequest;
use gvm_gmp::{
    compose_report_results_filter, EntityId, GmpRequestCodec, GmpVersion, PaginatedFilter,
    Pagination, ReportResultsFilterError,
};

fn compose(saved: Option<&str>, inline: Option<&str>) -> Result<String, ReportResultsFilterError> {
    compose_report_results_filter(
        &EntityId::new("report-1").expect("id"),
        saved,
        inline,
        Pagination::new(2, 3),
    )
}

#[test]
fn protected_scope_composes_saved_inline_and_window() {
    assert_eq!(
        compose(Some(" severity>5 "), Some("severity<9 OR severity=10")).expect("filter"),
        "severity>5 severity<9 OR severity=10 _and_report_id=report-1 first=4 rows=3"
    );
    assert_eq!(
        compose(None, Some("  ")).expect("filter"),
        "_and_report_id=report-1 first=4 rows=3"
    );
}

#[test]
fn both_inputs_reject_scope_and_pagination_overrides_quote_aware() {
    for key in [
        "report_id",
        "_report_id",
        "_and_report_id",
        "__and_report_id",
        "first",
        "rows",
        "_first",
        "_rows",
        "max",
        "offset",
        "limit",
        "page",
        "per_page",
        "ignore_pagination",
        "_ignore_pagination",
    ] {
        for operator in ["=", ">", "<", "~", ":"] {
            let fragment = format!("OR ({key} {operator} other)");
            for (saved, inline) in [
                (Some(fragment.as_str()), None),
                (None, Some(fragment.as_str())),
            ] {
                assert!(compose(saved, inline).is_err(), "{fragment}");
            }
        }
    }
    let literal = r#"name="a\" first=1 severity>=5" comment='rows=10 _and_report_id=x <=5'"#;
    assert!(compose(None, Some(literal)).is_ok());
    assert!(compose(None, Some(r#"name="literal" ROWS=2"#)).is_err());
}

#[test]
fn strict_comparators_do_not_change_legacy_escape_hatch() {
    for fragment in [
        "severity>=5",
        "severity<=5",
        "severity==5",
        "severity=>5",
        "severity!=5",
        "severity><5",
        "severity> =5",
        "severity>5<9",
        "severity>",
        "severity=",
    ] {
        assert!(
            matches!(
                compose(None, Some(fragment)),
                Err(ReportResultsFilterError::Comparator { .. })
            ),
            "{fragment}"
        );
        assert!(PaginatedFilter::new()
            .with_filter_string(Some(fragment))
            .build()
            .is_some());
    }
    for fragment in [
        "severity>5",
        "severity<9",
        "name~foo",
        "name:foo",
        r#"name="severity>=5""#,
        r#"comment='x <= 5'"#,
    ] {
        assert!(compose(None, Some(fragment)).is_ok(), "{fragment}");
    }
    assert_eq!(
        compose(Some(r#"name="oops"#), None),
        Err(ReportResultsFilterError::UnterminatedQuote)
    );
}

#[test]
fn composed_text_uses_existing_request_xml_escaping() {
    let request = GetResultsRequest {
        filter_string: Some(
            compose(Some("severity>5"), Some(r#"name="a&b" severity<9"#)).expect("filter"),
        ),
        ..GetResultsRequest::default()
    };
    let xml = String::from_utf8(request.encode(GmpVersion(22, 8)).expect("encode")).expect("utf8");
    assert!(xml.contains("severity&gt;5"));
    assert!(xml.contains("severity&lt;9"));
    assert!(xml.contains("a&amp;b"));
    assert!(xml.contains("_and_report_id=report-1"));
    assert!(!xml.contains("filt_id="));
}
