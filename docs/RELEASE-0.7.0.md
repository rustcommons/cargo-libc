# cargo-libc 0.7.0 release record

## Scope and status

Published 2026-11-08 UTC under explicit maintainer authorization to finish the
producer-to-consumer hand-off. Exact release source:
`6317117a2bea3644fa0cba11df5ac61448fd7408`. Official archive, annotated tag,
docs.rs and fresh registry installations are verified below. GitHub Release and
full published-documentation verification are complete; final closeout follows. Cargo
0.6.0 cannot be replaced. Producer workspace 0.6.0 is already released and needs
no republishing. No glibc data/source release is included.

The adoption merged as `b2db2daa13f5658ea406c55e67c59f5e371c7143`, preserving
reviewed `3a7ef25a1b1c501d6ab23d50ca9505a9aebf05d2`. User approval now selects
consumer 0.7.0. Preparation changes root version/lock metadata, workflow versions,
excluded documentation and the qualification scripts described below. Rust source,
tests, packaged README/legal files and the dependency graph remain equal to the
adoption merge; the fixture-fetch/evidence-retention correction is test-only.

## Version and graph review

Use a conservative minor for the adopted provider-cache selection/layout and
validation contract, including current-selector refusal rather than legacy
fallback. This follows [provider/cache contract review](RELEASING.md#contracts-to-review),
not a rule that all dependency updates require minors. The race repair preserves
legacy descriptors, while the new producer also changes generation selection
and physical paths; notes explain these observable effects and their limits.
No MSRV/native floor, CLI singleton rule, clock or system-unwind policy changes.

| Package | Manifest minimum | Locked version |
| --- | --- | --- |
| cargo-libc | root | 0.7.0 |
| libc-abi-tools | 0.6.0 | 0.6.0 |
| kernel-abi-tools | 0.3.3 | 0.3.3 |
| linux-abi-scan | 0.3.3 | 0.3.3 |
| linux-targets | 0.3.0 | 0.3.3 |
| glibc-abilists | producer's compatible 0.1 requirement | 0.1.2 |
| glibc-provider-sources | producer's compatible 0.1 requirement | 0.1.1 |

Only the root lock version changes from the qualified adoption. All six actual
registry identities/checksums must match; no path/git/patch/source replacement
may qualify a release. Producer checksum remains
`368579c7f6401ead63390c8c273da9d9840a905438b62e603917bb169e084d6e`.
The older tool locks are deliberate: their source/data match workspace 0.6.0.
Do not bump unrelated minima merely to match numbers.

## Carried adoption evidence

The [dated adoption record](evals/2026-11-08-provider-cache-adoption.md) retains
the 297-file proof, initial reserved-message-format failure and corrected resume,
actual legacy-provider transition, Windows prerequisite fetch, and review fixes.
Rust 1.90 source tests passed 105 library and ten process cases; Windows owner
checks passed 100 library and one process case, fmt/Clippy/rustdoc. The actual
unpatched candidate install exercised all five helpers, managed refresh/reuse,
saved plain-Cargo and explicit old Rust/C/CMake behavior, capability13 and stream
contracts. Tests establish bounded modern x86_64 Linux source mode, not old OS.

Exact adoption
[Linux/Windows/MSRV CI42501588430](https://github.com/rustcommons/cargo-libc/actions/runs/42501588430)
and [final docs CI42501821770](https://github.com/rustcommons/cargo-libc/actions/runs/42501821770)
passed with source/package equivalence. The historical published-0.6.0 test
exception is not used for the new installed release: the workflow now installs
0.7.0 and explicitly requires the generation gate. New final archives/install
identities must be measured rather than inferred from old source evidence.

## Final candidate gates

Qualified package inputs: `4caa88cccbae9dc4cfb5c3ba7b52d54bcfb415b8`.
Rust 1.90 on modern x86_64 Linux/WSL passed 105 library and ten process tests,
doctests (zero examples), clean package verification and all-six registry
archive/checksum checks. A fresh Cargo home installed the clean unpacked package
with `--locked --offline`, including all five helpers. Recorded metadata contains
only the root path and the six locked crates.io dependencies; no overrides.

The first native package-driver attempt exited 1 in the new generation fixture:
`cmake` was missing from the fresh Cargo home before its offline C fixture build.
The smoke script fetched that fixture only afterward. Qualification correction
`49c0ceea2d6b6bdadb24951e77bc8eeb91fb583e` moves the existing locked fetch before
the gate and retains its logs outside temporary cleanup. Product source/package
inputs did not change. The original failure is preserved; it is not a pass.

The same installed package passed the corrected resume: all sixteen generation
commands, managed rebuild/reuse, five retained file hashes, saved plain-Cargo and
explicit Rust/C/CMake inputs, native host-header separation and target C++ refusal.
Provider verification, capability13, CLI singleton regressions and actual run/test
stdout/stdin/status plus direct-runner controls passed. All 36 target-libgcc static
cases passed; no checker fixture was executed. No container flag was enabled.

A clean repackaged `49c0cee` archive has 28 files and 85,627 bytes, SHA-256
`b4a8982665ed1c80bae38422f2e0120921658869cbdc1225b183fa25ac1b3693`.
Only VCS metadata differs from the actual installed package; source, manifest,
lock, README and legal payloads match. The audit against official Cargo 0.6.0
allows only root-version/producer metadata and VCS differences. The official
archive at publication will identify the merge and can have a different checksum.

[Exact source portable CI42525285531](https://github.com/rustcommons/cargo-libc/actions/runs/42525285531)
passed Linux/Windows and MSRV; [docs CI42525288184](https://github.com/rustcommons/cargo-libc/actions/runs/42525288184)
passed on `4caa88c`. The later qualification-only ordering/retention correction
has identical Rust/package inputs and passed the resumed native checks; those
portable results carry within that scope. Final exact-head docs CI precedes merge.

The qualification packet contains 407 hashed regular files, actual candidate
archives, raw original/resume logs, graph, audits and installed-helper hashes:
350,264 bytes, SHA-256
`c33188bc53026de68cf1aa1db945a0729442f75eccc90ab3abb649b5eb826a82`.
Special static-test inputs such as the FIFO are recorded as special fixtures,
not read or copied as regular payload. Source-equivalent broad kernel/provider/
clock evidence keeps its original scope; no old-system/container/Python or LLVM
qualification is inferred from these checks.

## Publication and coordination

After applicable gates and review, merge normally and verify exact package-input
equivalence. Publish from the clean merge once. Confirm API/index/archive checksum,
non-yanked state, clean source VCS, file payload, annotated tag and docs.rs.
Then perform a fresh actual registry `--version 0.7.0 --locked` installation and
installed generation/capability/stream/helper checks. Publish exact committed
GitHub Release notes, latest with zero assets; verify live body/tag and full
published-documentation CI. No separate binary attachments.

Keep the organization current release matrix aligned with published state and
dependency delivery. Preserve its dated adoption/diagram records. Synchronize
the clean normal Windows Cargo checkout after completion; preserve the mirror's
uncommitted prototype. The four deferred LLVM/checklist PRs remain outside this
release. Final version-specific gates and source identities are recorded here.

## Limits

Retained generations protect against preparation, not destructive removal,
manual writes or external GCC startup-file changes. Stop readers/writers before
removal. No automatic GC, reader leases, crash durability, authenticated cache,
hermetic compiler closure or distributed-filesystem guarantee. No new container,
Python/shared-library, old-system/tier 3 or musl/static-host qualification. Prebuilt
legal review remains required; system libgcc_s and the deferred LLVM blockers
are unchanged.

## Official publication and installed verification

[PR26](https://github.com/rustcommons/cargo-libc/pull/26) merged normally as
`6317117a2bea3644fa0cba11df5ac61448fd7408`, preserving approved
`3e38c6aa0fe15ecf957390b340e8c78098246ace` exactly. Final
[docs CI42526153269](https://github.com/rustcommons/cargo-libc/actions/runs/42526153269)
passed. The sole publisher used Cargo 1.90 `publish --locked` from the clean
merge; no crate was republished and no older tag moved.

Official archive: 85,624 bytes, 28 files, SHA-256
`b6b7d6a7e43d76a3a22b19c784b97f234eb9e1538a3341a222b42d7ebd0c3b5d`.
API and sparse-index checksums agree, it is non-yanked, and clean VCS names the
exact merge. Every payload byte matches the audited merged archive. Source,
README/legal and six dependency checksums match the qualified candidate; only
merge VCS metadata differs from the earlier clean candidate archive. Annotated
[v0.7.0](https://github.com/rustcommons/cargo-libc/tree/v0.7.0) peels to that source.
docs.rs build succeeded and actual versioned README/API pages load.

Actual fresh registry `cargo install cargo-libc --version 0.7.0 --locked` passed
with isolated Cargo/install roots. All five helpers, generation16, C/CMake,
target C++ refusal, capability13, singleton options and libgcc36 were exercised.
The first local streams invocation passed run/test but its direct-runner fixture
looked under the wrong target directory inherited from installation. A test-only
change to `scripts/check-run-streams.py` explicitly selects its own temporary
target directory; the same installed binaries passed the corrected resume while
the conflicting outer target-dir value remained set. No runtime product or
package source changed. Both attempts remain in the evidence.

[Hosted actual published-install run42526522128](https://github.com/rustcommons/cargo-libc/actions/runs/42526522128)
passed at exact release source with a fresh Cargo home, the 0.7.0 registry tool,
producer 0.6.0 and the unchanged remaining graph. Its raw logs confirm all five
helpers, generation16 and both Docker/Podman run/test cases (stdin, exact stdout,
spaced environment and exit7). Container checks share a modern host kernel;
they do not establish tier3. That hosted run already passed under its clean
default target directory; no product fix was inferred from the local harness issue.

Hosted artifact13635497888: 25,261,969 bytes, ZIP SHA-256
`bccedfde902411e5490c4d1f03d070e5fe42316ae640fbb732ea263cd1da808e` matching GitHub's advertised digest. Its selected
raw log, generation summary and all sixteen zero-exit command records were
inspected. This is not a claim that every artifact member was individually
audited or that the hosted tests were rerun independently.

The local published proof contains 407 hashed regular files,
247,808 bytes, SHA-256
`6574a26400d0ab728d622c63496f137590b499b8575f36d46d7da82d4c645b85`; it includes the official archive, initial and resumed
logs, helper hashes, selected hosted verification and docs.rs status. Special
fixtures are recorded rather than treated as regular readable files.

Only excluded docs and stream-test infrastructure change during bookkeeping;
released product inputs remain exact. Exact-head docs review/checks precede
normal merge and publishing the exact committed Release body, latest/zero assets,
then full published-docs verification and final matrix/local-checkout alignment.

## Final Release closeout

Completed 2026-11-08 UTC. [PR27](https://github.com/rustcommons/cargo-libc/pull/27)
merged normally as `9052df87fcf9154875a6957821e8c2d585f9a2e2`, preserving reviewed
`bb0676b057c477511c32af0d373640f79d4f78fd` exactly. Its
[exact-head docs CI42527357980](https://github.com/rustcommons/cargo-libc/actions/runs/42527357980)
passed before normal merge and note publication. Protected package inputs remain
byte-identical to the released source; only excluded docs and the tested
stream-fixture target-directory isolation changed in publication bookkeeping.

The [live v0.7.0 Release](https://github.com/rustcommons/cargo-libc/releases/tag/v0.7.0)
has the exact committed notes, is stable/latest and has zero assets. Committed
note SHA-256: `0d1e71cb2b9b53535c7db3d39b0a22e6c5bf164cd379f47d25e359e512fb858a`.
The annotated tag remains at source `6317117a2bea3644fa0cba11df5ac61448fd7408`.
Authenticated local `--published` and
[full twelve-version verification CI42527435394](https://github.com/rustcommons/cargo-libc/actions/runs/42527435394)
passed; [release-event CI42527435111](https://github.com/rustcommons/cargo-libc/actions/runs/42527435111)
also passed. No crate republishing or tag movement occurred.

The [current organization matrix](https://github.com/rustcommons/.github/blob/main/docs/RELEASE-STATUS.md)
tracks delivered producer/consumer versions separately from dated adoption
snapshots. Its final Cargo row and the clean normal Cargo checkout are updated
as part of the same coordination closeout. Mirror prototype work and the four
held LLVM/checklist drafts are unchanged; all qualification limits above remain.
