# Roadmap

Plan established 2026-10-09. Initial scope confirmed: x86_64 Linux, glibc 2.17+.

## First release (0.1.0) scope

Decided 2026-10-11: the first release is **cargo-zigbuild's main use case for
x86_64 Linux glibc**, plus everything already built. Target: build for any glibc
2.17–2.44 with no setup, including crates that compile C.

**In scope:**

- [x] Automatic providers with no flags (2026-10-12): `source` mode by default (compiled with the
  host's GCC; Clang cannot compile glibc's startup sources faithfully), `container` by
  choice (`--provider-mode`, `CARGO_LIBC_PROVIDER_MODE`), never as a silent fallback
  ([PROVIDER-DELIVERY.md](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/PROVIDER-DELIVERY.md));
  `prebuilt` follows its legal review. CI builds, scans, checks, runs (tier 2) and
  verifies with the automatic provider for every release 2.17–2.44, refuses a too-new
  symbol for every release before 2.27, and builds and runs with a container-mode
  provider for 2.28.
- [x] `cargo libc provider prepare|list|path|verify|export|import|remove` (2026-10-12),
  the same implementation as `libc-abi provider`.
- [x] C dependencies (2026-10-12): the `cc` crate and CMake compile C for the target
  through `libc-cc` (Clang with the provider's or sysroot's headers, and its startup
  objects and stubs for links); build scripts and proc macros keep the host compiler.
  The flags live in the wrapper rather than `CFLAGS`, which host units would also
  read; the host `ar` archives. C++ and `-static` are refused. CI builds and runs
  `examples/c-deps` for every release 2.17–2.44 and with `--sysroot`.
