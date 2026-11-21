# cargo-libc 0.7.1 release record

## Scope and status

Release candidate prepared 2026-11-21 UTC under the owner's patch authorization
of the same date. Scope is existing main at
`d93872c6b7ea8e4dcb6dc8e8fbb5fd3bfb66d7e1`, including merged PR #40 (the
linux-abi-tools 0.6.1 crate set and the fuzz lock). Not published: no tag,
crate upload or GitHub Release exists yet. Publication goes only through
[Trusted Publishing](RELEASING.md#trusted-publishing) with the owner's
`crates-io` approval. Cargo-libc 0.7.0 is immutable and unchanged.

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
`-D warnings` passed; the root and fuzz locks resolve with `--locked`.

Results of the package audit and hosted runs are recorded below as they are
measured; none is inferred from 0.7.0 evidence.

## Publication

Pending. After publication: official archive/index checksum and VCS identity,
docs.rs, fresh registry installation (`installed` mode), GitHub Release notes
and the live release-documentation check.
