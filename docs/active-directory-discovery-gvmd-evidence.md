# Active Directory object discovery: gvmd evidence

Issue: [#848](https://github.com/greenbone/rust-gvm/issues/848).

## Source pin

Reviewed local `origin/main` of greenbone/gvmd at
`150a4cfad39d00f45ef147c35e1a5f27f415c8e9` (2026-11-18).

- [GMP schema, search and collection response](https://github.com/greenbone/gvmd/blob/150a4cfad39d00f45ef147c35e1a5f27f415c8e9/src/schema_formats/XML/GMP.xml.in#L10916)
- [Request validation, reference expansion and response rendering](https://github.com/greenbone/gvmd/blob/150a4cfad39d00f45ef147c35e1a5f27f415c8e9/src/gmp_active_directory.c#L395)
- [Management policy and LDAP discovery](https://github.com/greenbone/gvmd/blob/150a4cfad39d00f45ef147c35e1a5f27f415c8e9/src/manage_active_directory.c)
- [Maximum search results](https://github.com/greenbone/gvmd/blob/150a4cfad39d00f45ef147c35e1a5f27f415c8e9/src/manage_active_directory.h#L128)

This is source/schema qualification, not a live AD test or proof of availability
in a historical GMP release. The command requires both the AD build option and
the runtime integration feature. Version numbers alone do not establish support;
capability registration/discovery owns availability.

## Canonical request

`GetActiveDirectoryObjectsRequest` requires a saved credential identifier.
All other controls are optional and omitted on the wire unless set:

| Input | Evidence-backed semantics |
| --- | --- |
| scope | base, onelevel, subtree; server default onelevel |
| max_results | integer 1 through 1000; omitted retains server default |
| include_disabled_computers | boolean; default true |
| targets | boolean; default false |
| tasks | boolean; default false; true requires targets=true |
| details | boolean; default false |
| base_dn | nonempty when supplied; otherwise credential base DN |
| group_dn | nonempty when supplied; selects explicit direct group members |
| object_types | nonempty selection of ou/container/computer/group; default excludes container |

The request validates range, empty supplied selectors and the tasks dependency.
Repeated object types are emitted once, matching gvmd's bitmask interpretation.
It does not normalize DNs, attempt containment comparisons, resolve destinations,
or reinterpret saved credential settings. Those remain server policy.
The group DN can be outside a narrower search base while both must be within the
credential base DN. DN value case differences can cause server rejection.
This operation returns live results; it does not create or persist targets.

## Latest collection response

The current envelope is `objects/@truncated`, containing `count` and repeated
`object` children, not a flat list of objects beside a generic GMP count.

`count` has `ou`, `container`, `group`, and nested
`computer/total`, with optional `computer/disabled`. Counts describe the fetched
subset, not global directory totals. Disabled count omission is preserved.

Each object has a typed `type` plus name, distinguished_name, parent_dn and
description; the latter two can be empty. Computers can add DNS hostname, OS,
OS version, SPNs, disabled flag and target references. The source can omit the
disabled flag if userAccountControl is unavailable or invalid.

SPNs, targets and nested tasks use `Option<Vec<_>>` so omission and requested
empty expansions remain distinguishable. Targets and tasks retain identifiers
and names. A task's config reference is optional for unreadable/trash configs.
The schema-optional objects envelope is also represented as an option. Truncated
empty collections retain both the flag and counts.

## Verification boundary

`crates/gvm-gmp/tests/test_active_directory.rs` covers canonical wire encoding,
all selectors and flags, validation, the latest container-aware collection,
nested computer counts, escaped DNs, missing versus empty expansions, references,
malformed structural values, and source-shaped feature-disabled, permission and
destination-policy errors. Server status and message are retained through the
shared error parser. These deterministic fixtures do not qualify a live LDAP
connection, TLS, authorization, or the server's destination allow lists.