- [x] More Cargo commands (2026-10-12): `clippy`, `doc` and `rustc` (like `check`;
  tool arguments after Cargo's `--` may not replace the target or linker) and
  `install` (like `build`; checks every executable it built, after installation).
- [x] Plain-Cargo use (2026-10-12): `cargo libc env [--format sh|cargo]` prints the
  rustflags, rustdoc flags, runner, helper variables and C compiler wrappers for
  `cargo build --target x86_64-unknown-linux-gnu`. CI builds, runs and inspects with
  both formats (the C dependency example with the shell one) for every release
  2.17–2.44 (since step 10).
- [x] Depend on the 0.2 workspace crates (2026-10-12: `kernel-abi-tools`,
  `linux-abi-scan` and `linux-targets` 0.2.0, no code changes) and the provider module
  of `libc-abi-tools`, from a git revision until it is published (step 12). Packaging
  replaces the git source with the version (`0.2.0`), which has no provider module, so
  `cargo package` and `cargo publish` fail verification until the provider module is
  released and the requirement names that release.
- [x] Packaging (2026-10-12): crate metadata (description, readme, keywords,
  categories, docs.rs), an `include` list of the source, README, LICENSE and NOTICE
  (22 files, 61 KiB packed; CI checks the list), no `publish = false`. The name stays
  `cargo-libc` (decided 2026-10-12).
- [x] Hosted CI for all of the above, every release 2.17–2.44 (2026-10-12): automatic
  providers (source mode), C dependencies, and clippy/doc/rustc/install/plain Cargo,
  each for every release; container mode for 2.28 here, and for every release in
  linux-abi-tools' provider workflow, which also runs the full Cargo suite with both
  modes' providers.
- [x] User documentation (2026-10-12): [USER-GUIDE.md](USER-GUIDE.md) (install, choosing a
  target, providers, every command, C dependencies, testing tiers, plain Cargo and CI,
  troubleshooting, limits, reference), a README rewritten as the front page, the
  provider guide in linux-abi-tools
  ([USING-PROVIDERS.md](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/USING-PROVIDERS.md)),
  and a crate-level page for docs.rs.

**Already built and included:**
- versioned targets;
- `build` with post-build checks;
- the tier 1 scan;
- `inspect`, `check`, `doctor`;
- `test`/`run` under the tier 2 kernel simulation, with the host glibc check.

**Out of scope for 0.1.0, documented as such:**

- **C++ dependencies:** a compatible C++ runtime per glibc era is needed; the release
  fails clearly rather than silently using the host's.
- **Hosts other than Linux x86_64.**
- **Other targets:** aarch64 and other architectures, musl, macOS and Windows.
- **Real-target qualification (tier 3):** testing on a real Rocky Linux 9 build host
  and a real SLES 12 SP5 runtime is deferred so the release can ship.
  - The release states that its evidence is tiers 1 and 2 plus real-runtime tests of
    each glibc on a modern kernel, not testing on old distributions or kernels.
  - Users must qualify their own targets.

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
The glibc 2.20 provider is qualified: producer
5ed1e12856f98566dfb2c61d0c64ce7922306b27 with consumer
8444108c1b83e72ef484b95f756308de34725844 passed
[all fifteen provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38144128128) on 2026-10-11.
The glibc 2.21 provider is qualified, completing 2.17 through 2.22: producer
85b69c8ad27be4ae486e6723ca0698a06ba96c92 with consumer
16a464ea88aa3b54d652583303fe6ed6c4d26e58 passed
[all eighteen provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38147732823) on 2026-10-11.
Every glibc release in the ABI lists, 2.17 through 2.44, now has a provider: producer
ae68bf63f5ddde7fc7852e76eab83ad2c1beb2c0 with consumer
bfe04693f6685770b47c9346e32de556a8029f4e passed [all 84 provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38158161339)
on 2026-10-11, and libc-abi-tools 0.2.0 ships the matching descriptor parser.
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
| Exact provider/release validation | Local v1 descriptors for 2.17 through 2.44; matching headers/files and conflicting requests checked | Authenticated relocatable SDK inventory and more qualified providers |
| Target linking configuration | Startup/nonshared files, stubs, compatible unwinder, explicit Clang/GNU linker selection | Broader linking modes and native dependencies |
| Host versus target separation | Explicit Cargo target and encoded target rustflags; build-script/proc-macro tests pass | Preserve this in every adapter and plain-Cargo recipe |
| Build fingerprints | Selected paths, descriptor contents, and version selection participate | Immutable package identity and authenticated provider cache; current fingerprint does not hash every SDK file |
| Built-artifact validation | Requires the selected executable to be reported by Cargo; checks direct GLIBC requirements, interpreter, and build-sysroot search-path leakage | Workspace-wide selection/checks and full dependency-closure policy |
| Friendly versioned target syntax | Version suffix or explicit glibc selection; provider remains explicit | Authenticated acquisition and additional qualified targets |
| Provider acquisition | User supplies existing inputs | Discovery, download, verification, installation, cache identity, and offline reuse |
| Target compatibility evidence | Qualified examples and CI for every glibc release 2.17–2.44 / 29 real input profiles, including both 2.17 routes | Actual intended distributions/kernels and broader native-dependency/runtime coverage |

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
- [x] Host glibc check for `test`/`run` on the host userland: refuse a binary whose `GLIBC_*` requirements the host's libraries (resolved in loader order) do not define, warn when that cannot be determined (2026-10-11; refusal qualified in producer [run](https://github.com/rustcommons/linux-abi-tools/actions/runs/38166901453)).
- [x] Use the crates.io releases of linux-abi-scan and linux-targets (0.1.0, 2026-10-11).
- [ ] Publish portable provider and tool interfaces without requiring consumers to adopt the wrapper command.

Native pkg-config and C++ configuration, dependency
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

Provider delivery is the next major work: `cargo libc build --target x86_64-unknown-linux-gnu.2.28`
without provider flags, preparing a provider by compiling shipped glibc source (default), from
CI-built binaries, or with a local container build, plus `cargo libc provider` management
commands. Design: [PROVIDER-DELIVERY.md](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/PROVIDER-DELIVERY.md);
this precedes a first cargo-libc release.
Distribution-specific providers are planned for a later release (see the producer's
[roadmap](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/ROADMAP.md#future-release-distribution-specific-providers)
and [measured differences](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/DISTRIBUTION-DIFFERENCES.md)).
Distribution/kernel qualification and the remaining review items continue independently
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
- [ ] Qualify on an actual Rocky Linux 9 build host and SLES 12 SP5 runtime (deferred past 0.1.0, 2026-10-11).

## 2. Generated glibc provider

- [x] Implement the experimental local six-field v1 contract with libc-abi-tools; authenticated relocatable release SDK contract remains open.
- [ ] Consume authenticated SDK/ABI inventories directly; today the producer checks pinned ABI data and exports validated local descriptors.
- [x] Select generated link-only libraries by exact x86_64 Linux provider/release (2.17 through 2.44), linking `libpthread_nonshared.a` only before 2.28.
- [x] Select and validate supplied compatible CRT/nonshared/header/compiler-runtime inputs; automatic acquisition remains open.
- [ ] Validate dynamic-library closure and runtime search paths.
- [x] Qualify selected threads/TLS, unwinding, C FFI, and negative newer-API examples on 2.17/2.18/2.22 in shared producer CI.

## 3. Broader support

- [x] Configure target-only native C dependencies built with cc/CMake (2026-10-12).
- [ ] C++ (a target C++ runtime) and pkg-config/system libraries.
- [ ] Expand architectures after ABI and runtime qualification.
- [ ] Add other Rust-supported libc families when a concrete use case defines
  their baseline and static/dynamic linking behavior.
- [ ] Design flag merging, workspace-wide artifact checks, and packaging.

## Rust API documentation

Added 2026-10-11. cargo-libc is used as a Cargo subcommand, but its library is
published too and gets a docs.rs page. That page must explain installation and use
and link to the user documentation, and the library API must be documented. Today
`cargo doc` builds without warnings; the library has no crate-level documentation and
80 undocumented public items (measured with `-W missing_docs` on 2026-10-11).

- [ ] Add a crate-level overview: what cargo-libc does, installation, the main
  commands, and links to the user documentation.
- [ ] Document every public item, or make items that are not meant as API private
  (`pub(crate)`). Then add `#![warn(missing_docs)]` (CI denies warnings).
- [ ] Add `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` to CI, so
  broken intra-doc links and malformed docs fail the build.
- [ ] Add `documentation` (the docs.rs URL) and `[package.metadata.docs.rs]` (target
  `x86_64-unknown-linux-gnu`, all features documented) to each published Cargo.toml,
  and a docs.rs badge to each README. (cargo-libc has both entries since 2026-10-12;
  the badge waits for the first release.)
- [ ] After each release, confirm the docs.rs build succeeded and the pages read
  correctly.

## Repository presentation and release work

| Field | Prepared value |
| --- | --- |
| Tagline | Build for the libc you choose. |
| Description | Cargo builds for selected libc environments and ELF requirement inspection, starting with glibc. |
| Topics | rust, cargo, libc, glibc, elf, abi, linux, cross-compilation, build-tools |
| README image | docs/assets/banner.svg; original editable gear and ABI diagram |
| Badges | CI workflow, Rust 1.90+, MIT, prototype status |

- [x] Apply the description/topics in GitHub repository settings (2026-10-11).
- [ ] Upload a raster social preview if wanted; the README uses SVG directly.
- [x] Observe hosted companion and producer integration CI passing.
- [x] Qualify the Rust 1.90 minimum with hosted all-targets compilation checks.
- [ ] Confirm crate-name availability and choose release/versioning policy.
- [ ] Publish a release only after its supported environment is qualified. For 0.1.0 (decided 2026-10-11),
  tier 3 is deferred and the release documents its tier 1 and 2 evidence instead.

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
