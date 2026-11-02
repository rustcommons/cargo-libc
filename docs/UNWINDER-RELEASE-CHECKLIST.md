# Unwinder release and glibc-bundle checklist

This is the durable downstream gate for cargo-libc and upstream glibc ABI/source
bundle owners. Adopted for authorized 0.4.0 development on 2026-11-02. **The LLVM
runtime is not integrated or qualified yet.** Checklist presence and host baseline
probes do not make any pending case pass. The current published baseline is 0.3.0. Private LLVM-default wiring is paused:
[initial native proof](evals/2026-11-02-unwind-boundaries.md#initial-native-cross-dso-proof--later-2026-11-02)
shows hidden LLVM breaking U10/U11. That capture’s GNU label is inconsistent
with retained `-fuse-ld=lld`/LLD ELF evidence; GNU qualification remains pending.
Broad host export is diagnostic, not a final policy.

## When this gate applies

Every new/changed glibc release, ABI list, source/header/library bundle, stub or
provider regeneration needs a recorded unwind-impact disposition. Also trigger
requalification for LLVM source/archive/config/patch/build-tool changes,
Rust/std/panic/backtrace changes, Clang/GCC/GNU ld/LLD updates, container image or
sysroot rebuilds, target/features/EH/PIC/visibility changes, generated configuration,
cache/output selection changes, and relevant Python/PyO3/consumer/loader updates.
A new glibc bundle is not exempt as “data only.” Upstream data/source release
checklists must link this gate and obtain relevant downstream evidence before
release. No automatic data or producer-wide version bump is implied.

Repository-only docs edits can reuse execution evidence only with a byte-identity
comparison of affected runtime/package inputs and explicit scope. A docs-only
upstream archive must prove its data/source identity before reuse. Do not invent
runtime passes for a newer-than-host glibc: record link/inspection evidence and
the precise execution gap. Tier 3 actual old-system execution remains deferred;
prebuilt-provider legal clearance and runtime redistribution review are separate.

## Required evidence record

For each release/update, save exact consumer/producer/ABI/source refs and their
source/registry resolution; fixture/command hashes; Rust/Clang/GCC/linker/CMake
versions; LLVM revision/source digest/config/patches/notices; archive/member hashes,
architecture, PIC/EH/thread/visibility settings; selected glibc floors/routes;
cache/profile/content identity. No arbitrary host/system libunwind is a substitute.
Record deterministic archive contents, including thin-member containment refusal.

For each final ELF, retain imports **and exports**, visibility, GNU versions and
DSO owners, ordered NEEDED, interpreter, EH data, relocations/TEXTREL, search paths
and link map with selected archive members. Attribute new libc helper imports to
actual LLVM members and prove each is supported at the target floor; do not demand
clock-only non-clock identity when the runtime legitimately changes. Preserve all
twelve baseline Aero `_Unwind_*` functions and non-unwind compiler helpers.

Separate direct NEEDED, resolved transitive closure and observed lazy loads for the
bounded tested workload. Use fresh subprocesses/checkpoints and loader diagnostics
that retain transient loads, not only final maps. Record actual binding owners for
personalities and the context getters/resume paths. Absence of direct libgcc_s is
not proof it cannot load via libc, a DSO or dlopen. Do not build a general dependency-
closure feature as part of this gate or run host ldd on foreign binaries.

## Permanent acceptance cases and current automation

Every row needs an exact command/fixture/run/result or a precise failure/gap in the
release record. Missing evidence is incomplete, not an implicit pass. These IDs
retain the independent acceptance case numbering; they are verification cases,
not new runtime contracts. Run risk cases on native x86-64 with GNU ld and LLD,
Rust 1.90 and the supported current toolchain, representative debug/optimized
outputs and target floors 2.17/2.22; cover each additional claimed floor/route.

| Case | Required automated assertion | Current command or explicit gap |
| --- | --- | --- |
| U01 | Plain/PIE executable, normal return and local catch: dynamic glibc, archive extraction and bounded no-dynamic-unwind workload. | `probe-unwind-boundaries.py` checks native **baseline** plain/catch only; LLVM candidate fixture pending API. |
| U02 | Nested catch/resume with exact destructor order/once-only cleanup and payload. | Host baseline nested cleanup exists; full candidate resume/order fixture pending. |
| U03 | Thread panic/join then successful threads, bounded concurrent catches/backtraces. | Host baseline panic/join exists; candidate concurrency coverage pending. |
| U04 | First/repeated/threaded force_capture and formatted useful backtraces; capture lazy loads and debug named frames. | Host baseline capture/maps exists; transient binding traces/candidate pending. |
| U05 | Uncaught and double-panic child status/diagnostics with core dumps disabled. | Baseline uncaught exit 101 exists; double-panic isolated fixture pending. |
| U06 | Actual bin/test/doctest/example/bench/install vs cdylib/dylib/rlib/staticlib selection, mixed workspace; ambiguous forms refuse. | `cargo test --lib link_output --locked --offline`, C classifier and provider dispatch tests; probe captures two executable/two shared rustc links and zero rlib links. Full candidate/mixed workspace pending. |
| U07 | Native C host loads Rust cdylib; panic caught within DSO, translated to status and reusable. | Initial native GCC and LLVM variants pass internal catch; no full candidate/route/PyO3 qualification. No panic crosses ordinary C ABI. |
| U08 | Two independent DSOs, both load orders and supported local/global visibility; repeated use and controlled unload. | Native coexistence driver pending exact archive/profile. |
| U09 | Existing PyO3/maturin normal/error/panic conversion, repeated use, mixed bin+cdylib workspace with host build/proc-macro. | Existing Python workflow must be reused for regression; candidate not run. |
| U10 | Executable catch -> cleanup-bearing DSO C-unwind bridge -> executable panic -> originating executable catch, with all guards. | Initial host-SDK hidden LLVM SIGABRT versus GCC pass; nonhidden+broad-export passes diagnostically. Narrow compatible binding pending; no foreign-runtime catch. |
| U11 | DSO catch -> executable C-unwind bridge -> DSO panic -> originating DSO catch, all guards/continued use. | Initial host-SDK hidden LLVM SIGSEGV versus GCC pass; nonhidden+broad-export diagnostic passes. Narrow compatible binding pending. |
| U12 | Diagnostic hidden vs exported policies with real GCC DSO/LLVM executable; personality/context ownership in both phases and deletion. | Real native proof now blocks hidden default; broad export passes three cases with host LLVM interposition, but exposes unrelated symbols. Narrow unwind export/context coverage pending. |
| U13 | Supported native unwind metadata, glibc backtrace and C-only pthread_exit/deferred cancel controls; lazy-load ownership explained. | Native C control fixtures pending integration. No cancellation through Rust/ordinary C FFI; no target C++ support added. |
| U14 | All twelve baseline unwind functions and actual i128/math/native compiler helpers; explicit versioned refs handled or refused intentionally. | Symbol inventory recorded in design; actual extraction/call/helper/floor tests pending. Never blanket-drop libgcc. |
| U15 | Five default libc clocks, mixed legacy/default references and genuine RT calls on GNU/LLD. | Existing clock driver/fixtures are reusable, not a fresh candidate pass; no clock-setting calls executed. |
| U16 | Order/groups/repeated archives/whole-archive/static/dynamic/push-pop/as-needed and RT/script/response protections unchanged. | Classifier tests preserve args; complete executable runtime substitution/state/override checks pending. |
| U17 | Source/container, explicit provider, full sysroot each use validated contained target archives and read-only inputs. | Root helper API/discovery and per-route native execution pending; no silent host/dynamic fallback. |
| U18 | Wrong/missing/truncated/class/config/architecture/symlink/thin-member/cache-corruption inputs refuse; concurrent complete generations. | Producer builder/cache API and consumer adapter tests pending. |
| U19 | Same host/target triple, host build/proc macro, target C/CMake; no shared intermediate policy contamination. | Existing host bypass retained, final-kind scaffolding tested; actual LLVM mixed-workspace gate pending. |
| U20 | Same targetdir old0.3 -> candidate -> changed archive/profile -> candidate again: relink then no-op; shell/TOML stale-config refusal. | Fingerprint/export integration pending content-identified producer API. |
| U21 | Fresh old0.3/new0.4/Zig consumer at `9c248300f1e2ab20ce34dc4b227eb322e33e4c93`, 182 cases, original strict comparator and separate member-attributed unwind assessment. | Parent prepares exact-pair capture later; do not dispatch baseline-only or relax comparator. |
| U22 | Real registry graph/package/unpacked clean locked install and later official archive/published install/docs.rs/tag/notes. | Final runtime and release authorization required; versions/production locks unchanged now. |

A baseline-supported valid C-unwind path failing under the new default is a
material blocker: save exact bindings/context path and obtain a decision. Output
kind separation or hidden symbols alone cannot waive U10/U11. Preserve DSO artifact
link selection while testing the actual host-process bindings: DSOs can resolve
versioned unwind calls to host LLVM. Do not require invariant GCC ownership or
permit broad Rust/application export as a workaround. Native libc controls
may deliberately load GCC in both candidate and Zig; explain workload-specific
scope rather than claim universal absence or conceal accidental fallback.

## Source/data qualification input contract

Pre-publication qualification uses a **disposable consumer copy**. Verify clean
exact consumer and producer commits plus every optional changed glibc-abilists or
glibc-provider-sources checkout against the caller's expected SHA. Apply only the
named first-party crates.io patches in that copy; save before/after manifest/lock
and metadata. Unrelated registry packages/checksums stay fixed. A changed data
crate must demonstrably feed the producer being tested, not remain an unused patch.
Reject unspecified dirty/path/git inputs. Production manifests/locks never contain
these overrides, and no unpublished checksum may be invented.

Current runnable baseline command (existing tools only, fresh evidence directory):

```sh
RUSTUP_TOOLCHAIN=1.90.0 python3 scripts/probe-unwind-boundaries.py \
  --rustc /home/joey/.cargo/bin/rustc --cc /usr/bin/clang-18 \
  --evidence /tmp/unwind-baseline-FRESH
```

This is **not the candidate/data release gate**. It captures native output shapes,
synthetic interposition, core panic/status and final loaded-library snapshots; it
has no LLVM adapter and misses transient loads. The full source/data-override
qualification driver, exact API/build arguments and automated U01–U22 binding/load
assertions are pending producer integration. Data owners must not count this
baseline command or a source-graph preflight as qualified changed-bundle evidence.

## Updates and release completion

For new glibc/ABI/source bundles, freshly compare the selected LLVM archive import
closure against regenerated providers, keep 2.17/2.22 controls, and link/inspect
all added glibc versions. Run U01–04/U07–12/U14–20 on affected floors/routes, U21
and U22 when executable/runtime inputs ship, plus relevant negative/control cases.
Every skipped row needs an explicit justified scope and approval; an unexplained
failure never becomes a pass. Unrelated kernel data does not mandate a duplicate
unchanged kernel matrix, but requires a disposition and relevant affected checks.

Keep fixtures in permanent CI groups: portable dispatch/negative/cache;
GNU+LLD unwind/clock/state; DSO/Python/coexistence/load tracing; floors/provider
routes/host isolation/provenance; fresh consumer; package/install/docs. Cap workers,
use child timeouts, retain failures, and run the full applicable suite once on the
frozen final source; docs-only follow-ups use exact equivalence evidence.

Follow [release policy](RELEASING.md): independent review, normal merge, registry-
only resolution, package and clean unpacked install, coordinator-only publication,
official checksum/source, published install, exact immutable tag, docs.rs and
reviewed GitHub notes. Runtime license/provenance review is required independently
of prebuilt provider clearance; no new compiled-runtime distribution is implicit.
Update AGENTS, guide/architecture, trace, Unreleased and release evidence together.
