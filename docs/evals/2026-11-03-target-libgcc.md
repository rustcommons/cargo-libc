# Target libgcc checker source qualification

Historical source-stage evidence, 2026-11-03: no version or registry adoption
was assigned during these tests. Subsequent [0.5.0 release preparation](../RELEASE-0.5.0.md)
adopts published libc-abi-tools 0.5.0 and records final registry gates separately.
The source outcomes below and immutable 0.4.0 package remain unchanged.

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

29 native checks passed on x86-64 WSL GNU userland using GCC 13.3.0, GNU bfd 2.42; LLD was unavailable and no LLD result is claimed. Fixtures were compiled/linked and inspected, never
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


## Review fixes and final exact-pair qualification

Independent review found two missing controls in the first candidate: a wholly
unversioned runtime's unmodeled fallback, and optional symbol errors masking
independently valid node failures. The earlier v2 design itself conflicted about
the versionless case; the unreleased contract now explicitly distinguishes no
DT_VERDEF (fallback unresolved) from a versioned DSO missing a mandatory node.
No loader emulation or producer parsing/matching API was added.

Fixed implementation: `972783dfd1571fb8e23385b88d147ef42c7c6dfe`, with unchanged
producer `2c70e47954f328e55950d738d5a9295de7c44d67`. Independent focused code/contract
review passed; its seven targeted probes found both issues resolved. The strict
version phase is independent from optional dynsym/hash/relocation validation;
separate completion flags preserve proven node failure and unknown symbols.

The full source qualifier passed again at this exact pair: 102 library tests,
nine process tests, the zero-doctest command, 36 static native cases, Rust 1.90
build/docs and pinned 1.99 formatting/Clippy. Legacy text/JSON also matches actual
released 0.4.0 in the named control. The producer's default-feature `cargo test
--locked --offline` additionally passed 72 tests including doctests, with two
explicit ignores; default-feature Clippy passed. The no-default-feature 23-test
result remains separately recorded rather than summed into a unique-test claim.

Final source proof: 150 member hashes, 16,028,453 bytes, SHA-256
`0b7cd40fbf354141bde0d10205f53358685be26b25dd4e2e7880dab6fecc7ef8`.
It includes final source archives, actual command logs, default-feature producer
checks, fixture results and installed-0.4.0 legacy comparison. LLD was not
available locally; no LLD or Windows candidate result is claimed. The conditional
LLD control and portable unit tests remain permanent future gates.

Final implementation package-contents audit: 28 files, 84,776 bytes, clean exact
VCS `972783d`, SHA-256
`3fd755d1971450b3fa671c92ab9890a8a8a1a193508e253581e87bc4d96db4e1`.
This is still `--no-verify` contents evidence, not registry compilation. Producer
package verification passed at its exact clean API ref. Subsequent excluded
documentation/instruction-only updates do not change the tested package inputs;
new commit VCS metadata naturally changes an archive's overall hash.

No source was merged or published, no versions/minima/lock changed, and the
registry-only build remains intentionally blocked until hash-API publication
and real consumer adoption. No runtime/default/old-system safety claim follows.
Peak working-memory optimization through staged buffer release remains follow-up
work; current bounded reads are not misrepresented as a whole-process memory cap.


## Later real-registry candidate qualification

The source-stage outcomes above are retained unchanged. After actual producer
0.5.0 publication, candidate `c5f41485af05bd311c15f567a6bd9310a3154237` adopted
only libc-abi-tools 0.5.0 while preserving all other tool/data locks. Its source
bytes equal reviewed `972783d`. [Q50 release evidence](../RELEASE-0.5.0.md) records
unpatched verified package/clean-unpacked install, source/registry checks, 36
installed static cases and actual0.4 legacy comparison plus native Windows CLI
controls. The initial stream refusal caused by a driver RUSTDOCFLAGS leak is
retained separately from the clean-environment resume; no source/package/install
change was needed. The later 228-member packet SHA256 is
`461de6aff75d0009e11cd5d038346db123012aa70acc9067d485eb9c25894322`.
This is candidate qualification, not an official cargo-libc 0.5.0 archive or
published installation. All runtime/platform and memory limitations remain.
