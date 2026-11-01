# cargo-libc 0.2.3 release record

Status: **published and verified**, 2026-11-01. Immutable released source:
`289f28d161027e86decc808fc1745ec4a5b74d39`; tag `v0.2.3` points exactly to it.
Registry-only qualification, official archive, docs.rs rendering and clean
published locked installation passed.

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

## Release sequence (completed)

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
- Completed: independent review and normal merge approval; verified exact merged
  package inputs and record the immutable release commit before publication.
- Completed after publication confirmation: verified official checksum/VCS/docs.rs, tagged the
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
remained identical to tested `b40b2de58e842d1d274f7882b0396ddb9612eead`. Approved
head `f7330c2892745f9ca73f511f572b9b20c52b4293` merged normally in
[PR #4](https://github.com/rustcommons/cargo-libc/pull/4) as released commit
`289f28d161027e86decc808fc1745ec4a5b74d39`. The merge tree exactly matched that
approved head and preserved its commits. No full suite was repeated for excluded
evidence text.

This bounded evidence does not rerun the full unchanged glibc/Python matrix or
supply real old-system tier 3 qualification. Prebuilt legal clearance remains
open. The documentation-only patch introduces no wider compatibility claim.

## Official publication and docs.rs render

The official [cargo-libc 0.2.3](https://crates.io/crates/cargo-libc/0.2.3) archive
is non-yanked and its downloaded SHA-256 matches the registry API and sparse
index: `cec36f14679064031545748420e97b3fbadea811be7b504177f6b25c1dfd585a`.
Its clean VCS is `289f28d161027e86decc808fc1745ec4a5b74d39`; all original
manifest/lock/README/source/license/notice files match that immutable commit.
Normalized direct dependency floors are unchanged. The lock selects the exact
verified glibc 0.1.2/0.1.1 inputs and four unchanged 0.3.3 tool crates.

[Tag v0.2.3](https://github.com/rustcommons/cargo-libc/tree/v0.2.3) was created at
the released commit and verified remotely, without moving an existing tag.
[Docs.rs status](https://docs.rs/crate/cargo-libc/0.2.3/status.json) reports
`doc_status: true`; both the [versioned API](https://docs.rs/cargo-libc/0.2.3/cargo_libc/)
and [crate README](https://docs.rs/crate/cargo-libc/0.2.3) return HTTP 200.

A real-browser inspection of that exact docs.rs README confirmed absolute
GitHub guide/reference hrefs and a loaded raw-GitHub banner (`complete: true`,
1200 by 420 intrinsic pixels). Every extracted guide/reference destination and
the banner returned HTTP 200. The [captured render](assets/release-0.2.3/docsrs-readme.png)
and [DOM/link HTTP proof](assets/release-0.2.3/render-proof.json) are retained;
screenshot SHA-256:
`e0a9277dc6e660f3912f6dda6dced65766c3fe1bac6d9ea039112934b0ee4a88`.
The published snapshot retains its link to the earlier 0.2.2 evidence record;
this 0.2.3 record supplies the current release identity and results. Published
content is not rewritten by repository bookkeeping.

## Published locked installation

[Run 41274796646](https://github.com/rustcommons/cargo-libc/actions/runs/41274796646)
**passed** at the exact released commit on Ubuntu 24.04 x86_64 / Rust 1.90.0 /
Clang 18. Installed mode used a clean Cargo home and isolated root for
`cargo install cargo-libc --version 0.2.3 --locked`. All five executables passed
help/version/inspection, provider output/preparation/verification, C/CMake
build/run/inspect and target C++ refusal checks. Installed run/test passed
redirected stdin, exact JSON stdout, separate diagnostics, local kernel runner,
deliberate error/child statuses, and Docker/Podman run/test with spaced intended
environment variables, unrelated-variable exclusion and exit 7.

Downloaded [artifact 13070957129](https://github.com/rustcommons/cargo-libc/actions/runs/41274796646/artifacts/13070957129)
records the exact released commit and installed outcomes. Its downloaded ZIP
SHA-256 matches GitHub's digest:
`87b63123c39fef9ce1c95426f7991bc5f4ac8ffe302f4700383e2443f78ee048`.

Final bookkeeping changes only excluded release/trace/roadmap documents and
render-proof assets. Source, package README, manifests, lock, workflow and all
package inputs remain identical to the immutable released commit. Prior 0.2.2
history remains unchanged. The bounded verification limits above still apply:
no tier 3 qualification, new API/behavior/data/cache format or prebuilt legal
clearance is implied.
