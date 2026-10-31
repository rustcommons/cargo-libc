# cargo-libc 0.2.1

Published on 2026-10-31. [crates.io](https://crates.io/crates/cargo-libc/0.2.1) and
[docs.rs](https://docs.rs/cargo-libc/0.2.1/cargo_libc/) serve the released version.
Tag [`v0.2.1`](https://github.com/rustcommons/cargo-libc/tree/v0.2.1) points
to released commit `767c16aef0e49c37018b3907be480363dfdb9175`, the normal merge of
[PR #1](https://github.com/rustcommons/cargo-libc/pull/1).

The official crate archive SHA-256 is
`715e9c7eee7896a0e5055df1d67cdc4df7d4a57670f1d769562bcc19f2916398`.
It matches the registry API. Its clean `.cargo_vcs_info.json` identifies the
released commit; packaged Rust source and lock match that commit. The normalized
manifest has version 0.2.1 and `libc-abi-tools` minimum 0.3.2. All four consumed
tools crates are locked to registry 0.3.2; both glibc inputs remain 0.1.0.

## Published installation verification

[Run 41245509023](https://github.com/rustcommons/cargo-libc/actions/runs/41245509023)
passed on Ubuntu 24.04 x86_64 / Rust 1.90.0, using a new Cargo home and isolated
install root:

```sh
cargo install cargo-libc --version 0.2.1 --locked --root "$INSTALL_ROOT"
bash scripts/smoke-installed.sh "$INSTALL_ROOT" /usr/bin/clang-18 0.2.1
```

The downloaded log confirms registry installation of 0.2.1 and the four 0.3.2
producer packages. All five executables were present. Installed help/version,
host ELF inspection and provider stdout regressions passed. The installed linker,
runner and C wrappers built and ran the glibc 2.17 C/CMake fixture, inspected its
requirements, verified the provider and refused target C++ as expected.

[Artifact `published-install-41245509023`](https://github.com/rustcommons/cargo-libc/actions/runs/41245509023/artifacts/13059327372)
contains the saved log; digest
`sha256:366d24c67718286e7b15932035e94aec041872f442ce187422bc727113453b61`.
Docs.rs [status](https://docs.rs/crate/cargo-libc/0.2.1/status.json) reported
`doc_status: true`, its versioned API page returned HTTP 200, and
[build 5100697](https://docs.rs/crate/cargo-libc/0.2.1/builds/5100697) succeeded.

## Release notes

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

- [x] Release changes prepared in [fix PR #1](https://github.com/rustcommons/cargo-libc/pull/1).
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

[Registry-only validation evidence](REGISTRY-VALIDATION-0.2.1.md) records the
passing native run and clean unpacked-package installation.

- [x] Independently verified all five producer 0.3.2 registry archives as non-yanked,
  with API/archive checksums and `.cargo_vcs_info.json` identifying merged producer
  `d404073e2329405e9e6f2dd1005fd8671e0d0f41`. Their packaged Rust sources match
  that commit, which has the same tested source as `53d3c6f`.
- [x] Set cargo-libc to 0.2.1 and raise the `libc-abi-tools` caret minimum to 0.3.2:
  complete delegated provider stdout handling requires that fix. Other direct
  minimums remain 0.3.0 because their consumed library behavior does not require
  the standalone CLI stdout fixes. The real lock selects all four consumed tools
  crates at 0.3.2; both glibc 0.1.0 registry entries/checksums are unchanged.
- [x] Pass registry-only formatting, tests, Clippy, MSRV, source-package and bounded
  native integration checks with no patches or git/path dependency overrides.
- [x] Install the unpacked proposed package with `cargo install --path ... --locked`
  into an isolated root using a clean Cargo home; exercise the installed command
  and all five executables. Published registry installation is separately verified above.
- [x] Merged PR #1 normally. The merged source, manifest, lock and scripts match
  the validated revision. Published exact merge commit `767c16a`.
- [x] Verified the published archive checksum/source identity and docs.rs. Created
  and remotely verified `v0.2.1` at the released commit; no tag was moved.
- [x] Clean x86_64 registry installation and installed-helper/provider/C-CMake
  checks passed in run 41245509023, as recorded above. Release verification is
  complete for this scope.

The prior source-path runs are not substitutes for registry-package or installed
release verification. Real-old-system qualification remains outside these gates.
