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

Pending exact committed package, clean-home unpacked locked installation,
all-five helper smoke and installed duplicate-option/stream checks. Official
0.5.0 payload and registry graph comparison must show only the reviewed cli.rs
delta plus root version/VCS metadata, with the same file inventory.

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
