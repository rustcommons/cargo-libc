# cargo-libc 0.2.3 release preparation

Status: authorized documentation-only patch, **not published**. Both input
packages are published and verified; the real registry lock is adopted. Exact
0.2.3 registry/native/package validation passed; final review and merge remain
pending. Publication has not occurred.

## Scope

The published 0.2.2 README uses relative guide/banner links that resolve to
missing paths on docs.rs. Crates.io rewrites those links differently; that does
not fix the versioned docs.rs snapshot. The repository README now uses absolute
GitHub guide URLs and a raw GitHub banner URL. This release distributes those
corrections, distinguishes CI definitions from actual run evidence and links the
confirmed [requirements index](REQUIREMENTS.md).

The PR also refreshes maintained guides, historical labels and the trace index.
These repository docs are excluded from the crate. Rust source/API and examples,
collector/data/field/cache formats and product behavior are unchanged. Version
0.2.2, its archive, tag and [release record](RELEASE-0.2.2.md) remain immutable.

## Dependency and verification plan

- Completed: both documentation-only producer releases are available; verify official archives,
  registry/index checksums, non-yanked status and exact VCS refs. Record the
  producer audits proving unchanged runtime code/data/path payloads separately
  from their documentation changes.
- Completed: set cargo-libc 0.2.3 and deliberately adopt glibc-abilists 0.1.2 and
  glibc-provider-sources 0.1.1 in the real registry lock. Keep all four tools at
  0.3.3 and direct caret floors unchanged: scanner/kernel 0.3.3, libc-abi-tools
  0.3.2, linux-targets 0.3.0. No new dependencies, source overrides or patches.
- Completed: the exact registry candidate passed Rust 1.90 fmt/MSRV, unit/process/
  doctests, Clippy, docs and package gates; preserve bounded native provider/C/
  plain-Cargo/sysroot, stream/stdin/environment/status, scanner/real-ELF and
  Docker/Podman checks and clean unpacked-package locked installation.
- Completed: package paths and Rust/license/notice payload were compared byte-for-byte against
  the official 0.2.2 archive; compare manifests allowing only the package version
  change. Record README and intentional lock differences explicitly.
- Obtain independent review and normal merge approval; verify exact merged
  package inputs and record the immutable release commit before publication.
- After publication confirmation, verify official checksum/VCS/docs.rs, tag the
  exact release commit, run clean published 0.2.3 locked installation/native gates,
  and record results. Installed workflow arguments explicitly target 0.2.3.

