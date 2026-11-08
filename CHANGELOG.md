# Changelog

User-facing changes in `cargo-libc`. Versions follow the repository's
[release policy](docs/RELEASING.md), independently of producer/data packages.
Shipped-version dates below are original crates.io publication dates (UTC);
an explicitly marked candidate uses its preparation date. Notes reconstructed
on 2026-11-02 were not present in historical tags; see the
[history audit](docs/releases/HISTORY.md) for provenance and evidence gaps.
Historical GitHub Release backfill is complete for the eight tags through 0.3.0; see the
[completion evidence](docs/releases/HISTORY.md#completed-backfill).

## [Unreleased]

No additional changes assigned.

## [0.7.0] - 2026-11-08

**Published 2026-11-08.** [Notes](docs/releases/v0.7.0.md) and
[qualification record](docs/RELEASE-0.7.0.md).

### Fixed

- Require `libc-abi-tools 0.6.0` for immutable provider generations. Forced
  preparation retains paths already returned to builds; new automatic selections
  use the new generation and Cargo fingerprint. Old explicit descriptors and
  saved settings keep their original generation. Other dependency locks, glibc
  inputs, tool floors, clock policy and the system unwinder are unchanged.
  [Adoption and limits](docs/evals/2026-11-08-provider-cache-adoption.md).
  Shipped in 0.7.0; immutable cargo-libc 0.6.0 still uses the older producer.

## [0.6.0] - 2026-11-07

**Published 2026-11-07.** [Notes](docs/releases/v0.6.0.md) and
[qualification record](docs/RELEASE-0.6.0.md).

### Changed

- Reject duplicate `--sysroot`, `--gcc-runtime`, `--cc`, `--ld`, `--artifact`
  and inspect `--max-glibc` values, including identical repeats, before doing
  work. Previously the last value won. Invalid arguments exit 1 with a stderr
  diagnostic and no stdout report. Scripts must select one value instead of
  appending an override. Repeated switches and Cargo argument forwarding are
  unchanged. This stricter CLI contract requires a minor release. See the
  [migration guidance](docs/USER-GUIDE.md#duplicate-value-options-060).

## [0.5.0] - 2026-11-03

**Published 2026-11-03.** [Release evidence](docs/RELEASE-0.5.0.md) records exact
source/tag/official archive, docs.rs and a separate successful fresh registry
installation. Prepublication composite/refusal/resume evidence remains separately
identified, with memory/runtime/platform limits preserved.

### Added

- `inspect --target-libgcc FILE`: explicit read-only comparison of recorded
  libgcc-owned mandatory version nodes and exact function symbol/version pairs
  against the supplied x86-64 runtime ELF. Optional JSON evidence separates
  match, missing and unresolved results, input identities and the runtime's own
  direct GLIBC assessment. A match does not establish loader selection, runtime
  authenticity, transitive/plugin closure or safety to run.

### Upgrade and limits

- The minor version covers the new inspect option/report/status contract;
  ordinary inspect without the option preserves existing output and exits.
  No new GCC-floor flag, runtime selector, linker policy or unwinder is included.
- Only the root producer minimum becomes libc-abi-tools 0.5.0 following
  verified publication. Other tool/data locks and MSRV/native floors stay unchanged.
  The producer version participates in provider-cache identity; upgrading from
  0.4.x can prepare fresh providers. No automatic cache import/offline migration
  is promised; source mode needs host GCC, and container preparation has host-tool
  plus engine/image prerequisites.
- Bounded reads retain both input buffers; per-file/parser bounds are not a
  whole-process memory cap. [Release notes](docs/releases/v0.5.0.md) and the
  [checker contract](docs/TARGET-LIBGCC.md) give exact limits and known gaps.

## [0.4.0] - 2026-11-03

**Published 2026-11-03.** [Release evidence](docs/RELEASE-0.4.0.md) records exact
source/tag/archive, docs.rs and separate fresh registry-install verification,
alongside prepublication candidate evidence and preserved coverage limits.

### Changed

- Doctor/build diagnostics separate tool ABI provenance, bounded host observations,
  selected glibc inputs and execution scope. Unknown is not an unsupported distro;
  no install-time warning, distro rejection or new host qualification is added.
- Local execution now refuses a confirmed missing ELF PT_INTERP with status 2
  before launch. In 0.3.0, a direct run that passed existing checks and reached a
  failed OS launch returned helper error 1. Update scripts that distinguish these
  statuses. Container execution does not require the host's GNU loader.

### Documentation

- Explain deployment glibc/libgcc prerequisites, symbol-version versus package
  versions, and capability/qualification limits. Container provider preparation
  still requires host GCC/binutils/startup files; container execution supplies
  userland but shares the host kernel.
- Include maintained release-documentation guidance and records added since 0.3.0.
  The historically named [unwinder plan](docs/UNWINDER-0.4.0-PLAN.md) is deferred,
  unassigned future research, not this release. System unwinder, five-clock
  policy, dependencies, MSRV and native-tool floors are unchanged.

[Release notes](docs/releases/v0.4.0.md) · [Source tag](https://github.com/rustcommons/cargo-libc/tree/v0.4.0). Existing historical records below are unchanged.

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
