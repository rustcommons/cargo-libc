# Target libgcc checker source qualification

2026-11-03. Unreleased implementation; no version, registry minimum or publication
is assigned. This record does not update the immutable 0.4.0 package.

## Exact source pair and commands

- Consumer: `d50ef3452208450d97d3b32db9370895f13f60e0`.
- Producer byte-hash API: `2c70e47954f328e55950d738d5a9295de7c44d67`.
- Driver: `scripts/qualify-target-libgcc.py CARGO_CHECKOUT PRODUCER_CHECKOUT EVIDENCE_DIR`.
- The driver snapshots clean exact heads and applies a root-only source patch in
  a disposable consumer copy. No tracked manifest/lock or registry graph is
  adopted by this validation. Ordinary registry compilation is pending the
  published API and deliberate dependency adoption.

Rust 1.90: 23 producer root-library tests with default features disabled; 100
consumer library tests and nine process regressions; doctest command (zero
doctests); build and warnings-denied docs. CI-pinned Rust 1.99: formatting and
warnings-denied Clippy. Both local release-documentation checks passed.

The source-qualification driver passed all steps. An initial Clippy rejection of
the new remainder spelling was corrected with the MSRV-supported
`is_multiple_of` operation; the final exact-pair run is green. An earlier run
against a dirty scratch checkout refused before testing. Neither failed attempt
is treated as a pass.

## Static native controls

29 native checks passed on x86-64 WSL GNU userland using GCC 13.3.0, GNU bfd 2.42
; LLD was unavailable and no LLD result is claimed. Fixtures were compiled/linked and inspected, never
executed. Controls cover exact/absent/wrong-owner interfaces, weak and
zero-reference nodes, nondefault versions, SysV/GNU hash tables, sectionless
files, forged counts/relocation bounds, malformed version hashes, target-own
GLIBC failure with independent GCC match/unresolved results, missing files,
scripts, FIFOs, symlinks and non-UTF-8 paths. The actual Rust checker binary's
twelve GCC imports match the explicitly supplied host runtime; this is not a
claim about other applications or deployment systems.

Separately, final-code ordinary inspect text and JSON were byte-for-byte equal
to the installed released 0.4.0 binary on the same fixture. Optional GCC report
fields also preserve the legacy root metadata. No optional exact-parser stage
runs without the new option.

The source proof packet has 128 member hashes and SHA-256
`53f185c9076abb7f7b44f57b43ee62c27789e5eac95292b5688f4b860065d150`.
It includes exact source archives, qualification logs/results, fixture inputs
and outputs, tested binary and legacy-equivalence checks. The packet's filenames
are portable copies; a non-UTF-8 filename and explicit symlink fixture were copied
as ordinary bytes for transport. The committed driver reproduces native path
cases in a fresh directory.

## Package and release boundary

Producer clean package verification passed. Consumer package contents were
audited with `cargo package --no-verify --locked --offline`: 28 files, both new
Rust modules included, scripts/docs excluded, clean VCS, unchanged registry lock.
This contents-only audit is not unpatched package compilation. Source builds use
the unpublished producer API; real registry package/install gates remain pending.

The [contract](../TARGET-LIBGCC.md) and CL-REQ-032 define bounds and outcomes.
No target payload, IFUNC, loader or plugin was executed. No real old-system,
musl/static-host, dependency closure, loader-selected-runtime, GCC/LLVM coexistence
or general safety-to-run qualification is claimed. Versions, runtime defaults,
provider inputs and linker behavior remain unchanged.
