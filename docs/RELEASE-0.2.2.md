# cargo-libc 0.2.2 release preparation

Status: authorized patch release in preparation; not published. Version 0.2.1 and
its tag remain immutable. Registry adoption awaits verified availability of all
five linux-abi-tools 0.3.3 packages.

## Scope and compatibility

The reviewed changes are merged at
`fda38d91045b7052965e792e11b9083c7f822e46`. Product source is unchanged from
`748500343dfc0508c09b38a0ee0e007cfda98dae`; final passing consumer harness is
`192e0157063f6b2c1ab868f075eb6897b61b15f8`.

- Cargo-wrapper human diagnostics, including `--dry-run` command lines, use
  stderr; run/test stdout preserves program JSON and redirected pipelines.
  Explicit report and generated configuration output retain stdout.
- Final error reporting across all five executables is fallible and preserves
  intended error, refusal and child statuses when stderr is closed or full.
- Required scanner and container-runner dependency corrections bound malformed
  input processing while accepting valid large ELF tables, and preserve container
  stdin, intended environment variables and child statuses under Docker/Podman.

These correct existing contracts and use a patch under the
[release policy](RELEASING.md). No MSRV, native-tool floor or platform expansion is
planned. Duplicate-option handling remains deferred.

## Existing evidence

[Combined run 41265226302](https://github.com/rustcommons/cargo-libc/actions/runs/41265226302)
passed at consumer `192e0157063f6b2c1ab868f075eb6897b61b15f8` with producer
`6604ceea74152a2e8bb1b56e5605bd4feecd8365`. See the
[source qualification record](RUN-STDOUT-FIX.md) for exact artifacts, coverage and
superseded fixture failures. It used disposable source patches and does not
establish registry-only release validation.

## Remaining gates

- [ ] Verify all five official 0.3.3 archives, index checksums, non-yanked status,
  source identity and internal dependency requirements after publication.
- [ ] Set cargo-libc 0.2.2; require scanner/kernel 0.3.3 and audit propagation
  through provider/internal requirements. Keep unrelated minima and glibc 0.1.0
  inputs unless a concrete dependency requirement needs a change.
- [ ] Deliberately resolve all four consumed tool crates to registry 0.3.3 in the
  real lock; no path/git patches or unrelated dependency upgrades.
- [ ] Run exact-candidate fmt, MSRV, Clippy, tests, docs and package checks, plus
  registry-only native provider/reuse, C/CMake, config, host/target, stream,
  large/malformed ELF and Docker/Podman regressions.
- [ ] Install the unpacked proposed package with a clean Cargo home and `--locked`;
  exercise all five installed executables and affected native/container workflows.
- [ ] Independent review, normal merge and exact merged package-input equivalence;
  record the immutable release commit and artifact checksums before publication.
- [ ] After publication confirmation, verify the official archive, checksum,
  source identity and docs.rs; tag the exact release commit and run clean
  published `cargo install cargo-libc --version 0.2.2 --locked` validation.

Tier 3 real old-system qualification remains deferred. Tier 1 scans, tier 2
simulation and containers on modern kernels do not establish it. Broad unchanged
provider/Python matrices are outside this bounded release validation.
