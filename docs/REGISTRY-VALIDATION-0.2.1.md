# cargo-libc 0.2.1 registry release validation

Historical evidence for the exact revisions/version below. Current release and
registry state: [cargo-libc 0.2.2](RELEASE-0.2.2.md); current constraints and gaps:
[requirements index](REQUIREMENTS.md).

[Run 41244617689](https://github.com/rustcommons/cargo-libc/actions/runs/41244617689)
passed on Ubuntu 24.04 x86_64 at consumer
`c7fcd04312743f27f6a035df49530fb6f2ef5097`, using Rust/Cargo 1.90.0. This run used
registry packages only: no producer checkout, source patches, git overrides or
lock refresh in the validation workspace. Subsequent report-only commits preserve
the tested source, manifest, lock and validation scripts.

## Registry identity

All five producer 0.3.2 archives were independently downloaded from crates.io.
They were non-yanked, their archive hashes matched the registry API, and their
`.cargo_vcs_info.json` identified
`d404073e2329405e9e6f2dd1005fd8671e0d0f41`. Packaged Rust source matched that
commit, which is source-equivalent to tested candidate `53d3c6f`.

The saved metadata and production lock resolve exactly these four consumed
packages from `registry+https://github.com/rust-lang/crates.io-index`:

| Package | Version | Archive SHA-256 |
| --- | --- | --- |
| `libc-abi-tools` | 0.3.2 | `079327b25d8d9089e4dc20dc09de4d3dcc0fb2762b55166854c7787bc33db9fc` |
| `kernel-abi-tools` | 0.3.2 | `a9da5a4b7dedfb152d6635d7de87efa22a92b1fd6f496ec351caa6d3bbdfd209` |
| `linux-abi-scan` | 0.3.2 | `d2ea34b1aa4ff30bc553fd4a7061da0a3f40914d6e75394a2c1391c6d947bff4` |
| `linux-targets` | 0.3.2 | `a9e3ec31c779ff773874ca7541dbabc85868aa965af2c54f951516ea5699c104` |

The umbrella `linux-abi-tools` archive was verified but is not a consumer
dependency. Both glibc input crates remain at 0.1.0 with unchanged checksums.
The `libc-abi-tools` caret minimum is 0.3.2 because shared provider output requires
its fix; other library minimums remain unchanged. The package version is 0.2.1.

## Passed gates

- Formatting, MSRV/all-targets check, 71 unit tests, four process regressions and
  Clippy with warnings denied, locally and in the hosted registry-only run.
- Shared provider stdout: normal output, closed reader, genuine non-pipe errors,
  and a later export failure after an earlier successful export. Side effects
  continued and failure remained exit 1.
- Glibc 2.17/2.28 source-provider preparation, reuse, verification, build/run/inspect,
  tier 1 scanning and local tier 2 execution; too-new-symbol and C++ refusal.
- C/CMake target compilation and native host separation; clippy/doc/rustc,
  local fixture installation, both plain-Cargo formats and host-sysroot checks.
- `cargo package --locked --offline`, followed by installation of the unpacked
  package with `cargo install --path ... --locked --offline` into an isolated
  root with a clean Cargo home. All five installed executables were checked.
  Installed help/version/inspect/provider output passed, as did a real C/CMake
  build/run/inspect through the installed linker, runner and C wrappers, plus
  the installed C++ refusal path.

[Artifact `candidate-integration-41244617689`](https://github.com/rustcommons/cargo-libc/actions/runs/41244617689/artifacts/13058462392)
contains resolution metadata, the unchanged production manifest/lock, logs,
tool versions and the proposed package. Its digest is
`sha256:f0fd056fd67de4d87769f98286b5041e898b43454bf8dc1eace5b2aa58ff5b48`.
The saved package's Rust sources were compared with the reviewed tree.

## Publication and limits

These prepublication gates passed before normal merge and publication of exact
release commit `767c16aef0e49c37018b3907be480363dfdb9175`. The
[release record](RELEASE-0.2.1.md) now verifies the official archive/checksum/VCS
identity, exact `v0.2.1` tag, docs.rs and clean published registry installation.

The earlier [combined source report](COMBINED-STDOUT-INTEGRATION-0.3.2.md) remains
historical evidence, including the corrected fixture-only failure. This run adds
actual registry and unpacked-package installation evidence. It does not rerun
Python/container matrices, the full glibc matrix or tier 3 real-old-system
qualification. Local runtime execution used host glibc 2.39.
