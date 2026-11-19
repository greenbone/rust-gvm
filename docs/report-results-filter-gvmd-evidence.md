# Protected report-result filtering (#871, API #741)

Public source pin: gvmd `f2d5c97bcb2d3bdeac879c865ebe9cf9c998f766`.

- `src/gmp.c:17453–17479` extracts `_and_report_id` and passes the resolved report as an independent `init_result_get_iterator` restriction. A normal `report_id` predicate does not provide this guarantee.
- `src/manage_sql_filters.c:get_join` and `filter_term_sql` use OR between ordinary predicates unless explicitly preceded by AND. The scope restriction must not be part of that expression.
- `src/manage_filter_utils.c:parse_column_relation` models single-character `=`, `>`, `<`, `~`, `:` relations. Inclusive/compound syntax is not a reliable numeric comparison.
- `filter_term_value` also recognizes an underscore-prefixed keyword alias; the strict composer reserves `_report_id`, `_first`, `_rows`, and `__and_report_id` as well as canonical controls.

## Additive typed API

`gvm_gmp::compose_report_results_filter(&EntityId, Option<&str>, Option<&str>, Pagination) -> Result<String, ReportResultsFilterError>` takes a report identity, resolved saved-filter term text, inline term text, and page/per-page window. It always returns nonempty scope/window text. Pass it as `GetResultsRequest.filter_string`, leaving `filter_id` unset: gvmd saved-filter selection replaces rather than augments inline terms. Both fragments are independently validated before appending protected scope and pagination. Boolean expression semantics are retained (including default OR); saved and inline text are not parenthesized or rewritten into a different grammar.

The new error type leaves exhaustive matches on existing `FilterFragmentError` source-compatible. Existing raw `PaginatedFilter` and request escape hatches remain permissive. The strict helper rejects compound or malformed comparisons outside quoted literals and unclosed quotes; it is not a complete gvmd grammar/field validator. Scope and pagination keys are rejected quote-aware, including conservative pagination aliases.

## Bounded verification

The stateful result mock models numeric severity/QoD predicates, equality fields, default/explicit OR, explicit AND groups, independent protected report restriction, counts and pagination. It deliberately does not implement the full gvmd filter engine (parentheses, NOT, regexp/approximate matching, arbitrary fields, user ACLs, or database behavior). Typed composition/encoding tests cover quoted literals and wire escaping separately. This is pinned-source-shaped deterministic qualification, not live gvmd validation.
