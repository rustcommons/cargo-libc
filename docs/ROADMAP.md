# Roadmap

Plan established 2026-10-09. Initial scope confirmed: x86_64 Linux, glibc 2.17+.

## 1. Repository foundation and migration

- [x] Commit CLAUDE.md/AGENTS.md and imported Git conventions first.
- [x] Preserve the original cargo-glibc source in Git before migration.
- [x] Rename package, library, command, helper, and internal environment variables.
- [x] Retain sysroot builds, host/target separation, and direct ELF inspection.
- [x] Enforce the selected glibc target floor without dropping old symbol tags.
- [x] Add README artwork, attribution, CI configuration, and migration notes.
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