The shared [documentation standard](https://github.com/rustcommons/.github/blob/main/docs/DOCUMENTATION.md)
is verified live at central merge `c8a0b9c01aae5657dd0ef649bd37b18d81919ce5`.
The trace/common AGENTS rules remain unchanged. No broader capability, prebuilt
legal clearance or tier 3 real old-system qualification is implied; tier 3 stays
deferred and is not a routine-release blocker.

## Verified input adoption

Official API/index/archive verification on 2026-11-01 established non-yanked
status, matching checksums and clean VCS refs:

| Package | SHA-256 | Clean source commit |
| --- | --- | --- |
| glibc-abilists 0.1.2 | `0f76191d504fe7944029bf5ac1e3ee901b050e7ce3fc946fa2661a9094045287` | `d0cf8ee40529880373286c500dd22b9c21d79e2c` |
| glibc-provider-sources 0.1.1 | `da10605d1cab20eb0ab8cf7ebe19b619737618578dc8c74e6c62a57e1a7ec440` | `2a7a0fafce50b8516208a89c98f7b5023b77ecbf` |

The input release audits were independently approved. Local archive comparison
against the former 0.1.0 packages confirms the same protected payload:

- glibc-abilists: all 394 data files, generated source and hash implementation
  are byte-identical. Changes outside metadata/README/notice are documentation,
  lint and test/doctest additions in `src/lib.rs`; runtime code is unchanged.
- glibc-provider-sources: all source files, `pack/sources.gz`, qualification
  tables, test data and licenses/notices are byte-identical. Only manifest/lock/
  VCS metadata and README differ.

The new root lock resolves exactly seven packages: cargo-libc 0.2.3, the four
unchanged 0.3.3 tool entries, glibc-abilists 0.1.2 and glibc-provider-sources
0.1.1. Registry validation asserts every exact version, registry source and
checksum, and the unchanged direct requirements. No production source override
or third-party dependency is present. Runtime Rust sources, product test/scripts
and example payloads remain identical to released `b87563c`.

Superseded run 41274051174 omitted the final checksum assertions because of a
preparation-script error. Cancellation was requested during toolchain setup;
portable/provider checks and native stream setup proceeded before cancellation
completed, so those checks are redundant preliminary evidence, not release
qualification. The complete candidate is
`b40b2de58e842d1d274f7882b0396ddb9612eead`, dispatched as
[run 41274081431](https://github.com/rustcommons/cargo-libc/actions/runs/41274081431).
No product failure was observed and no local full suite was repeated.

A scoped local Rust 1.90 locked dependency documentation check,
`RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline`, passed on 2026-11-01.
The rendered `glibc_abilists::Signature::Exception` contains the corrected
explicit-exception wording and does not claim verified signature/key presence;
`glibc_provider_sources` documentation also renders from the adopted lock.
This checks the new dependency API documentation as well as the root docs,
without claiming additional native execution.

## Exact-candidate qualification

[Run 41274081431](https://github.com/rustcommons/cargo-libc/actions/runs/41274081431)
**passed** at `b40b2de58e842d1d274f7882b0396ddb9612eead` on Ubuntu 24.04 x86_64,
Rust/Cargo 1.90.0 and Clang 18. This was registry mode: no producer checkout or
source patches, and no dependency update in the disposable validation workspace.
Saved manifest, lock and metadata match the exact candidate and all seven
asserted versions and registry checksums. Direct requirement floors are unchanged.

Passing gates:

- fmt, MSRV/all-target check, 71 unit tests, seven process regressions, doctests,
  all-target Clippy with warnings denied and root docs with warnings denied;
- provider closed-stdout/later-error behavior, redirected run/test JSON stdout,
  separate diagnostics, stdin, dry-run, local kernel runner and error/child statuses;
- malformed offset/reused-chain rejection, aliased-header acceptance, and large
  official Rust Cargo/librustc ELF tables (79,207/146,741 records) within the named
  memory/time bounds;
- Docker/Podman run and test with stdin, spaced intended environment variables,
  unrelated-variable exclusion, exact JSON stdout and exit 7;
- bounded source providers 2.17/2.28, reuse/inspection/symbol refusal, C/CMake,
  target C++ refusal, Cargo commands, both plain-Cargo formats and host-sysroot/
  host-target separation;
- verified package, checksum-verified official 0.2.2 path/payload comparison and
  clean-Cargo-home `cargo install --path <unpacked-package> --locked`, followed by
  all five installed executables/helpers, provider/C/CMake checks and installed
  stream/Docker/Podman regressions.

Downloaded [artifact 13070901258](https://github.com/rustcommons/cargo-libc/actions/runs/41274081431/artifacts/13070901258)
contains exact refs, registry metadata, manifest/lock, logs, the tested crate and
`package-payload-comparison.json`. Its downloaded ZIP matches GitHub's digest:
`75b4c66e28de6095a5c82afb114024cafee8ef9da23be73c93fa5a25c45d0206`.
Tested native crate SHA-256:
`b4563628697239f3798987aff8440fb5a12189745abde32d79a98bba05b17318`.
This disposable package is test evidence, not an official published archive.

The package comparison confirms the same paths as official 0.2.2, excluding
optional VCS metadata, and byte-identical Rust/license/notice payload. The
original manifest differs only in its own version; the intentional root lock
changes and corrected README are the other package payload changes. Candidate
README, original manifest, lock and every Rust file were also compared directly
with the downloaded tested package.

The final evidence update changes only excluded release/trace/roadmap documents.
Source, tests, scripts, workflow, README, manifests, lock and all package inputs
remain identical to tested `b40b2de58e842d1d274f7882b0396ddb9612eead`. No full
suite is repeated for the evidence text. Independent final approval and normal
merge remain required; no publication or tag is claimed.

This bounded evidence does not rerun the full unchanged glibc/Python matrix or
supply real old-system tier 3 qualification. Prebuilt legal clearance remains
open. The documentation-only patch introduces no wider compatibility claim.
