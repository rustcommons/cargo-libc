# cargo-libc 0.4.0 — development scope

Not released, version not bumped. The user authorized a default static LLVM
libunwind for supported standalone target executables, retaining panic=unwind
and dynamic glibc. No public opt-in mode is planned. No final producer/runtime
archive API or qualified consumer candidate exists yet.

The [integration design](UNWINDER-0.4.0-PLAN.md) and canonical
[future-release/glibc-bundle checklist](UNWINDER-RELEASE-CHECKLIST.md) define scope.
Final target executables include Rust test/doctest harnesses and target C/CMake
executable probes. Preserve existing cdylib/dylib/PyO3/CMake-shared, native host and
shared-intermediate behavior, clocks and caller link state. Valid supported
cross-DSO C-unwind behavior is a release gate, not waived by output classification.

## Current increment

- Shared final-output classifier for direct/forwarded controls; focused tests.
- Native baseline/marker/output-kind/C-helper probes with saved failure evidence.
- AGENTS/guide/architecture/trace/checklist linked to CL-REQ-028–030.
- [Dated results and limits](evals/2026-11-02-unwind-boundaries.md): aarch64 host
  baseline only, no LLVM/x86-64 candidate qualification. Runtime selection is not
  yet active; production manifest/lock/version remain unchanged.

## Native binding blocker

[Initial verified proof](evals/2026-11-02-unwind-boundaries.md#initial-native-cross-dso-proof--later-2026-11-02)
shows hidden LLVM passing internal DSO catch but failing valid U10/U11 round trips.
Nonhidden+broad host export passes three prototype cases, with DSO/GCC versioned
getters interposing on host LLVM, but broad Rust exports are unacceptable as policy.
DSO artifact selection remains distinct from its actual host-process bindings.
Private-default wiring, full suites and dispatch remain paused pending a concrete
minimal-export/binding design; no runtime behavior has been changed here.

## Pending qualification

Producer-owned pinned archive/provenance/cache and complete helper/personality
support; consumer exact substitution/config/fingerprints and final command
verification; all U01–U22 native/portable/DSO/Python/state/floor/route/negative gates;
fresh same-targetdir upgrade and old0.3/new0.4/Zig comparison with unchanged strict
comparator and member-attributed import changes. The final source/data qualification
driver must prove changed bundles actually feed the selected runtime/provider.

Only after the final runtime/source pair and focused gates should one full
applicable suite run on frozen source. Real registry adoption, package/unpacked
locked install, independent review and separate merge/version/publication GO
remain required. Later official archive, published install, immutable tag, docs.rs,
changelog and GitHub Release follow the maintained policy. No source/data bundle
release or producer-wide lockstep bump is assumed.

Tier 3 real old-system and prebuilt legal gaps remain unchanged. Scope claims
will name actual tested executable/process workloads; no universal plugin/FFI or
never-dlopen-libgcc guarantee is made.
