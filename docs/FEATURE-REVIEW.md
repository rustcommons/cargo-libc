# Feature review: cargo-libc, cargo-zigbuild, and Zig

Review prepared 2026-10-10. This is a decision inventory, **not an approved
commitment to implement every feature**. All new scope decisions remain pending
user review. Existing functionality stays qualified only within its documented
host/target/provider scope.

## Product mission and naming gate

**Proposed mission:** build Rust applications against a selected, qualified libc
environment and explain whether their resulting artifacts meet that environment's
compatibility policy. Provider artifacts remain usable independently of the
optional Cargo coordinator.

`cargo-libc` fits this mission, including additional libc families and native
dependencies that affect compatibility. A general compiler/toolchain manager,
arbitrary SDK/package manager, or universal-binary packaging suite would broaden
the mission. Review those features individually before adopting them; reconsider
the name only if that broader role becomes the main product. No rename is proposed
or performed by this review. The producer remains `libc-abi-tools`.

| Scope | Meaning for the name |
| --- | --- |
| Core | Select libc/provider/version, inspect ABI requirements, qualify inputs and compatibility |
| Adjacent | Cargo ergonomics, native dependency adapters, CPU/toolchain coordination needed for those builds |
| Broader | Platform packaging or general compiler/SDK management with little connection to libc compatibility |

Avoid announcing "all Zig features and more": Zig supplies a compiler, linker,
runtime libraries, and multiple platform backends. Matching a Cargo wrapper's
user workflow does not imply reproducing that entire toolchain.

## Baseline and evidence

Pinned primary sources were retrieved through GitHub's contents API. The reviewed
revisions are cargo-zigbuild `d9f7fd32fbac13cc26d23847835f7f23a5ed3a25` and Zig
`738d2be9d6b6ef3ff3559130c05159ef53336224`. Source links below use those revisions,
so later upstream changes do not silently alter this review.

