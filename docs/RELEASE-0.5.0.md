# cargo-libc 0.5.0 release record

Published on crates.io 2026-11-03 from exact clean merge
`1761260a52e192e55f774602ca05add64bd88994`, annotated tag `v0.5.0`.
Official archive/index/API, docs.rs and the separate fresh published installation
are verified below. The earlier candidate record remains explicitly composite;
the published run has no failure/resume. Live GitHub Release completion follows
the exact-committed-body procedure below, with actual final run links retained
in the bookkeeping PR rather than pre-claimed or requiring another commit cycle.

[User-facing notes](releases/v0.5.0.md), [checker contract](TARGET-LIBGCC.md) and
[release policy](RELEASING.md) define the option/status/JSON scope and limits.
There is no runtime selector, GCC floor knob, link/unwinder change or new
platform/deployment safety qualification.

## Exact refs and actual registry graph

| Role | Exact identity |
| --- | --- |
| Reviewed implementation | `972783dfd1571fb8e23385b88d147ef42c7c6dfe` |
| Code-equivalent source documentation | `cf5e49efd09a1a61f498f7602ada059eb777e683` |
| Tested registry/package/install candidate | `c5f41485af05bd311c15f567a6bd9310a3154237` |
| Published producer supplying pure byte-hash API | 0.5.0, clean VCS `bc7fcb35ea86409a2a9f543b8d25b65ed321cf47` |
| Locked libc-abi-tools checksum | `c598bccdec99a399cc8ea639c69ffd59a58dd3dc7904ea5cc586afd01dcea469` |

Source bytes at the tested candidate equal both reviewed refs. The only manifest/lock changes relative to official 0.4.0 are the root version
and libc-abi-tools minimum/registry version/checksum.
Metadata on Linux and Windows resolves exactly seven packages: root 0.5.0 and
registry libc-abi-tools 0.5.0, kernel-abi-tools 0.3.3, linux-abi-scan 0.3.3,
linux-targets 0.3.3, glibc-abilists 0.1.2 and glibc-provider-sources 0.1.1.
The five unrelated dependency entries/checksums remain unchanged, with existing
direct floors. Rust 1.90 and native-tool floors stay unchanged. No production
path/git patch, replacement source or invented checksum is present.

Independent official API/archive checks verify the producer checksum,
non-yanked state, exact VCS and public `content_sha256`. The package auditor also
checks all six locked dependencies against their registry API/archive checksums.
The first-party digest API is pure; ELF parsing/matching stays internal to Cargo.
The new producer version still participates in provider-cache identity, so fresh
provider preparation may be needed. No automatic import/offline migration is
promised. Source preparation uses host GCC/recorded inputs; container preparation
also needs host startup/tools and its engine/image, distinct from execution.

## Package identity and source/payload audit

The clean Rust 1.90 `cargo package --locked --offline` **with verification** passed:
**28 files, 84,781 bytes**, SHA256
`5ac6373f445fceb8b0ffd0549cd9f33ff61de51f70d296ac4949f3dcc028a441`,
clean `.cargo_vcs_info.json` naming full tested `c5f41485` above.
This is a candidate archive, not an official published package or predicted
final-merge checksum. A later docs-only commit changes VCS identity naturally.

Official0.4.0 baseline: 26 files/73,212 bytes, source
`aa2ea56e5d7943aeb4e0023713a089693d2103c1`, SHA256
`7c6177bbd11ddc43cf2e800ea2b30050eeaf52942eebbec4e531dcbc60ba6e0b`.
The actual audit permits exactly new `src/elf/exact.rs` and `src/gcc.rs`, common
changes in `src/cli.rs`, `src/elf.rs`, `src/lib.rs`, `src/report.rs`, plus manifest/
root-producer lock/VCS metadata. README, LICENSE, NOTICE, provider/linker/startup/
unwinder inputs and every other source file equal that baseline. All source
files and Cargo.toml.orig/lock/legal/README bytes equal the tested checkout;
normalized manifest differs only in root and producer requirement versions.

The [candidate identity/inventory record](evals/2026-11-03-target-libgcc-candidate.json)
retains every packaged-file hash, six registry checksums, installed Linux/Windows
binary identities and all command results. Candidate source snapshot bytes and
all 228 listed proof members were independently hashed against the supplied packet.
The owner audit was read-only on ARM; no x86 executable/fixture was run there.

## Portable and source evidence

