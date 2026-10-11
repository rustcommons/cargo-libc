# Roadmap

Plan established 2026-10-09. Initial scope confirmed: x86_64 Linux, glibc 2.17+.

## Team handoff (2026-10-10)

F01 versioned targets and F02 Cargo flags/workspace qualification are complete
within the recorded Linux x86_64 provider scope. See the checkpoints below and
[Cargo workflows](CARGO-WORKFLOWS.md). Remaining feature candidates and enhancements
are individually tracked in [FEATURE-REVIEW.md](FEATURE-REVIEW.md); they are not
all approved implementation commitments. Keep the cargo-libc name while libc
selection and compatibility remain the main purpose.

**F03, cargo libc check (implemented 2026-10-11).**

- [x] Add check command parsing/help using the same target/provider selection and conflict rules.
- [x] Share managed target/tool configuration with build while preserving host build scripts/proc macros.
- [x] Forward appropriate Cargo check arguments and preserve Cargo diagnostics/exit status.
- [x] Do not require an executable artifact or imply linking, direct ELF inspection, or runtime qualification from check alone.
- [x] Cover success/failure, version conflicts, native host units and ordinary target forwarding; run required Rust checks and hosted provider fixtures.
- [x] Record qualified scope and pin the tested consumer in producer CI.

The glibc 2.19 provider is qualified: producer
fb5d99f977a34d8f75c455b947a22f83ef57ea35 with consumer
2a6d0e57f915c774d71b1261b6707014d2bde47e passed
[all twelve provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38142660054) on 2026-10-11.
The independent next provider rollout is glibc 2.20 in the producer roadmap.
Provider download/authentication/release SDKs, plain-Cargo qualification, native
dependency adapters, dependency closure, workspace-wide inspection, additional
platforms and old-kernel/distribution qualification remain open. No user release
has been published.

