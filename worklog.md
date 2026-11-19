# Report projection stack worklog

- 2026-11-18: Read project guidance/issues #856/#857; verified hive-bumblebee identity. Clean worktree based on 125e16f.
- #856: Added optional lossless VT discriminator, backwards-compatible serde, realistic fixture and opt-in stateful attributes. Parser regression covers classic, CVE, future, absent and unchanged JSON metadata.
- Local toolchain pending parent provisioning; no host tool changes.
- #857: Reused NamedEntity and shared reference parser for optional report task scan_config; both scan/audit paths covered. Shared mock renderer resolves only existing configs, with config-less omission tests.
- First PR: https://github.com/greenbone/rust-gvm/pull/858 (head 21a9656). Local rustfmt passed; local tests blocked before crate build by missing native libc/libgcc_s linker inputs. Full CI selected instead of changing host tools.
