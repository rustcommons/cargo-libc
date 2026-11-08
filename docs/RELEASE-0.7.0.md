# cargo-libc 0.7.0 release record

## Scope and status

Prepared 2026-11-08 UTC under explicit maintainer authorization to finish the
producer-to-consumer hand-off. **Candidate only, not published.** Published Cargo
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