| Source | Reviewed evidence |
| --- | --- |
| [S1: cargo-zigbuild README](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/README.md) | Advertised target/version/platform usage, installation, and limitations |
| [S2: command dispatch](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/bin/cargo-zigbuild.rs) | Cargo commands and tool wrappers |
| [S3: Cargo environment integration](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/zig/cargo_env.rs) | Host/target, native tools, CMake, bindgen, SDK handling |
| [S4: target wrappers](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/zig/wrapper.rs) | Target translation, CPU/features, compiler/archive wrappers |
| [S5: configuration resolution](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/zig/cli_config.rs) | CLI/config/environment rustflag handling |
| [S6: executable discovery/cache](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/zig/locate.rs) | Finding Zig/Python installations and wrapper caches |
| [S7: build/artifact handling](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/build.rs) | Cargo build forwarding and universal2 aggregation |
| [S8: test integration](https://github.com/rust-cross/cargo-zigbuild/blob/d9f7fd32fbac13cc26d23847835f7f23a5ed3a25/src/test.rs) | Cargo test command configuration |
| [S9: Zig glibc backend](https://github.com/ziglang/zig/blob/738d2be9d6b6ef3ff3559130c05159ef53336224/src/libs/glibc.zig) | Bundled ABI data, link-only DSOs, version-dependent CRT/nonshared builds, cache |
| [S10: Zig musl backend](https://github.com/ziglang/zig/blob/738d2be9d6b6ef3ff3559130c05159ef53336224/src/libs/musl.zig) | Separate musl implementation path |
| [S11: Zig C++ backend](https://github.com/ziglang/zig/blob/738d2be9d6b6ef3ff3559130c05159ef53336224/src/libs/libcxx.zig) | libc++/libc++abi support supplied by Zig |

This inventory covers public command families and relevant build/platform
capabilities, not every inherited Cargo flag or internal compiler workaround.
An implemented upstream helper is not proof that every platform combination is
qualified. Unknown upstream support is recorded as unverified, not absent.

Our current baseline: Linux x86_64 application builds, GNU providers 2.17 through 2.44,
29 real input profiles, direct ELF inspection, and an experimental local v1
contract. CLI/inspection tests on Windows do not qualify Windows application
builds. Native cc/CMake/pkg-config/bindgen adapters and user SDK delivery are not
implemented. Source/runtime/license scope is in [our validation](../VALIDATION.md).

## How to read the proposals

**Retain** preserves existing behavior; **Next** is a recommended near-term
candidate; **Later** depends on stronger foundations or a demonstrated use case;
**Defer** needs a separate scope decision. Recommendations are pending review.
Cost is relative: S = a bounded extension, M = several coordinated components,
L = a new integration/backend, XL = ongoing platform/toolchain ownership.
These are estimates, not delivery dates. Each row includes a completion test.

## Cargo workflow parity candidates

| ID / scope | Feature and upstream evidence | Our current state | Proposal / cost | Acceptance or review gate |
| --- | --- | --- | --- | --- |
| F01 Core | Version-suffixed targets (S1/S3/S4) | Suffix implemented and real-provider qualified; provider still explicit | Retain / M | Strip suffix for rustc; exact provider selected; reject conflicts/unknown releases |
| F02 Adjacent | Normal build flags/workspaces (S7) | Tested workspace/package/bin/example/features/profiles and fresh artifacts; reserved options remain managed | Retain / M | Profiles/features/bin/example/workspace/locked/offline options preserved; managed target cannot be overridden |
| F03 Adjacent | `check` (S2) | Implemented 2026-10-11 (host SDK smoke test and glibc 2.17/2.18/2.22 provider fixtures) | Done / S | No final-ELF requirement for checking; correct host/target units; honest lack of link/runtime evidence |
| F04 Adjacent | `clippy` (S2) | Use ordinary Cargo separately | Later / S | Same target context; normal diagnostic output and exit status |
| F05 Adjacent | `test` and runners (S2/S8) | CI examples are tested explicitly | Next / M | `--no-run`, test executables, runner configuration and target-runtime execution remain distinct |
| F06 Adjacent | `run` (S2) | Real-loader execution is currently CI scripting | Later / M | Explicit real runtime/runner; refuse generated stubs as runtime libraries |
| F07 Adjacent | `doc` (S2) | Use ordinary Cargo separately | Later / S | rustdoc uses target headers/config where relevant; host proc macros unaffected |
| F08 Adjacent | `install` (S2) | Local installation of this tool; no wrapped app installation | Later / M | Validate built app before installation; preserve Cargo destinations/version arguments |
| F09 Adjacent | `rustc` passthrough (S2) | Not a command family | Defer / M | Clear ownership of extra flags; no bypass of compatibility policy without an explicit report |
| F10 Adjacent | Multi-target/artifact handling (S3/S7) | One managed target and explicit executable | Later / M | Per-target caches/reports; discover only Cargo-reported artifacts; no stale-file guessing |
| F11 Adjacent | Library outputs (S7; universal2 includes libraries) | Final dynamic executables qualified | Later / L | cdylib/staticlib/rlib rules differ; define inspection and downstream-link policy per type |
| F12 Core | Host build/proc-macro separation (S3) | Stable explicit target + encoded target flags tested | Retain / M | Same-triple and cross-triple fixtures; no target SDK leakage into host executables; avoid undocumented Cargo overrides |
| F13 Adjacent | Config/environment/flag resolution (S5) | Existing global Rustflags rejected; managed flags explicit | Next / M | Document precedence; merge permitted flags; reject conflicting linker/sysroot/target settings |
| F14 Core | Normal Cargo usage without wrapper | Possible through ordinary linker configuration; recipe not qualified | Next / M | Same providers and compatibility results under plain Cargo; document separately invoked validation |

## Native dependency and compiler integration

| ID / scope | Feature and upstream evidence | Our current state | Proposal / cost | Acceptance or review gate |
| --- | --- | --- | --- | --- |
| F15 Core | Target C compiler adapter (S3/S4) | Controlled C fixtures; arbitrary build scripts not configured | Next / L | A real `cc` crate dependency compiles/links with target headers while host tools remain native |
| F16 Adjacent | C++ compiler/runtime adapter (S3/S11) | Not qualified | Later / L | Choose runtime explicitly; constructors/exceptions/unwind work; audit GLIBCXX/CXXABI or alternative runtime requirements separately |
| F17 Adjacent | ar/ranlib/archive wrappers (S2/S4) | GNU tools used in CI; no exported adapter | Next / M | Native static archives use the correct target tools; deterministic/repeated builds and spaces in paths |
| F18 Core | CMake toolchain generation (S3) | Planned | Next / L | Fixture cannot find host headers/libs; programs used during build remain host tools |
| F19 Core | pkg-config isolation (S1/S3; explicit SDK handling, not a universal dependency resolver) | Planned | Next / M | Search only selected target package roots; dependency paths validated; target library cannot come from host by accident |
| F20 Core | bindgen target headers/defines (S3) | Planned | Next / L | libclang sees the provider's headers/ABI; layout fixture matches a compiled target C object |
| F21 Adjacent | CPU/features forwarded to C/C++ (S4/S5) | Fixed x86-64 baseline for Rust; no general synchronization | Later / M | Rust/native flags agree; requested CPU baseline is separate from libc version |
| F22 Adjacent | LTO/debug/sanitizer combinations (S5/S7 forward Cargo options; universal support not asserted) | Selected debug/panic-unwind path qualified | Later / L | Document supported combinations; reject incompatible compiler/LTO/runtime inputs instead of implying all modes work |
| F23 Core | Compiler/linker qualification matrix (S1/S6 discover tools; our policy differs) | Rust 1.99, Clang 21/24, GNU ld; tool MSRV checks | Retain/extend / M | Test supported tuples; LLD is a separate qualification, not implied by Clang tests |

## Providers and platform scope

| ID / scope | Feature and upstream evidence | Our current state | Proposal / cost | Acceptance or review gate |
| --- | --- | --- | --- | --- |
| F24 Core | glibc ABI data + headers/CRT/nonshared generation (S9) | Producer matrix/stubs; pinned real support inputs | Retain/extend / L | Qualify each provider, preserve version/default/library identity; publish exact tested scope |
| F25 Core | Installation/discovery/cache (S1/S6; Zig carries its libc inputs) | User-supplied inputs; workflow bundles only | Next / L | Authenticated provider identity, relocatable installation, offline use, concurrent installs and cache invalidation |
| F26 Core | Unknown-version behavior (S1/S9) | Unsupported/mismatched providers rejected | Retain / S | No silent fallback; reported selected baseline equals request |
| F27 Core | Additional GNU architectures (S4/S9) | x86_64 only | Later / XL | Choose first architecture from a user need; headers/ELF/stubs/CRT/real-runtime tests before declaring support |
| F28 Core | Additional host platforms (S1/S3/S4 have platform branches) | Application builds Linux only; Windows CLI checks | Later / L | Qualify Windows/macOS-to-Linux separately, including executable tools and paths |
| F29 Core | musl dynamic/static targets (S1/S10) | Future backend | Later / XL | Independent ABI/runtime contract; static executable behavior and dependency policy; no glibc assumptions reused blindly |
| F30 Core | Static glibc (S1 documents unsupported Zig static glibc targeting) | Not supported | Defer / XL | Explicit static archive/NSS/licensing/runtime scope; not needed for zigbuild feature parity |
| F31 Core | PIE and other dynamic link modes (S9 builds version-dependent startup support; mode coverage varies) | Provider is non-PIE | Later / M | Correct startup objects, relocations, TLS and version limits for each declared mode |
| F32 Adjacent | Darwin SDK/deployment compatibility (S1/S3) | No backend | Defer / XL | Concrete libc/LibSystem compatibility use case, SDK sourcing, Mach-O reader/linker and runtime qualification |
| F33 Broader | macOS universal2/fat aggregation (S1/S7) | Not present | Defer / L | Packaging use case and naming/scope approval; independently valid slices before combining |
| F34 Broader | Framework/install-name/platform utility emulation (S1/S3) | Not present | Defer / XL | Prefer delegating platform tools; avoid an unrelated general toolchain layer |
| F35 Core extension | Windows GNU / MinGW runtime selection (S3/S4 helpers exist; README scope is narrower) | No backend | Defer / XL | Runtime-family/use-case decision; COFF/import library rules and actual target tests, not helper presence alone |
| F36 Core extension | OpenHarmony/bionic/other libc profiles (S1 mentions external OHOS NDK) | Not present | Defer / XL | Decide each family/API baseline separately; external SDK/license/ABI requirements established first |

Platform extensions in F32/F35/F36 can remain compatible with the name if libc
environment selection stays the central behavior. F33/F34 should not become the
main reason users install the tool without revisiting the product mission.

## Proposed enhancements beyond parity

These are proposed user benefits, **not claims that Zig/cargo-zigbuild lack every
similar capability**. Dedicated commands are distinguished from backend behavior
and external tools. Upstream differences should be rechecked before any marketing
claim. All decisions are pending.

| ID / scope | Enhancement | Current foundation | Proposal / cost | Acceptance or review gate |
| --- | --- | --- | --- | --- |
| E01 Core | Provider/toolchain lockfile | Descriptor/path/version fingerprint | Next / M | Immutable hashes and tool selections; locked/offline builds detect altered or missing inputs |
| E02 Core | Compatibility report for each Cargo output | Native ELF inspection + JSON + selected reported executable | Next / M | Per-artifact human/JSON reports; distinguish direct imports, weak requirements, dependencies, CPU/kernel and unknown evidence |
| E03 Core | Dependency closure and dynamic-load policy | Direct checks; selected CI loader lists and module traces | Next / L | Resolve supplied dependency roots; no host leakage; explicit runtime traces/allowlists for dlopen/NSS; report unobserved loads as unknown |
| E04 Core | Actionable symbol provenance/errors | Producer symbol/version/default matrix | Next / M | Identify offending library/interface/provider evidence; suggest reviewable alternatives without rewriting APIs automatically |
| E05 Core | SDK authentication and complete source/license inventory | CI integrity manifests and attribution | Next / L | Trusted publisher verification, install validation, exact source/license/package records before public SDK release |
| E06 Core | Machine-readable build plan and build manifest | Doctor/dry-run, link maps, input hashes in CI | Next / M | Export selected files/tools/flags and sanitized identities; stable report schema; inspect plan without building |
| E07 Core | SDK import/export for existing distro sysroots | Supplied sysroot mode | Later / L | Classify/validate inputs; preserve provenance; distinguish generic GNU from distro-qualified profiles |
| E08 Core | Target distro/kernel/CPU qualification profiles | Selected real loaders on modern CI hosts | Later / XL | Containers alone do not qualify old kernels; use suitable runners/VMs and retain actual runtime/CPU evidence |
| E09 Core | ABI change and baseline-impact reports | Complete covered-snapshot diffs, measured profiles | Next / M | Show changes relevant to observed imports; separate membership/default/property evidence from behavior/header changes |
| E10 Core | Hermetic input policy and reproducibility checks | Shared prepared bundles, deterministic data/stubs | Later / L | Controlled dependency paths and repeated builds; distinguish reproducible outputs from merely pinned inputs |
| E11 Adjacent | Foreign runtime requirements (GLIBCXX/CXXABI/GCC) | GLIBC assessment currently separate from other namespaces | Later / L | Runtime-specific policies instead of treating every version as a glibc number |
| E12 Core | Expanded library/module coverage | Eight-library matrix; selected NSS runtime inputs | Later / L | Inventory libanl/NSS/libnsl and distro/external ownership; full evidence before acceptance/rejection verdicts |
| E13 Core | Friendly missing-target/tool diagnostics | Existing doctor probes | Next / S | Explain missing Rust standard-library target/tool/provider; installations require an explicit user-selected acquisition policy |
| E14 Adjacent | CI policy output / SARIF or equivalent | JSON assessment and hosted tests | Later / M | Stable error categories with artifact/symbol evidence; select output format from a real CI consumer |
| E15 Core | Reusable library/CLI interfaces outside Cargo | Separate producer and native inspection library | Next / M | Plain Cargo/CI/other build systems consume the same contract; wrapper stays optional |
| E16 Adjacent | Cache lifecycle and concurrent version builds | Encoded flags include provider fingerprint | Next / M | Atomic install, unique artifact identity, simultaneous baselines, safe cache inspection/pruning within managed roots |

## Review process and recommended order

For each feature ID, record **Approve / Defer / Reject / Needs evidence**, the
user problem, mission/name fit, owner (`cargo-libc`, producer, provider packaging,
or external tool), prerequisites, the acceptance test above, and maintenance
cost. Recommendations are a starting point; no feature becomes approved merely
because it appears here.

Suggested review batches:

1. **Core delivery and ordinary Cargo:** F01/F02/F12/F13/F14/F24/F25/F26,
   E01/E02/E04/E05/E06/E13/E15/E16. Establish a useful optional wrapper and
   independently usable artifacts before claiming parity.
2. **Native dependencies and validation:** F15/F17/F18/F19/F20/F23,
   E03/E09/E10/E11/E12. These provide practical compatibility value but require
   real dependency fixtures and new policies.
3. **Cargo conveniences and link modes:** F03–F11/F16/F21/F22/F31/E14.
   Prefer forwarding/delegating Cargo functionality where possible.
4. **Backend/platform expansion:** F27–F30/F32–F36/E07/E08. Approve from
   concrete users and supported environments, not a feature-count target.

Continue glibc provider rollout (2.19 next) and historical research as already
tracked; neither is contingent on copying every upstream convenience. Before
public release, revisit the name against the approved scope and actual user
workflow. A feature review does not authorize a rename or SDK publication.

### Decision ledger

| Feature IDs | Proposed disposition | User decision | Notes |
| --- | --- | --- | --- |
| F01/F02 | Retain qualified behavior | Authorized by proceed instructions; complete within tested scope | Evidence recorded below and in ROADMAP.md |
| F03-F36 and E01-E16 | As stated per row | Pending individual implementation/review | Inventory is not approval of all proposed features |
| Product mission / `cargo-libc` name | Retain for libc compatibility and optional coordination | Pending review of expanded scope | Revisit if broader toolchain/platform management becomes primary |

See [ROADMAP.md](ROADMAP.md) for implemented checkpoints and ongoing work.

F01 implementation and qualification completed following the proceed instructions.
Producer cbf31dd / consumer ed4c997 passed the current Rust 1.99/Clang 21/24
provider matrix ([evidence](https://github.com/rustcommons/linux-abi-tools/actions/runs/38076305526)).
Other feature decisions remain pending; F01 does not add automatic acquisition.

F02 qualification completed following the proceed instruction; see [Cargo workflows](CARGO-WORKFLOWS.md) and [the passing real-provider matrix](https://github.com/rustcommons/linux-abi-tools/actions/runs/38077381465). Other proposed feature decisions remain pending.
