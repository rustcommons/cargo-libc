# cargo-libc 0.3.0 — qualified release candidate

Not published. Substantive pre-publication gates are complete; final narrow
review, normal merge and sole-coordinator publication remain. No release tag or
official cargo-libc 0.3.0 archive is claimed by this record.

## Behavior and migration

Ordinary references to `clock_gettime`, `clock_getres`, `clock_settime`,
`clock_nanosleep` and `clock_getcpuclockid` prefer libc's GLIBC_2.17 defaults.
There is no clock option or opt-out. Explicit legacy versions remain available;
genuine RT APIs retain their selected definitions. Library order, groups,
as-needed state, GCC/unwinder inputs and static-runtime scope are unchanged.
With no-as-needed, librt may remain DT_NEEDED while the clocks' version-needed
owner is libc. This does not add `-lrt` on later glibc releases.

The producer's `LibcClocksV1` validates actual selected libc/RT ELFs and owns a
read-only RT copy/cache, changing only the five GNU version-default flags. Raw
ABI/source data, user files and descriptor v1 remain unchanged. Automatic and
explicit providers use their selected stubs; full sysroots require contained
actual target ELFs, with no host fallback. Matching full-path RT inputs are
substituted at the same position. Completed-Clang named-input checks preserve
provable ELF/archive dependencies and bounded standard libc/libm/GCC scripts;
opaque custom scripts, responses and unproven linker-default fallbacks refuse.
See [Clock linking](USER-GUIDE.md#clock-linking) for the supported boundaries.

Regenerate complete saved shell/TOML configuration after upgrading. Both original
library hashes, the profile ID and an internal policy salt enter the target
fingerprint. The same-target-directory regression proves old-to-new relinking
without cleaning, at unchanged helper paths and descriptor. Other SDK contents
are not fully hashed. Existing provider-version cache refresh rules still apply;
no automatic import/offline migration is promised. Inspection does not retrofit
clock binding into an already-built artifact.

## Registry adoption

Only the direct `libc-abi-tools` minimum rises to 0.4.0, locked to registry checksum
`915611aa54c3fe77eefedb34bac233b077e59669130a116e2f20bc25297af009`.
Its official non-yanked archive has clean VCS
`7ce231df7128582a355c2dce47ae6436364d5e88`; all packaged `src/` bytes match
qualified producer `e857e43f113fd5fb02d0f2afff58b90fcfbd1ee6`.

Kernel/scanner minima stay 0.3.3 and linux-targets stays 0.3.0, with all three
locked at 0.3.3. Glibc inputs stay 0.1.2/0.1.1 with unchanged checksums:
`0f76191d504fe7944029bf5ac1e3ee901b050e7ce3fc946fa2661a9094045287`
and `da10605d1cab20eb0ab8cf7ebe19b619737618578dc8c74e6c62a57e1a7ec440`.
The seven-package production graph contains only registry dependencies, with no
path/git overrides, patches or new third-party dependencies.

## Qualification and exact refs

| Evidence | Ref and bounded result |
| --- | --- |
| Native source composite, x86-64 WSL | Cargo `50c9f7129abcfdb936b876fd0048c83e71fe30d1` + producer `e857e43f113fd5fb02d0f2afff58b90fcfbd1ee6`: Rust 1.90 core check/build, eight clock and sixteen ELF tests, sixteen GNU/LLD clock comparisons, explicit old versions, RT retention, host sysroot, same-directory upgrade and shell/TOML identity passed. Independent actual-wrapper named inputs passed 68/68 across GNU ld/LLD. |
| Resumed source phases | The original driver stopped at nested Clippy absent from the minimal 1.90 toolchain. Its exit 1/log is retained. Only remaining phases resumed: command smokes under installed Rust 1.99, C/CMake and host-SDK/local-3.10 simulation under Rust 1.90; all exited zero. This is composite evidence, not one successful driver exit. Driver-only `fd12615a252287a20647703fb5c9d7466aca2268` records the split and does not change production code. |
| [CI 41378566172](https://github.com/rustcommons/cargo-libc/actions/runs/41378566172) | All 15 jobs passed at `4037459a0498a0921c6a85a9d1073a3b43ff6fa8`. That commit only fixes native path components in a test expectation; production code is unchanged from `50c9`. |
| Windows registry/package | Rust 1.99 x86-64 MSVC, `4037459`: 75 library tests/all-target checks and verified 25-file package passed. Unix-only process tests are outside this Windows gate; no Linux target link or old-system qualification is inferred. |
| [Registry 41378810513](https://github.com/rustcommons/cargo-libc/actions/runs/41378810513) | Passed at `fa2826832d14f1f0586f116d45d6133731bc6d7d`: Rust 1.90 fmt/check/all-target+Unix-process tests/doctests/Clippy/docs, real registry graph, provider stdout, bounded real-ELF scanning, native streams, Docker/Podman stdin/JSON/env/exit-7, source providers 2.17/2.28, C/CMake/refusal, plain-Cargo formats, host-sysroot separation, package and clean locked install. Its README predates the final correction below. |
| README package correction | `84cc60ee4a60517f160b8003e502768575d39af7` changes only README: version-neutral status, interface-availability wording, five-clock guide link and both source-map modules. Source/manifests/lock equal `4037459`. Clean Windows VCS package: 25 files, archive SHA-256 `37677d6418834c40e630ff37cf12ce87abe4a11bec848e2782ffb622da9f8f00`. |
| [Focused package/install 41379965966](https://github.com/rustcommons/cargo-libc/actions/runs/41379965966) | Passed at `4ad06e5e2402a550eabdae2980a3e3517497a6bf`, package inputs equal `84cc60e`: revised README/package audit, verified package, fresh-Cargo-home unpacked `--locked --offline` install, all five executables and installed C/CMake/helper checks. Full/native/182-case matrices were not repeated. |

The source composite ZIP SHA-256 is
`5724ff3976d98f126d6bd711fec32387bed9f36ed60555c33572efd5d3b697ef`;
all 807 listed file digests and all 68 named-input pass results were checked.
No clock-setting fixture was executed. The initial blocked producer `1b3b51c8`
(overlap/rename race) is not final-gate evidence. Rejected ordering prototypes
and their real mixed-version alias loss remain in the dated
[investigation](evals/2026-11-02-clock-linking.md).

## Exact consumer comparison

[Run 41378111334](https://github.com/rustcommons/cargo-libc/actions/runs/41378111334)
passed at `29582bc7af35cba1f741f989b47bc520979e13ae`, using unchanged public
consumer `9c248300f1e2ab20ce34dc4b227eb322e33e4c93`, published cargo-libc
0.2.4, candidate 0.3.0 and Zig 0.17.0/cargo-zigbuild 0.23.4. Both Cargo variants
matched Zig behavior for 182/182 invocations. The candidate moved the consumer's
`clock_gettime` from librt/GLIBC_2.2.5 to libc/GLIBC_2.17, removed unneeded librt,
and preserved non-clock imports/owners and non-RT dependency order. The maximum
GLIBC requirement remained 2.18, within the selected 2.22 ceiling.

The original strict comparator is unchanged and exits 1 for both Cargo variants.
The candidate still needs libgcc_s.so.1 and retains existing non-clock import
differences relative to Zig. The separate clock assessment has no errors;
strict Zig/glibc-only equivalence is not claimed, nor is any consumer policy
relaxed. The successful three-way job definition remains byte-identical at the
final candidate; the later Windows fixture and README deltas do not change
production code or this observed behavior.

## Saved artifacts and package distinctions

| Artifact | SHA-256 of downloaded ZIP |
| --- | --- |
| `aero1553-three-way-clock-41378111334` | `fc2430f375145f818366535ef8036daa4f8144fa792ff775d38c8a1e808113ff` |
| `candidate-integration-41378810513` | `54e30f4d0c698e4d86616aad50ce163221209e084f9644977501eb5764e3f6b5` |
| `candidate-integration-41379965966` | `427bf987346d44739fd05a7d316791786bfbd715ec1443a550b143c97d8c96e1` |

Downloaded ZIP digests match GitHub's artifact metadata; exact saved refs,
resolved registry checksums, package payload hashes and consumer binary hashes
were audited. The focused CI candidate `.crate` SHA-256 is
`ac4b5a9f525bf1cce19ba1d93e85c426896e6b92bd1e22ea5c59f7c09ef74ba8`.
Its disposable git-archive source has 24 files and no VCS record; the clean
Windows package has 25 including `.cargo_vcs_info.json`. Source, lock and README
payloads agree. These are candidate archives, not an official publication.
The final README SHA-256 is
`cda3a5312a9be5761e07eb434562cdaa7860eaa66359ba0350cabd74844e5240`.

All package inputs remain equal to `84cc60e`; final guide/architecture/trace and
release-evidence edits are excluded by the unchanged include list. Package audit
also confirms the C startup/default-library/unwinder function body and unrelated
registry lock entries match official 0.2.4. The documented source changes are
limited to the clock profile/validation, fingerprint/config plumbing and related
help/metadata/tests; no general ABI-data or unwinder change is included.

## Remaining release steps and limits

Await final narrow review and normal merge; confirm the merged package tree
matches the qualified candidate. The coordinator alone publishes from that clean
merged commit. Afterwards verify official checksum/non-yanked/VCS identity,
docs.rs and actual README navigation/render, exact immutable tag, and the existing
published 0.3.0 clean-install workflow; record those results separately.

Tier 3 real old-system execution stays deferred. Modern-kernel containers/local
simulation do not establish actual SLES/kernel or universal ABI compatibility.
Container execution/IO has registry evidence; consumer clock integration with
container-provider preparation was not separately rerun. No broader native
adapter, arbitrary script/response language or runtime dependency-closure claim
is made. Prebuilt-provider legal gates are unchanged.
