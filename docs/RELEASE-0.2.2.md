# cargo-libc 0.2.2 release record

Status: **published and verified**, 2026-11-01. Released commit
`b87563c6be6faf89ac26055f88fb520991125b8b`; immutable tag `v0.2.2` points exactly
to it. Registry-only release validation, clean published installation and docs.rs
checks passed. Version 0.2.1 and its tag remain unchanged.

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
[release policy](RELEASING.md). No MSRV, native-tool floor or platform scope changed. Duplicate-option handling remains deferred.

## Provider cache refresh

The lock now selects `libc-abi-tools` 0.3.3. Its version is part of the provider
cache key, so the first automatic build may prepare fresh providers; entries
keyed by the previous version are not silently reused. Source mode uses embedded
ABI/source inputs but still needs host GCC and the required native tools.
Container preparation may need network access for source archives and access to
the pinned container image. No import or offline migration guarantee is implied.
This follows the published [cache-key and preparation implementation](https://github.com/rustcommons/linux-abi-tools/blob/d483fd5a58e1361d008f06db5989acef72a84e54/src/provider/prepare.rs).

## Existing evidence

[Combined run 41265226302](https://github.com/rustcommons/cargo-libc/actions/runs/41265226302)
passed at consumer `192e0157063f6b2c1ab868f075eb6897b61b15f8` with producer
`6604ceea74152a2e8bb1b56e5605bd4feecd8365`. See the
[source qualification record](RUN-STDOUT-FIX.md) for exact artifacts, coverage and
superseded fixture failures. It used disposable source patches and does not
establish registry-only release validation.

## Completed release gates

- [x] Verify all five official 0.3.3 archives, index checksums, non-yanked status,
  source identity and internal dependency requirements after publication.
- [x] Set cargo-libc 0.2.2; require scanner/kernel 0.3.3 and audit propagation
  through provider/internal requirements. Keep unrelated minima and glibc 0.1.0
  inputs unless a concrete dependency requirement needs a change.
- [x] Deliberately resolve all four consumed tool crates to registry 0.3.3 in the
  real lock; no path/git patches or unrelated dependency upgrades.
- [x] Run exact-candidate fmt, MSRV, Clippy, tests, docs and package checks, plus
  registry-only native provider/reuse, C/CMake, config, host/target, stream,
  large/malformed ELF and Docker/Podman regressions.
- [x] Install the unpacked proposed package with a clean Cargo home and `--locked`;
  exercise all five installed executables and affected native/container workflows.
- [x] Independent review, normal merge and exact merged package-input equivalence;
  record the immutable release commit and artifact checksums before publication.
- [x] After publication confirmation, verify the official archive, checksum,
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

## Registry-only release validation

[Run 41268879926](https://github.com/rustcommons/cargo-libc/actions/runs/41268879926)
**passed** at exact candidate
`734f9bc4205e4a5c119feaa3dd35c49a4761e1c3` on Ubuntu 24.04 x86_64, Rust 1.90.0
(the declared MSRV) and Clang 18. Registry mode skipped the candidate checkout
and patch/update steps; saved metadata and lock prove all four consumed tools
resolve to registry 0.3.3 with the verified checksums above. Both glibc 0.1.0
entries are unchanged.

Passing coverage:

- fmt, all-target/MSRV check, 71 unit tests, seven process regressions, doctests,
  all-target Clippy with warnings denied and docs with warnings denied;
- shared-provider closed-stdout/later-error handling, redirected run/test JSON
  stdout, separate diagnostics, stdin, dry-run, local kernel runner and deliberate
  error/child statuses;
- malformed offsets and reused-version-chain rejection, aliased-header acceptance,
  and official Rust Cargo/librustc large fixed tables (79,207/146,741 records);
- Docker/Podman run and test with redirected stdin, spaced intended environment
  variables, exact JSON stdout, unrelated-variable exclusion and child status 7;
- source providers 2.17/2.28, reuse/inspection/symbol refusal, C/CMake, target C++
  refusal, Cargo commands, both plain-Cargo formats, sysroot/host-target checks;
- verified packaging and `cargo install --path <unpacked-package> --locked` with a
  clean Cargo home and isolated installation root, followed by all five installed
  executable/helper, provider, C/CMake and stream/container regressions.

Local aarch64 Rust 1.90.0 fmt/check/tests/Clippy/docs/package checks also passed;
those portable checks do not establish native x86_64 behavior.

Downloaded [evidence artifact 13068637219](https://github.com/rustcommons/cargo-libc/actions/runs/41268879926/artifacts/13068637219)
contains exact refs, registry metadata, manifest/lock, native logs and the tested
package. Its downloaded ZIP SHA-256 matches GitHub's digest:
`e753ea450bb03cd2e652a6ea29e3ef43d850fa64122ceae6c347491086fa72e8`.
The tested native `.crate` SHA-256 is
`7e137008235f64e063d99d8e15fd3b04c920df6aba11842aed0f6ef262f0d3a7`.
Its original manifest, lock and every Rust source file match the exact candidate.
This disposable workflow package is test evidence, not a published archive; the
official archive checksum/VCS verification is recorded below.

Final reviewed head `2d6f77ed765e12e1343704694bf9abb9d5db6e57` changed only this
excluded release document relative to tested
`734f9bc4205e4a5c119feaa3dd35c49a4761e1c3`. Source, regression scripts/tests,
workflow, manifests, lock and all package inputs remained identical.
[PR #3](https://github.com/rustcommons/cargo-libc/pull/3) merged normally as
`b87563c6be6faf89ac26055f88fb520991125b8b`, preserving reviewed commits; its tree
exactly matches the approved head. No full suite was repeated for evidence text.

## Publication and installed verification

The official [crates.io 0.2.2 release](https://crates.io/crates/cargo-libc/0.2.2)
is non-yanked. Downloaded archive SHA-256 matches both the API and sparse index:
`7ebc74cde32437217a0e8bcf0fec84c33436fb2ed33d066fac9651e4d0da7caf`.
`.cargo_vcs_info.json` records clean released commit
`b87563c6be6faf89ac26055f88fb520991125b8b`. All package payload files, original
and normalized manifests and the registry lock were verified against that commit;
all four tools are registry 0.3.3 and glibc inputs remain 0.1.0 with unchanged
checksums. No source override is packaged.

[Tag v0.2.2](https://github.com/rustcommons/cargo-libc/tree/v0.2.2) was created at
that exact commit and verified remotely. The [docs.rs status](https://docs.rs/crate/cargo-libc/0.2.2/status.json)
reports `doc_status: true`, and the [versioned API page](https://docs.rs/cargo-libc/0.2.2/cargo_libc/)
returns HTTP 200.

[Published-install run 41269812853](https://github.com/rustcommons/cargo-libc/actions/runs/41269812853)
**passed** at the exact released commit on Ubuntu 24.04 x86_64 / Rust 1.90.0 /
Clang 18. Installed mode ran
`cargo install cargo-libc --version 0.2.2 --locked` in a clean Cargo home into an
isolated installation root, using the published registry package. It exercised
all five executables, help/version/inspection, shared provider output and
preparation/verification, C/CMake execution and target C++ refusal. Installed
run/test also passed exact JSON stdout, separate diagnostics, redirected stdin,
local kernel runner and failure statuses, plus Docker and Podman run/test with
spaced intended environment variables, unrelated-variable exclusion and exit 7.

Downloaded [published-install artifact 13069180772](https://github.com/rustcommons/cargo-libc/actions/runs/41269812853/artifacts/13069180772)
records the exact release commit and passing installed checks. Its downloaded ZIP
SHA-256 matches GitHub's digest:
`cf3b0085c417d685ed7ca93e943d6bdc523d3278a1ea5dde73f9261d76fa9275`.

Release bookkeeping changes only excluded repository documentation; package
inputs remain identical to the immutable released commit. Tier 3 qualification
remains deferred as described above; no real old-system compatibility or broader
unchanged provider/Python matrix result is claimed.
