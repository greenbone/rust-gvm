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
