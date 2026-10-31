# Combined stdout-fix integration with linux-abi-tools 0.3.2

On 2026-10-31, [run 41243277747](https://github.com/rustcommons/cargo-libc/actions/runs/41243277747)
passed every step of the manual candidate integration workflow, including the
shared-provider stdout regression and bounded native x86_64 checks.

| Input | Exact revision |
| --- | --- |
| Producer, `rustcommons/linux-abi-tools` | [`53d3c6f0fa0fee5739e4021eb335017533396f06`](https://github.com/rustcommons/linux-abi-tools/commit/53d3c6f0fa0fee5739e4021eb335017533396f06) |
| Consumer, `rustcommons/cargo-libc` | [`b8ff22a5b1cbb71f123b58c4efdd9be7614bfce4`](https://github.com/rustcommons/cargo-libc/commit/b8ff22a5b1cbb71f123b58c4efdd9be7614bfce4) |

Consumer Rust source, manifest and production lock are identical to fix revision
`5d5184c5d8521fba50191716d0f1e38724ba04f4`. The intervening commit corrected a test
fixture and its checklist wording. Subsequent evidence-only documentation does
not change the tested source.

## Resolution and saved evidence

The workflow used temporary `[patch.crates-io]` paths and refreshed only a
disposable consumer lock. Downloaded `metadata.json` and `Cargo.lock` confirm
exactly one resolution for each package below, all **0.3.2** with no registry source:

| Package | Manifest under `/home/runner/work/cargo-libc/cargo-libc/candidate/` |
| --- | --- |
| `libc-abi-tools` | `Cargo.toml` |
| `kernel-abi-tools` | `crates/kernel-abi-tools/Cargo.toml` |
| `linux-abi-scan` | `crates/linux-abi-scan/Cargo.toml` |
| `linux-targets` | `crates/linux-targets/Cargo.toml` |

`glibc-abilists` and `glibc-provider-sources` remain registry **0.1.0** dependencies;
their full lock entries and checksums match production. Production manifests,
locks and the existing CI workflow were unchanged.

[Artifact `candidate-integration-41243277747`](https://github.com/rustcommons/cargo-libc/actions/runs/41243277747/artifacts/13057974492)
contains commit identities, resolution metadata, the disposable manifest/lock,
tool versions and all check logs. Its digest is
`sha256:c1dd5f3b700866ae0353f12e6f7d68eb92a155ed9e61b3dfe8b43f3e5045ed84`.

## Checks and results

The runner was Ubuntu 24.04 x86_64 with Rust/Cargo 1.90.0, Clang 18.1.3,
GCC 13.3.0, binutils 2.42 and host glibc 2.39. Build workers and Rust test threads
were capped at two. All these commands passed in the disposable consumer:

```sh
cargo fmt --all -- --check
cargo test --locked --offline
cargo clippy --all-targets --locked --offline -- -D warnings
python3 scripts/check-provider-stdout.py target/debug/cargo-libc
bash scripts/smoke-providers.sh /usr/bin/clang-18 2.17 2.28
bash scripts/smoke-c-deps.sh /usr/bin/clang-18 2.17 2.28
bash scripts/smoke-commands.sh /usr/bin/clang-18 2.17 2.28
CARGO_LIBC_SMOKE_CC=/usr/bin/clang-18 bash scripts/smoke-linux.sh
```

- **71 unit tests and four process regressions passed.** Owned help/version and
  provider-help output, inspect text/JSON, direct and scan failure statuses,
  ordinary command errors and non-pipe stdout errors were covered.
- **Shared provider output passed.** Normal and closed-output listing succeeded;
  `/dev/full` remained a failure. A two-entry fake-cache export first established
  the normal baseline: one successful export emitted its path, then the next
  failed on a missing descriptor. With stdout closed before launch, both entries'
  manifest-copy side effects still occurred and the later error retained exit 1.
  No compiler or real provider generation was needed for this regression.
- **Source providers 2.17/2.28 passed** preparation, verification, linking,
  cached reuse, inspection, tier 1 scans, local tier 2 execution and too-new-symbol
  refusal. C/CMake target headers and native host C remained separated, and target
  C++ was refused.
- **Cargo workflows passed:** clippy/doc/rustc, local fixture installation,
  linker-override refusal, shell and TOML plain-Cargo configurations, separate
  inspection and the `memfd_create` boundary. The host-sysroot checks also passed
  artifact matching, cached output, paths with spaces, JSON reports, local runner
  diagnosis, check diagnostics and C dependencies.

The producer's own [run 41243021989](https://github.com/rustcommons/linux-abi-tools/actions/runs/41243021989)
also passed at `53d3c6f`: Windows/Linux Rust checks, MSRV and kernel simulation.
Its provider tests exercise continued export side effects and later failures;
its scanner tests cover closed output. These are separate producer checks, not
additional consumer platform qualification.

## Superseded evidence and limits

[Run 41243145475](https://github.com/rustcommons/cargo-libc/actions/runs/41243145475)
at consumer `5d5184c` failed only the original provider regression fixture. It
incorrectly expected listing a regular-file cache root to fail; `cache::list`
intentionally treats a missing target directory as an empty cache. Both normal
and closed listing returned success. Native steps were skipped. The corrected
export fixture and passing run above supersede that test-only failure; no product
cache behavior was changed and no product regression was found.

The earlier [d2f9abb integration](CANDIDATE-INTEGRATION-0.3.2.md) remains historical
evidence for its recorded revisions, but does not qualify the new stdout fixes.
This report supersedes it for the current source-candidate adoption decision.

This is source-path integration, not registry adoption or validation of
`cargo install cargo-libc --locked` against unpublished crates. Python/cdylib
packaging, container modes, the full release matrix and tier 3 real-target
qualification were not rerun. Local execution used host glibc 2.39; tier 1/2 do
not qualify old runtimes or kernels.

Remaining gates are review, actual producer registry publication, a
`libc-abi-tools >=0.3.2` requirement, deliberate registry-lock refresh, unpatched
package/locked-install checks and separate release authorization. Follow the
[0.2.1 checklist](RELEASE-0.2.1.md). No version bump, package publication, merge,
tag or release was performed by this integration.
