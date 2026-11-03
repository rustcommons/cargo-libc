# Changelog

User-facing changes in `cargo-libc`. Versions follow the repository's
[release policy](docs/RELEASING.md), independently of producer/data packages.
Dates below are original crates.io publication dates (UTC). Notes reconstructed
on 2026-11-02 were not present in historical tags; see the
[history audit](docs/releases/HISTORY.md) for provenance and evidence gaps.
GitHub Release backfill is complete for all eight existing tags; see the
[completion evidence](docs/releases/HISTORY.md#completed-backfill).

## [Unreleased]

### Development fixes

- Classify final link outputs from the completed Clang linker stream, preserving
  mode-looking RUNPATH values and actual PIE precedence. Align shared/partial
  startup inputs and refuse unsupported ambiguous overrides; LLVM runtime and
  flag/default direction remain paused.

### Documentation

- Add this changelog, versioned user-facing notes for all eight existing tags,
  release-documentation checks and navigation. No runtime, dependency or crate
  version changes; existing detailed qualification reports remain evidence.
- Record the [authorized 0.4.0 executable LLVM unwinder development](docs/UNWINDER-0.4.0-PLAN.md)
  and durable [every-bundle release checklist](docs/UNWINDER-RELEASE-CHECKLIST.md).
  Initial output classification and baseline probes are implemented; the producer
  runtime adapter/coexistence qualification are pending. No static LLVM default
  is shipped yet, and no version bump is made in this stage.

## [0.3.0] - 2026-11-02

### Changed

- Ordinary references to all five overlapping clocks prefer libc GLIBC_2.17; preserve explicit legacy references, real RT APIs and library order/state. Read-only producer overlays, effective named-input validation and profile fingerprints require regenerated saved configuration. Root libc-abi-tools minimum becomes 0.4.0; unwinder policy is unchanged.

[Release notes](docs/releases/v0.3.0.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.3.0)

## [0.2.4] - 2026-11-01

### Documentation

- Make packaged README release navigation version-neutral. Runtime code and dependency payloads remain unchanged from 0.2.3.

[Release notes](docs/releases/v0.2.4.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.2.4)

## [0.2.3] - 2026-11-01

### Documentation

- Fix packaged README guide/banner URLs for docs.rs and add maintained trace links. Adopt docs-only glibc inputs 0.1.2/0.1.1 in the lock without changing ABI/source payloads or runtime behavior. The pinned old “Current release” label is corrected in 0.2.4.

[Release notes](docs/releases/v0.2.3.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.2.3)

## [0.2.2] - 2026-11-01

### Fixed

- Keep program stdout clean by sending owned wrapper diagnostics/dry-run descriptions to stderr; preserve final error/refusal/child statuses. Adopt scanner/kernel 0.3.3 large/malformed ELF and container stdin/env/status corrections. Provider version-keyed caches may need fresh preparation.

[Release notes](docs/releases/v0.2.2.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.2.2)

## [0.2.1] - 2026-10-31

### Fixed

- Handle closed stdout without panics or suppression of later failures; forward test/run arguments after Cargo’s second separator. Include plain-Cargo too-new-symbol checks and SLES-container integration; require provider 0.3.2 fixes.

[Release notes](docs/releases/v0.2.1.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.2.1)

## [0.2.0] - 2026-10-14

### Changed

- Restrict the public Rust API to documented executable entry points. Former public internals become crate-private; the CLI workflow is unchanged.

[Release notes](docs/releases/v0.2.0.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.2.0)

## [0.1.1] - 2026-10-13

### Added

- Support provider-backed shared libraries/Python extensions: keep Rust position-independent and omit executable startup inputs for shared links; document maturin/PyO3.

[Release notes](docs/releases/v0.1.1.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.1.1)

## [0.1.0] - 2026-10-13

### Added

- First crates.io release: managed glibc 2.17–2.44 builds on x86-64 Linux, provider/sysroot selection, ELF inspection, Cargo workflow, target C wrappers and kernel simulation with native host tools preserved.

[Release notes](docs/releases/v0.1.0.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.1.0)
