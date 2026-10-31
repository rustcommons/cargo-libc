# 0.2.1 release preparation (unreleased)

This is the release scope and checklist, not a publication announcement. The
manifest is prepared as 0.2.1 with registry-only producer dependencies. Publication
is gated on the checks below and the exact merged release revision.

## Intended release notes

- Cargo-libc-owned stdout no longer panics when a downstream reader closes early.
  Only `ErrorKind::BrokenPipe` is discarded at the write/flush boundary. The
  operation continues: failed scans, child exit codes and later command errors
  remain failures. Other stdout errors are reported, and stderr is unchanged.
- `test` and `run` forward arguments after Cargo's second `--` to the test harness
  or program. This fix is already on `main`; published 0.2.0 rejected those args.
- Help and documentation describe current provider and `cdylib` workflows.
- Shared provider stdout handling comes from the required `libc-abi-tools` 0.3.2
  fix. The manifest now requires at least 0.3.2; 0.3.0 is not sufficient for the
  complete provider-output behavior.

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

- [x] Independently verified all five producer 0.3.2 registry archives as non-yanked,
  with API/archive checksums and `.cargo_vcs_info.json` identifying merged producer
  `d404073e2329405e9e6f2dd1005fd8671e0d0f41`. Their packaged Rust sources match
  that commit, which has the same tested source as `53d3c6f`.
- [x] Set cargo-libc to 0.2.1 and raise the `libc-abi-tools` caret minimum to 0.3.2:
  complete delegated provider stdout handling requires that fix. Other direct
  minimums remain 0.3.0 because their consumed library behavior does not require
  the standalone CLI stdout fixes. The real lock selects all four consumed tools
  crates at 0.3.2; both glibc 0.1.0 registry entries/checksums are unchanged.
- [ ] Pass registry-only formatting, tests, Clippy, MSRV, source-package and bounded
  native integration checks with no patches or git/path dependency overrides.
- [ ] Install the unpacked proposed package with `cargo install --path ... --locked`
  into an isolated root using a clean Cargo home; exercise the installed command
  and all five executables. This is not yet a published registry installation.
- [ ] Merge the reviewed release after these gates pass. Verify the merged source,
  manifest and lock match the validated revision. Publish that exact revision.
- [ ] Verify the published 0.2.1 archive checksum and source identity, then create
  `v0.2.1` at the released commit without moving an existing tag. Verify docs.rs.
- [ ] In a clean x86_64 environment, run
  `cargo install cargo-libc --version 0.2.1 --locked` and exercise the installed
  helpers, provider output and C/CMake build/run/inspect paths. Record results
  before marking the release complete.

The prior source-path runs are not substitutes for registry-package or installed
release verification. Real-old-system qualification remains outside these gates.