- [Real-registry portable CI41684307122](https://github.com/rustcommons/cargo-libc/actions/runs/41684307122)
  passed at exact `c5f41485`: Linux/Windows Rust 1.99 fmt, all-target tests,
  doctest command, warnings-denied Clippy/docs and Rust 1.90 all-target check.
  Native/provider/consumer/installed workflow jobs were intentionally skipped.
- [Local-mode docs CI41684311480](https://github.com/rustcommons/cargo-libc/actions/runs/41684311480)
  passed at the same candidate, `verify_published=false`. It does not claim a
  GitHub Release for unpublished cargo-libc 0.5.0.
- The [historical source qualification](evals/2026-11-03-target-libgcc.md) at
  implementation `972783d` / producer API source `2c70e479` passed 102 library,
  nine process, zero-doctest command, 36 GNU static cases and the named legacy 0.4
  comparison with Rust 1.90/Clippy1.99. Its 150-member proof SHA256 remains
  `0b7cd40fbf354141bde0d10205f53358685be26b25dd4e2e7880dab6fecc7ef8`.
  That disposable-patch source evidence is distinct from the unpatched real
  registry/package qualification below and is not relabeled as one.

## Native real-registry composite qualification

Native x86_64 WSL GNU userland, Rust 1.90 core and pinned 1.99 formatting/Clippy,
Clang 18 and GCC/GNU tools; two build workers, no overlapping heavy jobs. The
retained raw logs show 102 library tests, nine process tests, zero doctests,
docs, fmt/Clippy and verified package compilation passing. A fresh Cargo home and
isolated root installed the actual unpacked candidate with `--locked --offline`;
all five helpers installed and version reports 0.5.0. No unpublished registry
cargo-libc 0.5.0 installation was attempted.

Installed C/CMake smoke passed: actual provider 2.17 and target C headers, native
host proc-macro separation, app execution under local kernel 3.10 simulation,
provider stdout/verification, direct inspect and target C++ refusal. Thirteen
installed capability cases passed, including no-probe help/version/inspect,
local/direct missing interpreter and closed-diagnostic-pipe status 2, recording
container-route status 17, doctor mock failure and native C/C++ host wrappers.

### Original failure and clean-environment resume

The parent driver retained docs-only `RUSTDOCFLAGS=-D warnings` in the child
environment. During the first installed stream sequence, the run control passed;
`cargo libc test` then correctly refused that override with its existing error
and status 1. The stream checker itself exited 1. Original stdout/stderr/result
remain in the packet; this attempt is **not a pass** and no product change was
needed. No source, manifest, lock, package or installed binary changed afterward.

After clearing the unintended overrides, only the remaining stream/checker/docs
phases resumed. The full run/test/direct-runner stream script passed: exact JSON
stdout, redirected stdin, separated diagnostics, intended failure/pipe/full-device
statuses. Installed static checker regressions passed all **36 GNU cases** with
the actual installed released 0.4.0 baseline argument: ordinary inspect text/JSON
and exits were byte-for-byte equal in the named control. Static fixtures were
compiled/linked and inspected, never executed; no fixture IFUNC, target loader or plugin was invoked by the checker.
The final local release-doc command passed. This is **composite resumed evidence**,
not a claim that one monolithic driver execution was all green.

LLD was unavailable in the named source/native environment; no LLD pass is
claimed. Its conditional fixture remains a future gate when LLD is available.

## Actual Windows CLI controls

Native Windows x86_64 MSVC/Rust 1.99 built the five binaries against the same
real-registry graph at clean `c5f41485`. The actual Windows cargo-libc.exe read
three transported GNU ELF fixtures: match/status 0, missing/status 2 and wholly
versionless unresolved/status 3. Both artifact and runtime full-file SHA256s match
the corresponding Linux fixture bytes; the Windows executable hash is retained.
This is file-inspection portability, not Linux target execution or Windows
provider/managed-build platform qualification. Portable tests were not repeated:
the separate green portable CI above remains the evidence for those gates.

## Proof identity, resource scope and remaining release steps

`candidate-c5f41485-proof.tar.gz`: **14,930,341 bytes**, **228 verified member
hashes**, SHA256
`461de6aff75d0009e11cd5d038346db123012aa70acc9067d485eb9c25894322`.
Raw commands/results, the original failed stream attempt, clean resume, actual
package and source archive, Linux installed binaries/fixtures and native Windows
binary/outputs are retained. Independent scoped release review accepted this
exact candidate and composite evidence; the final excluded-doc head still needs
its narrow review before normal merge.

Primary 256 MiB and target 64 MiB input buffers are both retained, with additional
hash/parser/report working memory. Symbol/metadata/string/relocation/report budgets
in the [contract](TARGET-LIBGCC.md#bounds-and-limits) are not a whole-process memory
cap. No peak-memory optimization or general runtime/dependency resolver ships.
A structural interface match does not establish authenticity, loader selection,
transitive/plugin/CPU/kernel closure or safe execution. No real container matrix,
real old-OS/tier3, musl/static-host or LLVM/unwinder interoperability qualification
is claimed. The container engine in the capability packet is a recording fixture.
System unwinder, clock behavior and prebuilt legal gates remain unchanged.

Only excluded evidence/status docs follow the tested candidate; compare source,
package inputs and harness to `c5f41485` during narrow review. No build/install/
unchanged runtime matrix is rerun for that delta. Parent alone publishes locally
after normal reviewed merge, then official archive/docs.rs/published installation,
immutable tag and GitHub Release completion follow the established policy.
Those candidate-stage boundaries are superseded only by the separate publication
evidence below; this documentation update changes no product/package inputs.

## Publication and installed verification

The authorized local publisher ran `cargo +1.90 publish --locked` from clean
release `1761260a52e192e55f774602ca05add64bd88994`, exit 0. Independently downloaded
API/index/archive records agree on checksum and non-yanked state. The actual
published package contains 28 files, 84,779 bytes, SHA256
`b48ab447b632dad90e97ee03fa77ac4eabfc86712c00ccd824db3b4ad454171d`.
All source/manifest/lock/README/legal bytes equal the released checkout and the
normalized manifest equals qualified C5. Only `.cargo_vcs_info.json` differs from
C5. This official archive hash is distinct from candidate hash 5ac6373f….

Annotated [`v0.5.0`](https://github.com/rustcommons/cargo-libc/tree/v0.5.0),
object `d25665dc7aefb36761a18cf2192e594c74104ddf`, peels exactly to that release
commit. It was created only after confirming no existing local/remote tag and
never moved. Release-commit [docs CI 41689551201](https://github.com/rustcommons/cargo-libc/actions/runs/41689551201)
passed before completion. Docs.rs [API](https://docs.rs/cargo-libc/0.5.0/cargo_libc/),
[actual README](https://docs.rs/crate/cargo-libc/0.5.0) and
[build 5142829](https://docs.rs/crate/cargo-libc/0.5.0/builds/5142829) returned HTTP 200.
A browser verified the actual README and six loaded package images, plus guide/
checker/policy/banner targets returning HTTP 200. Docs.rs's own build toolchain is not
the MSRV gate.

### Fresh published-registry installation, no resume

On native x86_64 WSL GNU with Rust 1.90 and two workers, a new work directory/
Cargo home and isolated installation root ran the actual published command:

```sh
cargo install cargo-libc --version 0.5.0 --locked --root <isolated-root>
bash scripts/smoke-installed.sh <isolated-root> /usr/bin/clang-18 0.5.0
python3 scripts/check-target-libgcc.py <isolated-root>/bin/cargo-libc \
  <fixtures> <verified-installed-0.4.0>/bin/cargo-libc
python3 scripts/check-installed-capabilities.py \
  <isolated-root>/bin /usr/bin/clang-18 <capabilities>
python3 scripts/check-run-streams.py <isolated-root>/bin /usr/bin/clang-18
```

All five phases exited 0: registry install 23.71s, installed smoke 14.97s,
checker 0.58s, capabilities 0.34s, streams 18.18s. All five executable hashes match
the retained binaries. The actual official archive equals the independent owner
download. This is not an unpacked/path install or reuse of candidate execution.
The driver removes conflicting flags before execution; no original failure or
resume occurred in this postpublication run.

Actual provider 2.17, real C/CMake target headers and host proc-macro separation,
app execution under local kernel 3.10 simulation, direct inspect/provider stdout/
verification and target C++ refusal passed. The installed checker passed all 36
GNU static cases including the actual released 0.4.0 ordinary text/JSON comparison.
The static ELF/IFUNC payloads were never executed. Thirteen capability controls,
including missing-interpreter status 2, closed diagnostic pipe and recording
container-route/doctor cases, passed. Run/test/direct-runner exact JSON stdout,
redirected stdin, separated diagnostics, intentional failure/pipe/error statuses
also passed. Candidate Windows portability evidence remains separately scoped;
no Windows published installation is invented.

Proof `published-consumer-proof.tar.gz`: **7,467,920 bytes**, SHA256
`f6d3ef73bc03bc0b5998af3519f5367792045558b213e474807a41ab04977472`;
all **177 member hashes**, five binary identities, 36 checker/13 capability results,
commands/driver/logs and official archive were independently checked. The
[publication identity record](evals/2026-11-03-target-libgcc-publication.json)
retains official payload and published-install binary/phase identities.

This native result is not a hosted published-install workflow. Container controls
use a recording engine only. No real container matrix, old-OS/tier3, musl/static
host, LLVM replacement/interop, full loader/dependency/plugin closure or general
safe-to-run certificate is claimed. Both input buffers remain retained; read/
parser/report bounds are not a total-process memory cap. System unwinder, clocks,
MSRV/native floors, unchanged dependency/data locks and prebuilt legal gate remain
as documented. No republication, credential provisioning or product change is
part of this completion.

### GitHub Release completion procedure

Use the exact reviewed committed `docs/releases/v0.5.0.md` after green docs checks
and normal documentation merge, with existing immutable `v0.5.0`, stable/latest
status and zero assets. Verify live exact body/tag/latest/assets, then the full
registered release-doc workflow with `verify_published=true` and equivalent:

```sh
python3 scripts/check-release-docs.py --repo rustcommons/cargo-libc --published
```

Actual final run/result links are retained in the reviewed bookkeeping PR and
[release-doc workflow history](https://github.com/rustcommons/cargo-libc/actions/workflows/release-docs.yml).
This procedure does not pre-claim a future check as passed; earlier local-only
checks are not relabeled as published-mode evidence. No source/tag movement or
further product/package build is needed to store those final links.
