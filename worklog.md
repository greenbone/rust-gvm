# Report projection stack worklog

- 2026-11-18: Read project guidance/issues #856/#857; verified hive-bumblebee identity. Clean worktree based on 125e16f.
- #856: Added optional lossless VT discriminator, backwards-compatible serde, realistic fixture and opt-in stateful attributes. Parser regression covers classic, CVE, future, absent and unchanged JSON metadata.
- Local toolchain pending parent provisioning; no host tool changes.
- #857: Reused NamedEntity and shared reference parser for optional report task scan_config; both scan/audit paths covered. Shared mock renderer resolves only existing configs, with config-less omission tests.
- First PR: https://github.com/greenbone/rust-gvm/pull/858 (head 21a9656). Local rustfmt passed; local tests blocked before crate build by missing native libc/libgcc_s linker inputs. Full CI selected instead of changing host tools.
# Agent GMP stack — 2026-11-18

- Read repository instructions, code map, roadmap/status, issues #851–#855 and pinned gvmd evidence. Isolated worktree from origin/main 125e16f.
- #851: implementing nested scheduler entries, shared delay validation, scanner defaults and agent-group delay round trips. Legacy text entries accepted on decode only; current nested wire always emitted.
- Parallel bounded implementation of #852 and #853–#855 in independent worktrees; changes will be reviewed and cherry-picked in required order.
- Native Rust toolchain pending workspace provisioning; no host packages installed.
- #851 local verification: workspace unit/integration tests with `gvm-client/unix-socket-tests` passed (including group round trips and rollback); initial doc test failed because default temp is noexec. Re-ran `cargo test --workspace --doc` with workspace TMPDIR and linker flags: passed. Strict workspace all-targets clippy with socket-test features passed; cargo fmt/diff checks passed. GMP group parser recheck passed after final assertions.
- Tooling: sourced /workspace/.cache/rust-tooling/env; RUSTFLAGS/RUSTDOCFLAGS `-C linker=rust-lld -L native=/workspace/.cache/rust-tooling/native-libs`, TMPDIR `/workspace/.cache/rust-tooling/tmp`. No host changes.
- #852 cherry-picked and reviewed: redacted token wrapper, discovery gate, deterministic mock and inventories. Stacked focused tests passed: 4 socket-client tests and 3 GMP tests. PR #863 based on #861.
- #853 cherry-picked and reviewed: four optional OS fields plus independent populated/absent/empty fixture. Twelve relevant parser tests passed. PR #864 based on #863.
- #854 cherry-picked and reviewed: optional anonymization with all nine encryption/anonymization combinations; command tests and 16 versioned-client tests passed. PR #865 based on #864.
- #855 cherry-picked and reviewed: application counts/threat preserve server severity; independent malformed/absent/empty/zero fixtures and client/mock drill-down test passed. Full final-stack workspace run underway.
- #851 full CI passed at 8668795 (run 44398663891), including all features, stable/MSRV, strict clippy, coverage, documentation and python-gvm integration. Upper PR CI dispatched explicitly because pull_request branch filters exclude stacked bases.
- Final combined local verification: `cargo test --workspace --features gvm-client/unix-socket-tests` passed 1,585 tests including doctests (one pre-existing ignored doctest); all 121 target summaries passed. Strict clippy found one test-only missing semicolon in #854; repaired in that layer and rebased #855. Strict workspace all-targets clippy and fmt now pass. No behavior changed by the lint repair.
# AD stack worklog

- 2026-11-18: Read #847/#848 and repository guidance; source evidence from gvmd origin/main. Isolated worktree from 125e16f. GitHub skill used, identity hive-bumblebee.
- #847 credential/verification layer in progress; #848 new-file discovery layer delegated independently. Commits and PRs remain sequential.
- No local Rust initially; parent arranging workspace-local toolchain. No host installs.

- #847 implementation complete: typed AD credentials/metadata, mutually exclusive verification, discovery gating, redaction, and bounded mock rejection.
- Focused checks passed: AD protocol (4), existing credentials (9), enums (31), AD facade integration (1). Full workspace unit/integration tests passed with unix-socket-tests; initial doctest run hit /tmp noexec, then all workspace doctests passed with TMPDIR under target and workspace-local rust-lld flags.
- Strict Clippy with unix-socket-tests passed, as did formatting/diff checks. Bare default all-target Clippy trips pre-existing cfg-disabled test-crate missing-doc warnings; enabling integration tests resolves them. All-feature Clippy blocked locally by absent native cc; CI required for native TLS/SSH dependencies.

