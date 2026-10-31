# 0.2.1 release preparation (unreleased)

This is a draft scope and checklist, not a release announcement. The manifest
version and production lock are unchanged. No package, tag or release is approved
by this document.

## Intended release notes

- Cargo-libc-owned stdout no longer panics when a downstream reader closes early.
  Only `ErrorKind::BrokenPipe` is discarded at the write/flush boundary. The
  operation continues: failed scans, child exit codes and later command errors
  remain failures. Other stdout errors are reported, and stderr is unchanged.
- `test` and `run` forward arguments after Cargo's second `--` to the test harness
  or program. This fix is already on `main`; published 0.2.0 rejected those args.
- Help and documentation describe current provider and `cdylib` workflows.
- Complete provider-output handling requires the shared `libc-abi-tools` fix.
  Published 0.3.0 still uses panicking writes inside delegated provider commands;
  cargo-libc's own output fix does not repair those dependency writes.

## Evidence and source integration gates

- [ ] Complete review of [fix PR #1](https://github.com/rustcommons/cargo-libc/pull/1).
- [x] Formatting, 71 unit tests, four process regressions, Clippy with warnings
  denied, help/output checks and source-package checks passed on the fixed source.
- [x] Record exact producer `53d3c6f0fa0fee5739e4021eb335017533396f06` and consumer
  `b8ff22a5b1cbb71f123b58c4efdd9be7614bfce4`. The consumer Rust source is unchanged
  from `5d5184c`; only the later-error test fixture and documentation changed.
- [x] Fresh [combined integration](COMBINED-STDOUT-INTEGRATION-0.3.2.md) passed,
  with exact path resolutions and a saved artifact. It covers the provider export
  failure after an earlier successful export, including continued side effects
  with closed stdout. The original listing fixture was incorrect and is superseded.
- [x] Producer Windows/Linux, MSRV and kernel checks passed at the recorded SHA,
  including provider/scanner output regressions. Temporary patches and refreshed
  locks stayed in the disposable workspace. Historical `d2f9abb` evidence does
  not qualify these stdout fixes.

The bounded native integration covers glibc 2.17/2.28, source providers, C
libraries, plain Cargo and host sysroots. Unchanged inputs do not require a broad
provider matrix for this fix. Python/container matrices and tier 3 real-target
qualification remain outside that run; do not claim them from it.

## Registry adoption and publication gates

1. Verify all four required 0.3.2 packages are actually published on crates.io:
   `libc-abi-tools`, `kernel-abi-tools`, `linux-abi-scan`, and `linux-targets`.
   Confirm their intended source/release identity. Source-path tests are not
   registry-package or packaging evidence.
2. Raise the `libc-abi-tools` minimum to **0.3.2** once published: complete provider
   broken-pipe behavior depends on its fix, so allowing 0.3.0 would be incorrect.
   Raise other minimums only where consumer behavior requires their fixes, with
   a recorded reason. Deliberately refresh the production lock from the registry;
   inspect every version, source and checksum change. Preserve the glibc data
   inputs unless an independent change is authorized.
3. Remove all candidate overrides and verify the registry-only dependency graph.
   Run required checks, the bounded consumer integration, and source-package
   verification. Validate `cargo install --path ... --locked` from the unpacked
   proposed package in a clean environment with an isolated install root; this
   is a local package-install check, not proof of an unpublished registry install.
4. Review the final 0.2.1 version/lock and release notes as a separate release
   preparation change. Obtain explicit publication authorization before merging
   release changes, creating tags/releases or publishing packages.
5. Only after authorized publication, verify
   `cargo install cargo-libc --version 0.2.1 --locked` in a clean environment and
   exercise the installed command/helpers. Confirm it resolves the released
   producer packages and includes the second-`--` and broken-pipe fixes.
