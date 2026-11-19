# Active Directory credentials and verification

Evidence: gvmd `150a4cfad39d00f45ef147c35e1a5f27f415c8e9` (origin/main,
2026-11-18), `src/schema_formats/XML/GMP.xml.in` credential create/modify/get
and `verify_active_directory_credential`; `src/gmp_credentials.c` and
`src/manage_credentials.c`. Implements #847. This is source parity, not live AD qualification.

## Canonical API

`CredentialType::ActiveDirectory` encodes `ad`. Set
`CreateCredentialRequest::active_directory` to `ActiveDirectoryCredentialFields`:
`host`, `bind_dn`, `bind_password`, `base_dn`, `tls_mode` are required for create
and inline verification. `port` and PEM `ca_certificate` are optional. Modes are
`ActiveDirectoryTlsMode::{Ldaps, StartTls}`; omitted ports remain omitted so gvmd
selects 636 or 389. Zero ports are rejected locally. XML escaping is structural.

`ModifyCredentialRequest::active_directory` accepts partial replacements; omission
preserves existing values and an explicit empty CA certificate clears it. Set clear_port on modify to restore
the TLS-mode port default; combining clear_port with a port is invalid. Credential
metadata exposes optional non-secret fields through `Credential::active_directory`.
`CredentialKind::ActiveDirectory` no longer classifies `ad` as unknown.

`VerifyActiveDirectoryCredentialRequest::{Saved { credential_id }, Inline(fields)}`
makes mixing saved and inline forms impossible. Execute it directly or through
`GmpClient::verify_active_directory_credential`. It returns a status-only typed
response; verification does not persist credentials.

## Availability, authorization and policy

No release/version minimum is inferred from source dates. Verification requires
positive XML-help command discovery (`discover_commands`), since gvmd's
`ENABLE_AD_INTEGRATION` build/runtime availability cannot be determined from GMP
22.x alone. Discovery is not proof of authorization or destination acceptance.
The server's explicit `verify_active_directory_credential` permission is also
required for AD create/copy/modify, including administrators; generic credential
operations retain their existing availability because their saved type may be
unknown locally. Callers should discover AD support before presenting AD workflows.
All server status errors, including feature-disabled, permission, TLS/LDAP and
destination-policy denials, are preserved. No local network or DN-policy checks
attempt to replace gvmd decisions.

Bind passwords are write-only in response models and redacted in request Debug
and client wire traces. Raw request bytes necessarily contain the supplied secret.
The built-in mock rejects AD verification/credential mutations rather than claiming
a successful bind or retaining AD secrets in its generic credential store. Explicit
fixture overrides and scripted scenarios provide deterministic outcomes; neither
connects to LDAP. Existing generic clone requests remain server-authoritative.

Tests: `test_ad_credentials` covers escaping, omitted/default ports, partial/clear
updates, required fields, metadata and status preservation. Client facade coverage
checks discovery on 22.4, missing advertisement on 22.8, and denial responses;
wire-trace tests check bind-password redaction. No live AD endpoint was used.