Latest qualified code (2026-10-11): producer
c8ecf2ec36f808766c31fba31bbfe43463128aec and consumer
772548dbc95b515dfb224702fdbb85b62f48a751 (build, tier 1 scan, tier 2 local
runner, and check against every glibc 2.17/2.18/2.22 provider route);
[all nine provider jobs passed](https://github.com/rustcommons/linux-abi-tools/actions/runs/38104336231).
Producer CI pins that full consumer revision. Later consumer changes are
smoke-test or documentation only. Read CLAUDE.md before starting work.

## Product direction and user value

Treat qualified provider artifacts and ABI tooling as the core, usable with
ordinary Cargo and other build systems. Keep `cargo libc` an optional coordinator
for selecting inputs, configuring a build, and validating the resulting artifact.
A new Cargo command is not required to compile against those inputs. The wrapper
should earn its place by removing setup and preventing mistakes.

For a fixed project/CI environment, normal Cargo with explicit linker and target
flags can be sufficient. Users who already accept Zig can use [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild#specify-glibc-version)'s
existing version-target workflow. Our narrower Rust implementation adds an
alternative with explicit, independently audited provider inputs; it does not
claim general cargo-zigbuild parity.

| Capability | Current state | Remaining work |
| --- | --- | --- |
| Exact provider/release validation | Local v1 descriptors for 2.17/2.18/2.19/2.22; matching headers/files and conflicting requests checked | Authenticated relocatable SDK inventory and more qualified providers |
| Target linking configuration | Startup/nonshared files, stubs, compatible unwinder, explicit Clang/GNU linker selection | Broader linking modes and native dependencies |
| Host versus target separation | Explicit Cargo target and encoded target rustflags; build-script/proc-macro tests pass | Preserve this in every adapter and plain-Cargo recipe |
| Build fingerprints | Selected paths, descriptor contents, and version selection participate | Immutable package identity and authenticated provider cache; current fingerprint does not hash every SDK file |
| Built-artifact validation | Requires the selected executable to be reported by Cargo; checks direct GLIBC requirements, interpreter, and build-sysroot search-path leakage | Workspace-wide selection/checks and full dependency-closure policy |
| Friendly versioned target syntax | Version suffix or explicit glibc selection; provider remains explicit | Authenticated acquisition and additional qualified targets |
| Provider acquisition | User supplies existing inputs | Discovery, download, verification, installation, cache identity, and offline reuse |
| Target compatibility evidence | Qualified examples and CI for four glibc releases / five real input profiles, including both 2.17 routes | Actual intended distributions/kernels and broader native-dependency/runtime coverage |

### Ordinary Cargo remains a supported design objective

- [ ] Document and test an ordinary `cargo build` recipe using the same qualified provider inputs and a reusable linker configuration/helper.
- [ ] Check host build scripts/proc macros and version switches in that recipe, including builds where host and target triples are identical.
- [ ] Compare wrapper and plain-Cargo results for accepted/rejected interfaces, imported versions, and matching real-runtime execution.
- [ ] Document the checks users must invoke separately when bypassing the coordinator.
- [ ] Keep provider selection/link configuration and ELF inspection reusable independently of the Cargo subcommand; direct `cargo-libc inspect` / `doctor` invocation already exists.
- [x] Tier 1 kernel scan in `inspect` (`--distro`, `--kernel`, `--scan`, `--strict`, `--verbose`) through `linux-abi-scan` (2026-10-10).
- [x] Optionally run the same scan on the built artifact after `cargo libc build` (`--distro`/`--kernel`/`--strict`, 2026-10-11).
- [x] Tier 2: `cargo libc test`/`run` with `--runner local|container` through the `libc-runner` target runner, including doctests (2026-10-11).
- [x] `cargo libc doctor --distro|--kernel [--runner] [--image]` checks both runners by running under them, with or without toolchain inputs (2026-10-11).
- [x] Use the crates.io releases of linux-abi-scan and linux-targets (0.1.0, 2026-10-11).
- [ ] Publish portable provider and tool interfaces without requiring consumers to adopt the wrapper command.

Automatic acquisition, native cc/CMake/pkg-config configuration, dependency
closure checks, and authenticated SDK publication remain unfinished. Existing
input validation and direct ELF checks do not replace those features or qualify
arbitrary runtime compatibility.

## Feature and naming review

See [the feature review](FEATURE-REVIEW.md) for 36 upstream parity candidates
(F01-F36) and 16 additional enhancement candidates (E01-E16), with pinned source
evidence, current coverage, recommended priority, estimated effort, and acceptance
gates. New scope proposals remain pending review; listing them is not approval.

Keep `cargo-libc` while selecting libc environments and explaining compatibility
remain the main product. Native dependency integration and additional libc
families fit that purpose. Revisit the name before adopting general toolchain/SDK
management or platform packaging as a primary role, and again before public
release against the actual approved scope. No rename is planned by this review.

- [x] Inventory upstream capabilities and additional enhancements with stable IDs.
- [x] Separate core, adjacent, and broader features and document the naming gate.
- [ ] Review the product mission and each proposed feature: approve, defer, reject, or request evidence.
- [ ] Record ownership, prerequisites, maintenance cost, and acceptance evidence for approved items.
- [ ] Review core delivery/plain Cargo first, then native dependencies, Cargo conveniences/link modes, and platform expansion.
- [ ] Recheck the product name against approved scope before public release.

Provider rollout (2.20 next) and existing qualification work continue independently
of whether we adopt every upstream convenience.

## 1. Repository foundation and migration

- [x] Commit CLAUDE.md/AGENTS.md and imported Git conventions first.
- [x] Preserve the original cargo-glibc source in Git before migration.
- [x] Rename package, library, command, helper, and internal environment variables.
- [x] Retain sysroot builds, host/target separation, and direct ELF inspection.
- [x] Enforce the selected glibc target floor without dropping old symbol tags.
- [x] Add README artwork, attribution, CI configuration, and migration notes.
- [x] Remove the original cargo-glibc folder after tests, commits, and a final
  file-by-file comparison against the preserved import.
- [ ] Qualify on an actual Rocky Linux 9 build host and SLES 12 SP5 runtime.

## 2. Generated glibc provider

- [x] Implement the experimental local six-field v1 contract with libc-abi-tools; authenticated relocatable release SDK contract remains open.
- [ ] Consume authenticated SDK/ABI inventories directly; today the producer checks pinned ABI data and exports validated local descriptors.
- [x] Select generated link-only libraries by exact x86_64 Linux provider/release (2.17/2.18/2.19/2.22).
- [x] Select and validate supplied compatible CRT/nonshared/header/compiler-runtime inputs; automatic acquisition remains open.
- [ ] Validate dynamic-library closure and runtime search paths.
- [x] Qualify selected threads/TLS, unwinding, C FFI, and negative newer-API examples on 2.17/2.18/2.22 in shared producer CI.

## 3. Broader support

- [ ] Configure target-only native C/C++ dependencies, cc/CMake/pkg-config.
- [ ] Expand architectures after ABI and runtime qualification.
- [ ] Add other Rust-supported libc families when a concrete use case defines
  their baseline and static/dynamic linking behavior.
- [ ] Design flag merging, workspace-wide artifact checks, and packaging.

## Repository presentation and release work

| Field | Prepared value |
| --- | --- |
| Tagline | Build for the libc you choose. |
| Description | Cargo builds for selected libc environments and ELF requirement inspection, starting with glibc. |
| Topics | rust, cargo, libc, glibc, elf, abi, linux, cross-compilation, build-tools |
| README image | docs/assets/banner.svg; original editable gear and ABI diagram |
| Badges | CI workflow, Rust 1.90+, MIT, prototype status |

- [ ] Apply the description/topics in GitHub repository settings.
- [ ] Upload a raster social preview if wanted; the README uses SVG directly.
- [x] Observe hosted companion and producer integration CI passing.
- [x] Qualify the Rust 1.90 minimum with hosted all-targets compilation checks.
- [ ] Confirm crate-name availability and choose release/versioning policy.
- [ ] Publish a release only after its supported environment is qualified.

Repository pushes and hosted validation are complete; user release publication remains a separate milestone.

## Experimental provider checkpoint

- [x] Consume the libc-abi-tools version-1 glibc 2.17 local descriptor via --provider.
- [x] Preserve original sysroot mode and host build scripts/procedural macros.
- [x] Verify both real 2.17 input routes, target threads, and link/runtime paths locally.
- [x] Observe Rust 1.99/Clang 21 and 24 hosted integration jobs passing with both real 2.17 input routes.
- [x] Pin the tested companion commit in libc-abi-tools CI.
- [ ] Qualify additional glibc versions listed in libc-abi-tools' roadmap.
- [ ] Add authenticated relocatable SDK inventory, licenses, and actual target distro/kernel qualification.

## GNU glibc 2.22 checkpoint

- [x] Accept explicit 2.22 descriptors and reject mismatched release selections.
- [x] Pass local host build-script/proc-macro, target thread, real loader, and link/runtime path checks.
- [x] Recheck both 2.17 provider routes with the extended consumer.
- [x] Observe all four extended Clang 21/24 and glibc 2.17/2.22 hosted provider jobs passing.
- [ ] Qualify actual SLES 12 SP5 inputs and native dependencies; generic GNU 2.22 is a separate environment.

## GNU glibc 2.18 checkpoint

- [x] Accept explicit 2.18 descriptors and reject every mismatched release selection.
- [x] Pass local Cargo host-unit, target thread, real-loader, path, and too-new-symbol checks.
- [x] Recheck both 2.17 routes and source-built 2.22 with the extended consumer.
- [x] Observe all six 2.17/2.18/2.22 Clang 21/24 provider jobs passing with producer `75945c2` and consumer `edfca5b` ([run](https://github.com/rustcommons/linux-abi-tools/actions/runs/38066726167)).
- [ ] Qualify actual target distributions, kernels, native dependencies, and release SDK packaging.

The producer's [2.18 provider reference](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/GLIBC-2.18.md)
records pinned real inputs, all-eight-library ABI evidence, exhaustive relocation
coverage, and the new pthread default-attribute boundary fixture.

## Shared provider preparation and future target selection

The producer prepares each supported glibc release once, audits the ABI and
exhaustive relocations, and transfers checked inputs to Rust 1.99/Clang 21 and 24
application jobs. Each application job builds Rust, creates a fresh local v1
descriptor after extraction, and verifies Cargo linking and real execution.
These workflow bundles are CI inputs, not published user SDK releases.
See [CI and release artifacts](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/CI-AND-RELEASE-ARTIFACTS.md).

- [x] Add coordinator syntax such as `--target x86_64-unknown-linux-gnu.2.17`, splitting the normal Rust triple from exact glibc selection; reject conflicting/unsupported selections.
- [ ] Resolve qualified providers through an authenticated inventory, reproducible cache identity, and an installable artifact layout.
- [ ] Publish tool and build-provider releases only after host/target, compiler support, source/license, and package qualification.

Version-suffixed target parsing is implemented, unit-tested, and qualified against the current real providers. Provider selection remains explicit; builds also support
`--glibc` and `--provider` selection; the provider contract is unchanged.

## Versioned-target qualification

- [x] Pass real-provider suffix builds, host-unit/thread execution, ordinary-triple artifact checks, path audits and too-new-symbol rejection for 2.17/2.18/2.22 with Rust 1.99 and Clang 21/24, including both 2.17 routes.
- [x] Reject provider mismatches and conflicting suffix/explicit version requests with expected diagnostics.

Qualified producer cbf31dd with consumer ed4c997: [all nine jobs passed](https://github.com/rustcommons/linux-abi-tools/actions/runs/38076305526).

## Cargo workflow qualification (F02)

Producer c442310 with consumer 6b07260 passed [all nine provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38077381465) under Rust 1.99 and Clang 21/24 across glibc 2.17/2.18/2.22, including both 2.17 input routes. Workspace/exclusion, binary/example and package selection, feature/no-default-feature options, release/custom profiles, locked/offline and fresh cached builds passed. Real runtimes executed all three fixture programs. Selecting an existing executable absent from the current Cargo invocation was rejected. The wrapper still validates one explicitly selected executable, not every workspace output.
