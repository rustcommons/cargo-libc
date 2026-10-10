# Roadmap

Plan established 2026-10-09. Initial scope confirmed: x86_64 Linux, glibc 2.17+.

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

- [ ] Agree on a versioned artifact contract with libc-abi-tools.
- [ ] Consume a pinned ABI database with provenance and integrity checks.
- [ ] Select generated link-only libraries by target and release.
- [ ] Supply compatible CRT, nonshared objects, headers, and compiler runtime inputs.
- [ ] Validate dynamic-library closure and runtime search paths.
- [ ] Exercise threads/TLS, unwinding, C FFI, and negative newer-API cases on 2.17/2.22.

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
- [ ] Observe hosted CI results after the first push.
- [ ] Qualify the declared MSRV locally or in hosted CI.
- [ ] Confirm crate-name availability and choose release/versioning policy.
- [ ] Publish a release only after its supported environment is qualified.

Live repository settings, pushes, and releases are not part of this local bootstrap.

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
- [x] Observe all six 2.17/2.18/2.22 Clang 21/24 provider jobs passing with producer `75945c2` and consumer `edfca5b` ([run](https://github.com/rustcommons/libc-abi-tools/actions/runs/38066726167)).
- [ ] Qualify actual target distributions, kernels, native dependencies, and release SDK packaging.

The producer's [2.18 provider reference](https://github.com/rustcommons/libc-abi-tools/blob/main/docs/GLIBC-2.18.md)
records pinned real inputs, all-eight-library ABI evidence, exhaustive relocation
coverage, and the new pthread default-attribute boundary fixture.

## Shared provider preparation and future target selection

The producer prepares each supported glibc release once, audits the ABI and
exhaustive relocations, and transfers checked inputs to Rust 1.99/Clang 21 and 24
application jobs. Each application job builds Rust, creates a fresh local v1
descriptor after extraction, and verifies Cargo linking and real execution.
These workflow bundles are CI inputs, not published user SDK releases.
See [CI and release artifacts](https://github.com/rustcommons/libc-abi-tools/blob/main/docs/CI-AND-RELEASE-ARTIFACTS.md).

- [ ] Add coordinator syntax such as `--target x86_64-unknown-linux-gnu.2.17`, splitting the normal Rust triple from exact glibc selection; reject conflicting/unsupported selections.
- [ ] Resolve qualified providers through an authenticated inventory, reproducible cache identity, and an installable artifact layout.
- [ ] Publish tool and build-provider releases only after host/target, compiler support, source/license, and package qualification.

Version-suffixed target syntax remains planned. Current builds use explicit
`--glibc` and `--provider` selection; the provider contract is unchanged.
