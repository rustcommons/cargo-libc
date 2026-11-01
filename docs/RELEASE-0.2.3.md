# cargo-libc 0.2.3 release preparation

Status: authorized documentation-only patch, **not published**. Registry adoption
awaits verified availability of glibc-abilists 0.1.2 and glibc-provider-sources
0.1.1. No 0.2.3 release validation is claimed yet.

## Scope

The published 0.2.2 README uses relative guide/banner links that resolve to
missing paths on docs.rs. Crates.io rewrites those links differently; that does
not fix the versioned docs.rs snapshot. The repository README now uses absolute
GitHub guide URLs and a raw GitHub banner URL. This release distributes those
corrections, distinguishes CI definitions from actual run evidence and links the
confirmed [requirements index](REQUIREMENTS.md).

The PR also refreshes maintained guides, historical labels and the trace index.
These repository docs are excluded from the crate. Rust source/API and examples,
collector/data/field/cache formats and product behavior are unchanged. Version
0.2.2, its archive, tag and [release record](RELEASE-0.2.2.md) remain immutable.

## Dependency and verification plan

- Wait for both documentation-only producer releases; verify official archives,
  registry/index checksums, non-yanked status and exact VCS refs. Record the
  producer audits proving unchanged runtime code/data/path payloads separately
  from their documentation changes.
- Set cargo-libc 0.2.3, deliberately adopt glibc-abilists 0.1.2 and
  glibc-provider-sources 0.1.1 in the real registry lock. Keep all four tools at
  0.3.3 and direct caret floors unchanged: scanner/kernel 0.3.3, libc-abi-tools
  0.3.2, linux-targets 0.3.0. No new dependencies, source overrides or patches.
- Run the exact registry candidate once through Rust 1.90 fmt/MSRV, unit/process/
  doctests, Clippy, docs and package gates; preserve bounded native provider/C/
  plain-Cargo/sysroot, stream/stdin/environment/status, scanner/real-ELF and
  Docker/Podman checks and clean unpacked-package locked installation.
- Compare package paths and Rust/license/notice payload byte-for-byte against
  the official 0.2.2 archive; compare manifests allowing only the package version
  change. Record README and intentional lock differences explicitly.
- Obtain independent review and normal merge approval; verify exact merged
  package inputs and record the immutable release commit before publication.
- After publication confirmation, verify official checksum/VCS/docs.rs, tag the
  exact release commit, run clean published 0.2.3 locked installation/native gates,
  and record results. Installed workflow arguments explicitly target 0.2.3.

The shared [documentation standard](https://github.com/rustcommons/.github/blob/main/docs/DOCUMENTATION.md)
is verified live at central merge `c8a0b9c01aae5657dd0ef649bd37b18d81919ce5`.
The trace/common AGENTS rules remain unchanged. No broader capability, prebuilt
legal clearance or tier 3 real old-system qualification is implied; tier 3 stays
deferred and is not a routine-release blocker.