- #847 published as PR #866, commits ed18e32 and 1523673. Evidence hash corrected against immutable git show origin/main (shared gvmd checkout is older). Current source permission and destination policy rechecked.
- #848 restored independent discovery layer, reviewed against immutable current schema/source, wired exports, discovery-only capabilities, negative built-in mock, inventories and client integration coverage. Preserves current nested counts and optional collections/references.
## 2026-11-19 — PR #866 review
- Review at f832a831: AD partial updates containing only host, port, clear-port or CA bypass built-in mock rejection.
- Clean review worktree; adding table-driven typed-client coverage for all eight AD update shapes across Echo/Fixture/Stateful. Stateful test seeds a saved AD credential, authenticates, and checks complete stored snapshots remain unchanged despite accompanying ordinary-field mutations.
- Tooling uses workspace-local Rust/linker/native libraries and existing AD target cache; no host changes.
- Confirmed regression fails on original code with false 200 OK, then passes for all 24 mode/update combinations after including host/port/CA children.
- Full workspace socket-enabled + serde verification: 1,614 tests passed across 124 targets, zero failures, one pre-existing ignored doctest. Strict workspace/all-target Clippy passes after extracting the test case table; formatting/diff checks pass.

## 2026-11-19 — PR #861 review
- Confirmed pinned gvmd requires nonempty modify-agent-group name before processing delay.
- Constructor now takes a name; validation/encoding reject missing or empty names. Updated callers and migration guidance.
- Stateful mock rejects nameless updates before mutation; regression compares complete response before/after rejected schedule, delay, comment, and membership changes.
- Optional serde accepts legacy cron strings and current objects while retaining structured serialization, with malformed-input and round-trip tests.
- Verification: workspace tests with serde and both Unix-socket features passed 1,599 tests across 122 targets (zero failures, one pre-existing ignored doctest). Strict workspace/all-target Clippy with those features passed. Formatting and diff checks passed. No live-server qualification.

## 2026-11-19 — PR #866 review
- Review at f832a831: AD partial updates containing only host, port, clear-port or CA bypass built-in mock rejection.
- Clean review worktree; adding table-driven typed-client coverage for all eight AD update shapes across Echo/Fixture/Stateful. Stateful test seeds a saved AD credential, authenticates, and checks complete stored snapshots remain unchanged despite accompanying ordinary-field mutations.
- Tooling uses workspace-local Rust/linker/native libraries and existing AD target cache; no host changes.
- Confirmed regression fails on original code with false 200 OK, then passes for all 24 mode/update combinations after including host/port/CA children.
- Full workspace socket-enabled + serde verification: 1,614 tests passed across 124 targets, zero failures, one pre-existing ignored doctest. Strict workspace/all-target Clippy passes after extracting the test case table; formatting/diff checks pass.

## 2026-11-19 — bounded built-in fixture completion
- Isolated branch/worktree from 2ee5789; parent checkout untouched.
- Added crate-local built-in agent OS/scheduler and report-application fixtures from existing source-shaped parser cases; GMP 22.8 scanner fixture adds optional Agent Control defaults while retaining ordinary scanner absence.
- Added typed Unix-socket coverage for built-in decoding, legacy omission/empty/zero distinctions, client version gating versus direct raw fixture playback, inert support-bundle options/history, and seeded unknown VT type/opaque metadata.
- Verification: 5 new socket tests passed; affected client + mock crate suites with both Unix-socket features passed (748 tests across 67 targets, zero failures). Strict Clippy for the new integration target, cargo fmt --all -- --check, and git diff --check passed. Tooling: /workspace/866-test-env and shared target cache. No LDAP, signing, scheduler execution/persistence, or archive transformations implemented.

## 2026-11-19 — mock/docs/spec completion
- Audited all eleven feature slices. Added built-in agent OS/nested scheduler, scanner-default, and application-count fixtures with typed socket tests, bundle flag history/inert-byte checks, and opaque VT metadata decoding.
- Added scan/audit optional config transport regression plus direct AD rejection and explicit scenario-success coverage. Live LDAP/scheduling/signing/anonymization remain deliberate boundaries.
- Integrated both #861/#866 review corrections through all six upper branches without rewriting published history; exact branch heads saved in review-restack.json.
- Full combined workspace verification with serde and socket tests: 1,634 passed across 128 targets, zero failures, one pre-existing ignored doctest. Strict socket-enabled workspace/all-target Clippy, formatting and diff checks passed.
- Documentation matrix links APIs, feature evidence and mock limits; downstream REST contract supplement remains explicitly planned, not an implemented OpenAPI claim.
