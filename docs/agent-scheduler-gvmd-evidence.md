# Structured agent schedules and scheduler delay (#851)

Evidence: gvmd `150a4cfad39d00f45ef147c35e1a5f27f415c8e9`,
`src/gmp_agent_control_scan_agent_config.c` (nested scheduler input),
`src/gmp_agents.c` (agent output), `src/gmp.c` (scanner defaults), and
`src/gmp_agent_groups.c` (group create/modify/read).

## Public contract

`AgentScriptExecutorConfig::scheduler_cron_time` now contains `AgentSchedule`
values with `cron: String` and `max_delay_seconds: Option<u32>` in requests and
responses. Existing string literals with `.into()` continue to work; callers
holding `Vec<String>` must map entries with `AgentSchedule::from`. Encoding
always emits `<item><cron>…</cron><max_delay_seconds>30</max_delay_seconds></item>`
(the delay child is omitted when `None`). gvmd defaults omitted delay to zero.
The previous text-only item encoding does not configure current gvmd correctly.
Legacy text-only response entries remain readable with no delay; there is no
legacy encoder or inferred server-version downgrade. Empty cron observations
remain empty rather than disappearing. Cron interpretation remains server-owned.

Agent-group create and modify expose `max_delay_seconds: Option<u32>` and reads
preserve the optional value. Both group mutations and agent/default scheduler
mutations validate the final request delay against `0..=2147483647` before
encoding or transport. On group modification, omitted delay **resets to zero**
in pinned gvmd; it does not preserve an earlier delay. Set the observed delay
explicitly when retaining it is intended.

`Scanner::agent_control_config_defaults` retains the optional `agent_defaults`
configuration and `update_to_latest`. The same scheduler parser handles agent
configurations and scanner defaults. Absent fields stay absent; absent/empty
schedule lists are empty vectors. Invalid numeric delays fail response decoding.

## Qualification

Independent source-shaped parser/encoder tests cover nested, legacy, empty,
zero, omitted and malformed values plus request integer bounds. Stateful mock
agent-group handling validates/persists delay and reproduces omitted-to-zero
semantics; typed client tests exercise create and modify round trips. The mock
does not execute schedules, synchronize an Agent Controller, or simulate jitter.
Scanner/agent schedule response tests are source-shaped fixtures, not live-gvmd
qualification. Existing GMP 22.8 capability gates remain unchanged.
