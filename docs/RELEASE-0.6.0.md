# cargo-libc 0.6.0 qualification and publication

## Scope and authorization

The maintainer authorized the next minor release on 2026-11-07 after merging
the duplicate value-option fix in PR22. The release tightens six previously
last-value-wins options; [user notes](releases/v0.6.0.md) describe migration.
Implementation `fdf487fa6c13d99eb7acaa479e031528aa502910` was merged as
`219027a17cb9e35d82b0d6e1d56862bd68f001a5`. Only the root package version/lock
entry changes for 0.6.0; the six registry dependencies and checksums remain
unchanged. No producer release, LLVM wiring or new platform support is included.

## Existing source evidence

- Windows Rust 1.99: 100 library tests and one portable process test, formatting,
  all-target Clippy and rustdoc with warnings denied passed.
- Linux/WSL Rust 1.90: 105 library and ten process tests passed; zero doctests.
- The real-CLI process regression failed before the fix, then passed. Tests cover
  identical/different duplicates, defaults, switches and forwarding boundaries.
- [Exact source docs CI](https://github.com/rustcommons/cargo-libc/actions/runs/42425732808)
  and [merged docs CI](https://github.com/rustcommons/cargo-libc/actions/runs/42425764541)
  passed. The unchanged native/provider/container matrix was not repeated.

## Candidate qualification

Exact candidate `1d74aea48f686654bebdb62e2a25ac6676260a74` passed native
Rust 1.90 tests (105 library, ten process, zero doctests), registry-only metadata,
verified `cargo package --locked --offline`, and clean Cargo-home unpacked
`cargo install --path ... --locked --offline` into an isolated root.

The 28-file candidate archive is 85,623 bytes, SHA256
`6b7b1e7b35d1f3799d78a51461ed4b47e226a60bbce3df83d46ca0d08885c046`,
with clean exact-candidate VCS identity. Comparison with official 0.5.0
(`b48ab447b632dad90e97ee03fa77ac4eabfc86712c00ccd824db3b4ad454171d`)
verified the same inventory; changes are only reviewed `src/cli.rs`, root
version/lock metadata and VCS. README, legal files, helper/linker/runtime sources,
dependency versions and checksums are unchanged. All six registry records were
checked against the lock; metadata contains only the root plus registry inputs.

Installed results: 89 CLI cases (84 identical/different duplicate refusals and
five positive help/version/inspection controls), all five helpers through real
provider 2.17/C/CMake/host separation/local kernel simulation and target C++
refusal, exact run/test/direct-runner stdout/stdin/status checks and the bounded
capability suite passed. The first stream attempt failed because the driver
leaked its installation `CARGO_TARGET_DIR` into a fixture that expects a default
target directory. The original exit 1/log remains; clearing only that driver
setting allowed stream and capability phases to pass with the same installed
binaries. This is **composite evidence**, not one all-green initial run.

Candidate proof: 244 member hashes, 126,225 bytes, SHA256
`320d75965944bd17de756ba64558c598f711f85b7a52558a4226ae801aa30336`.
The packet and per-phase logs are retained in coordinator thread storage
(`release060-evidence/candidate-proof.tar.gz`); no new distribution assets are
attached. Source-equivalent excluded docs can reuse this evidence; a merged
package must still have its exact VCS identity verified.

[Candidate docs CI](https://github.com/rustcommons/cargo-libc/actions/runs/42427367825)
passed. [Portable Linux/Windows/MSRV CI](https://github.com/rustcommons/cargo-libc/actions/runs/42427491408)
passed at exact `1d74aea`: Linux and Windows Rust 1.99 tests/fmt/Clippy/docs,
plus Linux Rust 1.90 all-target checking. Package inputs and release scripts
are frozen to that tested candidate; later evidence-only docs do not alter them.

## Publication

Pending normal release merge, clean merged-VCS package, crates.io publication,
official archive/index verification, fresh locked registry installation,
annotated source tag, docs.rs and exact-body GitHub Release/published-docs checks.

## Limits

Qualification is bounded x86-64 GNU Linux/WSL and portable Windows parser testing.
No new old-system/tier 3, musl/static-host, real-container matrix or LLVM
interoperability qualification is claimed. Recording-engine capability cases,
if run, test route selection only. Unchanged dependency/runtime behavior is not
newly certified; existing libgcc, closure and prebuilt legal limits remain.
