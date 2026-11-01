# cargo-libc 0.2.2 release preparation

Status: authorized patch release in preparation; not published. Version 0.2.1 and
its tag remain immutable. All five linux-abi-tools 0.3.3 packages are published and independently verified.
Registry-only release validation remains pending.

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

- [x] Verify all five official 0.3.3 archives, index checksums, non-yanked status,
  source identity and internal dependency requirements after publication.
- [x] Set cargo-libc 0.2.2; require scanner/kernel 0.3.3 and audit propagation
  through provider/internal requirements. Keep unrelated minima and glibc 0.1.0
  inputs unless a concrete dependency requirement needs a change.
- [x] Deliberately resolve all four consumed tool crates to registry 0.3.3 in the
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

## Registry adoption

All five official 0.3.3 archives match crates.io API and sparse-index checksums,
are non-yanked, and record clean VCS source
`d483fd5a58e1361d008f06db5989acef72a84e54`. Every packaged Rust file matches
that repository revision. Its Rust sources are unchanged from qualified producer
`6604ceea74152a2e8bb1b56e5605bd4feecd8365`.

| Package | SHA-256 |
| --- | --- |
| linux-targets 0.3.3 | `9138ccb80a4467c37c805b9ed7bfab8a574bd749a52fe42c4e00921e24affcb1` |
| kernel-abi-tools 0.3.3 | `0feddf7e5e33b15df1b279e3ba99c6a0743aae7edb91dd654e3a69ac01234dd3` |
| linux-abi-scan 0.3.3 | `eedffc57c88f27dee9bd28c3f177f39a457a694ddbe2678d9ba9e6078702028e` |
| libc-abi-tools 0.3.3 | `6a600e4bb78f84101a30c2658bd26a51ded0c6f79f73b74b99ad1168fa6b3bc7` |
| linux-abi-tools 0.3.3 | `82e7bc3bc7fe5bea39f215c6489b83833fd3aaa722b8d7840a15a103fc7ec2e0` |

The real registry lock selects all four consumed tools at 0.3.3. Scanner and
kernel direct caret minima are `"0.3.3"`; scanner also requires kernel and
linux-targets 0.3.3 internally, and kernel requires linux-targets 0.3.3. The direct
linux-targets minimum stays `"0.3.0"`. The provider minimum stays `"0.3.2"`, which
already guarantees its required shared-output fixes; its 0.3.3 Rust source adds no
new consumer requirement. Both glibc 0.1.0 versions/checksums remain unchanged.
No release patches, git/path overrides or unrelated upgrades are present.
