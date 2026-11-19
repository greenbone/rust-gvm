# Agent inventory token (#852)

## Evidence and contract

Pinned public gvmd [handler at 150a4cfad39d00f45ef147c35e1a5f27f415c8e9](https://github.com/greenbone/gvmd/blob/150a4cfad39d00f45ef147c35e1a5f27f415c8e9/src/gmp_agent_inventory_authentication.c#L49)
and [current schema](https://github.com/greenbone/gvmd/blob/150a4cfad39d00f45ef147c35e1a5f27f415c8e9/src/schema_formats/XML/GMP.xml.in)
implement the no-argument `<get_agent_inventory_token/>` command. Success is
`200` with one `<token>` text child; XML entities are decoded without treating
the token as a JWT or interpreting its contents. Permission is `get_agents`,
not a new command-specific permission. Denial is **400** (`Permission denied`),
not 403. Agent-disabled builds return **503** (`Command unavailable`);
unavailable issuance returns **503** (`Agent inventory token issuance is unavailable`);
internal generation failures return **500**. The client preserves server status
and message before attempting successful-response decoding.

## Rust API

`GetAgentInventoryTokenRequest` is a unit request with no inputs. It binds to
`GetAgentInventoryTokenResponse`; `GmpClient::get_agent_inventory_token(request)`
delegates to canonical execution. As with other new commands introduced without
a version bump, call `discover_commands()` first. The minimum is the agent-enabled
GMP 22.8 family, but version alone is not evidence of command availability.
The older repository schema snapshot remains unchanged; the registry explicitly
qualifies this command as `PublicSourceOnly`.

`response.token.as_str()` deliberately exposes the confidential token for use.
Both token and enclosing response Debug output redact it. Opt-in client wire
tracing already redacts `<token>` and is exercised end-to-end by the tests.
The optional serde implementation serializes the actual token as a string for
explicit application use; serialization and raw protocol access are not safe
logging surfaces.

## Mock and verification boundary

Fixture and authenticated stateful modes return the same deterministic,
nonfunctional `mock-agent-inventory-token`. Stateful help advertises the command
for 22.8; fixture-mode callers supply a positive XML help override when testing
discovery. Fixture overrides cover disabled,
unavailable, denied and server-error cases. This is not a token signer, expiry
model or full ACL simulation. Tests cover exact no-argument encoding, escaped
payload decoding, missing payload, capability discovery, typed client failures,
and Debug/wire-trace redaction. These checks are not live gvmd qualification.
