# cargo-libc 0.7.1 release record

## Scope and status

Published 2026-11-21 UTC (crates.io `created_at` 14:09:57Z) under the owner's
patch authorization of the same date, through
[Trusted Publishing](RELEASING.md#trusted-publishing) run
[44902383329](https://github.com/rustcommons/cargo-libc/actions/runs/44902383329)
after the owner's `crates-io` approval. Exact release source: merge
`15d27d50d7be9b5d7e2822e47b493147fb382c3a` of PR #41, annotated tag `v0.7.1`.
Its tree is identical to the qualified PR head
`a5999b8165a3f0d6bff06ce7f4e1f19bc1268a2b`. Scope was main at
`d93872c6b7ea8e4dcb6dc8e8fbb5fd3bfb66d7e1`, including merged PR #40 (the
linux-abi-tools 0.6.1 crate set and the fuzz lock). Cargo-libc 0.7.0 is
immutable and unchanged.

Preparation changes the root version (`Cargo.toml`, the root `Cargo.lock` entry
and the fuzz workspace's path entry), changelog, notes, this record, the package
audit and the candidate workflow's expected versions. No application source
changes.

## Version and graph review

Patch under the [version policy](RELEASING.md#choosing-a-version): a compatible
dependency update preserving documented contracts.

| Package | Manifest minimum | Locked 0.7.0 | Locked 0.7.1 |
| --- | --- | --- | --- |
| cargo-libc | root | 0.7.0 | 0.7.1 |
| libc-abi-tools | 0.6.0 | 0.6.0 | 0.6.0 |
| kernel-abi-tools | 0.3.3 → 0.6.1 | 0.3.3 | 0.6.1 |
| linux-abi-scan | 0.3.3 → 0.6.1 | 0.3.3 | 0.6.1 |
| linux-targets | 0.3.0 → 0.6.1 | 0.3.3 | 0.6.1 |
| glibc-abilists | producer's 0.1 requirement | 0.1.2 | 0.1.2 |
| glibc-provider-sources | producer's 0.1 requirement | 0.1.1 | 0.1.1 |

New lock checksums: kernel-abi-tools
`c9b45684cc8e511bce6cc2faeeedbb2bc61ea83999af084996eeaaf46731410e`, linux-abi-scan
`24731c0b81bd30c6c78e58d51e5f7454dd5131fb42c6243df01a36f93b01e468`, linux-targets
`56e731b038e838d441d01f141330c5569ca44c34e04c0780a90d6a54e1270792`. Producer and
glibc input checksums are unchanged. No third-party crate enters the graph.

### Dependency compatibility evidence

The official 0.3.3 and 0.6.1 crates.io archives were unpacked and compared on
2026-11-21:

- `linux-abi-scan`, `linux-targets`: every source file is byte-identical; only
  `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json` and
  (for linux-abi-scan) `README.md` differ.
- `kernel-abi-tools`: the `data/` kernel ABI tables are byte-identical. `lib.rs`
  replaces its row splitter with a new `src/data.rs` decoder that returns an
  error, not a panic, on a malformed row; the embedded tables still decode to
  the same syscall and madvise rows (the producer's 0.6.1 notes record equal
  results with 0.6.0).

Hence `inspect`/`build`/`test`/`run` scan results and simulation profiles are
unchanged. Provider-cache identity in `libc-abi-tools` 0.6.0 covers the producer
version, glibc release/mode/ABI lists, libgcc interface and host GCC details, none
of which change, so 0.7.0 providers are reused with the same host tools.

### Packaged source since 0.7.0

Besides the manifest/lock changes above, the package differs from the official
0.7.0 archive by:

- `src/fuzzing.rs` (new) and `pub(crate)`/`#[cfg(fuzzing)]` hooks in
  `src/cli.rs`, `src/clock_named.rs` and `src/lib.rs`: compiled only under
  `--cfg fuzzing`; shipped builds are unaffected. `Cargo.toml` declares that cfg
  in `[lints.rust]`.
- `README.md`: an OpenSSF Scorecard badge.

[`check-release-package.py`](../scripts/check-release-package.py) enforces exactly
this delta against the official 0.7.0 archive
(`b6b7d6a7e43d76a3a22b19c784b97f234eb9e1538a3341a222b42d7ebd0c3b5d`, 85,624
bytes, 28 files, source `6317117a2bea3644fa0cba11df5ac61448fd7408`).

## Candidate gates

Local, Rust 1.90.0 (aarch64 Linux host): `cargo fmt --check`, 105 library and
ten process tests, doctests (zero examples), Clippy `-D warnings` and rustdoc
`-D warnings` passed; the root and fuzz locks resolve with `--locked`. The
package audit passed at `a5999b8` (29 files, SHA-256
`962b035554aa778df4a3233531c1c2aa4e9b1cc45aa966ccdb4e6047f217fa91`).

Hosted candidate runs on PR head `a5999b8`, both successful:

- [Registry integration 44900786798](https://github.com/rustcommons/cargo-libc/actions/runs/44900786798),
  x86_64 Ubuntu 24.04, Rust 1.90.0, Clang 18.1.3. Resolved only the six locked
  crates.io dependencies with the checksums above (no path, git or patch
  sources). Passed fmt, check, 105 library and ten process tests, doctests,
  Clippy and rustdoc; provider stdout; automatic providers 2.17 and 2.28
  (built, checked, inspected, verified); C/CMake for 2.17 and 2.28 with native
  host C; Cargo commands and both plain-Cargo formats; host/target separation
  on an existing sysroot; direct, Docker and Podman run/test streams; malformed
  scan bounds (largest fixed tables 79,207 and 146,741 records within
  1,536 MiB/60 s). The package audit reproduced the local archive byte-for-byte
  (`962b0355…fa91`), and the clean unpacked `--locked --offline` install passed
  the installed-helper, provider-cache generation, duplicate-option and stream
  checks.
- [Portable integration 44900788720](https://github.com/rustcommons/cargo-libc/actions/runs/44900788720):
  Ubuntu 24.04 and Windows Server 2025 on Rust 1.99.0 (fmt, tests, doctests,
  Clippy, rustdoc) and the Rust 1.90 MSRV check.

## Publication verification

Verified 2026-11-21:

- Official archive `cargo-libc-0.7.1.crate`: 86,919 bytes, SHA-256
  `c6e51758ceec7a6484ed8132fe0f52bb2a448d68b352db3a22c272380fcadcc3`, equal to
  the crates.io API and sparse-index checksums; not yanked; `rust_version` 1.90;
  published through Trusted Publishing. Index dependencies: kernel-abi-tools,
  linux-abi-scan and linux-targets `^0.6.1`, libc-abi-tools `^0.6.0`.
- `.cargo_vcs_info.json` names `15d27d50d7be9b5d7e2822e47b493147fb382c3a`, the
  commit of annotated tag `v0.7.1`. The package audit run on a clean checkout of
  the tag against the official archive passed: every packaged file matches the
  tagged source, the delta from official 0.7.0 is exactly the one above, and all
  dependency archives match their registry checksums. Against the qualified
  candidate archive, only `.cargo_vcs_info.json` differs.
- docs.rs: `doc_status` true for 0.7.1; the versioned API page
  (`https://docs.rs/cargo-libc/0.7.1/cargo_libc/`) returns 200.
- Fresh registry install, aarch64 Linux, Rust 1.90.0, empty Cargo home:
  `cargo install cargo-libc --version 0.7.1 --locked` downloaded the six
  expected dependencies, installed all five helpers and reported
  `cargo-libc 0.7.1`. Its `inspect` correctly refused the host's aarch64
  `/bin/true` (only x86-64 ELF is supported); this host proves registry
  resolution only.
- Fresh x86_64 registry installation, [installed integration 44913046448](https://github.com/rustcommons/cargo-libc/actions/runs/44913046448)
  on main `15d27d5`, Ubuntu 24.04, Rust 1.90.0, Clang 18, clean Cargo home:
  `cargo install cargo-libc --version 0.7.1 --locked` passed the installed
  smoke with all five executables, the provider-cache generation gate (managed,
  saved plain-Cargo and explicit old Rust/C/CMake inputs), provider stdout,
  target input probes, duplicate-option checks, and direct, Docker and Podman
  run/test stream checks.
- GitHub Release [`v0.7.1`](https://github.com/rustcommons/cargo-libc/releases/tag/v0.7.1),
  published 2026-11-21T14:17:33Z with `--verify-tag` from the reviewed
  [notes](releases/v0.7.1.md): latest, not a draft or prerelease, no assets.
  `check-release-docs.py --published` passed for all 13 versions, including the
  exact live 0.7.1 body and latest status.

## Limits

Unchanged from 0.7.0: tier 1 inspection and tier 2 simulation are not
old-system qualification; real old-system (tier 3) evidence is deferred.
Modern-host container checks share the host kernel. Target libgcc matching
remains a bounded explicit two-file check.
