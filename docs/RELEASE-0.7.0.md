# cargo-libc 0.7.0 release record

## Scope and status

Prepared 2026-11-08 UTC under explicit maintainer authorization to finish the
producer-to-consumer hand-off. **Candidate only, not published.** Published Cargo
0.6.0 cannot be replaced. Producer workspace 0.6.0 is already released and needs
no republishing. No glibc data/source release is included.

The adoption merged as `b2db2daa13f5658ea406c55e67c59f5e371c7143`, preserving
reviewed `3a7ef25a1b1c501d6ab23d50ca9505a9aebf05d2`. User approval now selects
consumer 0.7.0. This preparation changes only root version/lock metadata, expected
workflow versions and excluded documentation; Rust source, tests, scripts,
packaged README/legal files and dependency graph must remain equal to that merge.

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

Pending exact committed candidate: portable/MSRV checks, all-target tests,
package verification, comparison against the official Cargo 0.6.0 archive,
clean unpacked locked installation and generation/capability/stream/helper
checks. Use the maintained
[native package driver](../scripts/qualify-diagnostics-package.sh) with an exact
candidate SHA, real crates.io inputs and evidence outside the checkout. All
previous failures must stay distinguishable from corrected results.

Source-equivalent broad kernel/provider/clock qualification keeps its original
scope. No unchanged full matrix or new platform/LLVM claim follows. Final
recorded results and exact-head documentation checks precede normal merge.

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
