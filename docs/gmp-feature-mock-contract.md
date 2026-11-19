# Recent GMP features: client API and mock contract

This is the feature-level contract for rust-gvm issues #847–857. The changes
are delivered as an incremental PR stack; they are not yet a released or merged
API. The support below describes deterministic test behavior, not live gvmd,
LDAP, Agent Controller, or scanner qualification.

## Feature coverage

| Issue / PR | Typed client contract | Mock behavior and boundary |
| --- | --- | --- |
| #850 / #860 | Sensor scanners accept port `0`; other scanner and relay ports remain nonzero. | Stateful create normalizes sensor ports; modify validates the effective type and retained port before mutation. Changing a normal scanner to a sensor without supplying a port preserves and rejects the old nonzero port. |
| #849 / #862 | Target create/modify and reads preserve optional `source`. | Stateful defaults, replacement/omission, copying, filtering and atomic invalid-update rejection are covered. This does not discover directory hosts. |
| #856 / #858 | `NvtRef::type_` retains optional, open-vocabulary VT type beside opaque `type_metadata`. | Stateful result resources accept seeded `nvt_type` and `type_metadata` attributes and XML-escape them. Missing values are not invented. |
| #857 / #859 | Structured scan/audit report tasks expose optional `scan_config: NamedEntity`. | The shared stateful report renderer includes resolvable configuration references and omits them for configless tasks. No client lookup fills missing report metadata. |
| #851 / #861 | `AgentSchedule { cron, max_delay_seconds }`, agent-group delay and scanner defaults are retained. | Group delay persists; omitted modify delay resets to zero. Missing/empty modify names and out-of-range delay fail before mutation. Source-shaped fixtures exercise nested agent/scanner schedules. Agent/controller mutation success is only a bounded acknowledgement, not config persistence, schedule execution or jitter simulation. |
| #852 / #863 | `GetAgentInventoryTokenRequest` returns a confidential token wrapper. | Fixture and authenticated stateful modes return a fixed nonfunctional token; fixtures model issuance failures. No signer, expiry or full ACL model is provided. |
| #853 / #864 | Agent reads expose `operating_system`, `architecture`, `os_cpe`, `os_status`. | Source-shaped fixture responses cover populated, empty and missing observations. The mock does not detect an OS. |
| #854 / #865 | Support bundles accept independent optional `anonymized` and `encryption` flags. | Wire history retains omitted/false/true requests; decoded payload remains inert fixed bytes. A successful mock response does not prove anonymization or encryption. |
| #855 / #867 | Application summaries retain optional `hosts_count`, `occurrences`, and `threat`; severity remains server text. | Stateful drill-down and source-shaped fixtures provide distinct counts with absent application IDs. No Agent aggregation or occurrence fallback algorithm is simulated. |
| #847 / #866 | AD credential create/partial modify, non-secret reads, saved/inline verification. | Built-in modes reject AD create/modify fields and verification. Every AD child, including host-only, port-only, `<port/>`, and CA-only updates, is guarded before generic CRUD. Explicit fixture overrides and scenarios can supply outcomes. |
| #848 / #868 | AD object discovery preserves typed counts, truncation, optional expansions and task/config references. | Built-in modes reject live discovery. Explicit fixture/scenario responses model collections and errors without creating targets or performing LDAP queries. |

For exact public values, defaults and pinned source evidence, see
[scheduler](agent-scheduler-gvmd-evidence.md),
[inventory token](agent-inventory-token-gvmd-evidence.md),
[agent metadata and bundles](agent-request-gvmd-evidence.md),
[report applications](report-request-gvmd-evidence.md#application-count-projection-855),
[VT/report migration](v0.11.0-migration.md#result-vt-discriminator-856),
[AD credentials](active-directory-credential-gvmd-evidence.md), and
[AD discovery](active-directory-discovery-gvmd-evidence.md).

## Mock modes and availability

- **Echo** acknowledges recognized commands but is not a state/payload model.
  Deliberately unsupported AD operations are exceptions: they return an error.
- **Fixture** serves deterministic source-shaped responses. Explicit builder
  `.fixture(command, response)` overrides take precedence over built-in AD
  rejection, allowing success, disabled-build, denied-policy and error cases.
- **Stateful** requires authentication and uses shared command/version gates.
  Resource seeds can supply response observations; this does not imply that
  every acknowledged mutation persists every nested field.
- **Scenario** scripts exact outcomes rather than executing external operations.
  Strict matching is preferable when validating a multi-command flow.

Agent commands require GMP 22.8. Inventory token issuance additionally requires
positive XML-help discovery; AD verification/discovery also require discovery,
not a guessed minimum version. Explicit fixtures/scenarios are programmable test
inputs, not evidence that a deployment enables a feature. Follow the
[support matrix](SUPPORT_MATRIX.md) and server permission/policy responses.

Secrets need explicit handling: token and bind-password Debug output and client
wire tracing are redacted, but explicit serialization/raw payload access is not
a safe logging surface. Use dummy credentials/tokens in fixtures. Do not use
inert mock support bundles to test real privacy or cryptography guarantees.

## Regression entry points

- [Sensor, target, result and report conformance suites](../crates/gvm-mock-server/tests/)
- [Typed mock/client facade coverage](../crates/gvm-client/tests/typed_facade_coverage.rs)
- [Versioned agent/bundle calls](../crates/gvm-client/tests/versioned_client.rs)
- [Inventory discovery/error/redaction tests](../crates/gvm-client/tests/agent_inventory_token.rs)
- [Structured report config parser tests](../crates/gvm-gmp/tests/report_scan_config.rs)
- [Agent scheduler and delay tests](../crates/gvm-gmp/tests/agent_scheduler_test.rs)

Run socket-level tests explicitly (they are opt-in):

```console
cargo test --workspace --features gvm-client/unix-socket-tests,gvm-mock-server/unix-socket-tests,gvm-gmp/serde
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## REST/API boundary

These PRs implement the **GMP library and mock**, not downstream REST routes.
The corresponding rust-gvm-api work remains in
[#727–#736](https://github.com/greenbone/rust-gvm-api/issues?q=is%3Aissue+is%3Aopen+created%3A2026-11-18).
Its design contracts and eventual generated OpenAPI must consume these typed
requests/responses, never introduce gateway-local GMP XML parsing. In particular,
no new scanner write endpoint or audit/delta REST surface is implied here.
