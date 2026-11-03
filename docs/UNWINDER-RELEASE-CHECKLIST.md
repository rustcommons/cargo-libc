# Unwinder release and glibc-bundle checklist

This is the canonical corner-case inventory and downstream release gate for
cargo-libc and upstream glibc ABI/source bundle owners. Adopted 2026-11-02;
corner-case/evidence update 2026-11-03. **Current system unwinding is unchanged.**
LLVM preparation, lifecycle contracts and proposed selectors do not establish an
application policy, implementation pass or release GO. The earlier executable-
default goal remains blocked by measured mixed-runtime failures; no selector is
wired or selected as the eventual release policy.

For users: passing the Aero command comparison does not establish panic, native
cleanup, plugin or Python compatibility. A hidden/static LLVM executable can
break otherwise valid cleanup-bearing calls, while an unchanged extension can
bind differently when loaded by a different host. Preserve shared/PyO3/Python-
embedding and host behavior; do not silently narrow existing supported use.
Known C cleanup/mixed-context failures block a changed default. This inventory
does not conclude that every possible coherent implementation is impossible.

## Valid fixtures and prohibited substitutes

U07 catches a panic **inside** its owning Rust DSO before returning through ordinary
`extern "C"`. U10/U11 use deliberately unwind-permitting `extern "C-unwind"`
bridges, cleanup-bearing crossed frames and the **originating runtime's** catch and
payload ownership. Assert exact drop/cleanup counters and subsequent reuse.
A panic crossing non-unwind `extern "C"`, a foreign-runtime Rust panic caught as if
locally owned, or a guessed opaque-context/layout conversion is not a valid
positive oracle. Target C++ compilation remains unsupported; a foreign C++ DSO
behind valid C APIs/internal catches is still a loader/binding hazard to assess,
not permission to unwind foreign exceptions through forbidden boundaries.

U13 cancellation/`pthread_exit` controls use entirely **C worker stacks**, plain and
`-fexceptions`, deferred cancellation and exactly-once cleanup. Never cancel or
force-unwind through Rust worker frames as a passing test. These rules distinguish
valid baseline regressions from prohibited-ABI negative controls.

## When this gate applies

Every new/changed glibc release, ABI list, source/header/library bundle, stub or
provider regeneration needs a recorded unwind-impact disposition. Also trigger
requalification for LLVM source/archive/config/patch/build-tool changes,
Rust/std/panic/backtrace changes, Clang/GCC/GNU ld/LLD updates, container image or
sysroot rebuilds, target/features/EH/PIC/visibility changes, generated configuration,
cache/output selection changes, and relevant Python/PyO3/consumer/loader updates.
A statically incorporated runtime/security fix requires rebuilding affected
archives **and relinking/redeploying consuming executables**; updating a shared
runtime package alone cannot replace code embedded in old executables. Record
source/patch/archive/output identities and affected consumers. This is a future
release/update obligation, not freshness automation already implemented here.
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

This is the canonical inventory; existing U01–U22 meanings remain stable. Each row
records trigger/impact, baseline oracle, expected boundary, bounded evidence/gap
and release disposition. Evidence keys below identify dated results, not new
requirements. A baseline-supported valid failure blocks a changed default; do not
silently turn it into a passing negative test or an unsupported feature. No new
execution was performed just to document this inventory.

| Case | Trigger / user impact | Baseline oracle | Expected behavior / boundary | Actual bounded evidence / remaining gap | Release gate |
| --- | --- | --- | --- | --- | --- |
| U01 | Executable recipe/PIE/floor changes; startup or local panic failure. | Same fixture under published system runtime; normal return and local catch. | Keep panic=unwind and dynamic glibc; inspect extracted members, imports and bounded lazy loads. | [System/marker baseline sanity](evals/2026-11-02-unwind-boundaries.md) only; no candidate/floor runtime pass. | Required core executable gate for changed runtime; U13 failures cannot be waived. |
| U02 | Nested panic, rethrow/resume or exception disposal changes; leaks/double drops. | Same-origin nested catch/resume with payload identity and once-only ordered drops. | Preserve cleanup/order and ownership through resume and disposal. Do not assume every Rust 1.90 catch calls `_Unwind_DeleteException`; qualify actual lifetime paths. | Baseline fixture sanity plus bounded round trips E2/E3; full candidate lifetime/rethrow coverage pending. | Required cleanup/ownership gate; no guessed Rust-class or opaque-context lifetime fix. |
| U03 | Thread/std/runtime updates; thread panic corrupts later work. | System-runtime panic/join then successful threads and repeated local catches. | Exactly-once cleanup and continued use; no cancellation through Rust frames. | Baseline panic/join sanity only; candidate concurrency pending. | Required thread/reuse gate for claimed executable scope. |
| U04 | First/repeated/threaded backtrace or loader change; missing frames/crash/lazy GCC load. | System fixture with useful named frames and fresh subprocess loader capture. | Capture first/repeated behavior and transient loads, not only final maps or DT_NEEDED. | E6 bounded Zig C backtrace passes while exit/cancel fail; candidate Rust backtrace/concurrency remains pending. | Required backtrace/load gate; one successful trace does not waive cleanup failures. |
| U05 | Uncaught/double panic or diagnostics change; wrong process status/output. | System children with core dumps disabled and exact exit/stdout/stderr. | Retain deliberate panic statuses, useful diagnostics and isolated failure handling. | Baseline uncaught 101 recorded; full double-panic/candidate gate pending. | Required status/failure gate, separate from ordinary Aero errors. |
| U06 | Compiler forwarding/output/PIE/LTO/strip changes; wrong CRT or runtime in a DSO. | Actual completed Clang command vs real ELF for executable/PIE/shared/relocatable; rustc target kinds. | Preserve operand slots and caller flags; classify final output, not filename. Cover bin/test/doctest/example/bench/install and shared intermediates. | E8 exact cce9718 scoped GO; parent x86 Rust 1.90/Clang18 reports nine tests including native controls. No LLVM selector. LTO/strip/runtime integration still pending. | Required output gate on GNU BFD/LLD; classifier pass alone is not runtime qualification. |
| U07 | Rust DSO loaded by C/native host; internal panic/error leaves host unusable. | [bridge/host fixture](../tests/fixtures/unwind-coexistence/README.md), system runtime; internal DSO catch and repeated calls. | Catch/own payload within DSO before ordinary C return; preserve counters/status and reusable host. | E2/E3 bounded internal catches pass. These are one Rust DSO mechanism, not full Python or route qualification. | Required existing DSO behavior; no ordinary-C panic crossing as a success oracle. |
| U08 | Multiple plugins, local/global/deep binding, handle lookup or load/unload order changes. | Matched system-runtime DSOs with both load orders, RTLD_LOCAL/GLOBAL/DEEPBIND where applicable and direct handle lookups. | Preserve baseline-valid calls and cleanup. Unload only after active frames/threads/callbacks/exception use end. A public shim cannot guarantee interception of handle-direct calls. | E3 ordinary versioned interposition and E4–E7 real-GCC selection demonstrate hazards; arbitrary deepbind/handle-direct Resume/getter and multi-DSO compatibility unqualified. | Required plugin/loader gate for supported routes; unsupported claim needs explicit decision, not silent exclusion. |
| U09 | PyO3/Python/embedding or mixed-workspace changes; extension regression or new build prerequisites. | Existing Python/maturin workflows: normal/error/internal panic translation, repeated use; ordinary extension-only and embedding host separately. | Shared/Python/host artifacts retain baseline selection. Embedding host may alter actual bindings; extension-only builds must not prepare LLVM. | No fresh candidate Python/embedding/multiple-extension pass. E2/E3 do not prove general Python failure or success. | Required existing Python regression gate before changed runtime claims. |
| U10 | Executable-origin panic crosses a cleanup-bearing DSO C-unwind bridge. | Exact executable catch -> DSO bridge -> executable panic -> original executable catch; system baseline and exact counters. | Payload caught/destroyed by originating runtime, all crossed guards run once, process remains usable. | E2 hidden LLVM SIGABRT; E3 eighteen-unwind-only exports pass actual BFD/LLD for this fixture. Broader compatibility still blocked. | Required valid round-trip gate; bounded E3 pass cannot waive E4–E7. |
| U11 | DSO-origin panic crosses executable C-unwind bridge; wrong ownership or cleanup. | Exact DSO catch -> executable bridge -> DSO panic -> originating DSO catch; system baseline and exact counters. | Same-origin catch/disposal and once-only crossed guards; no foreign-runtime catch assumption. | E2 hidden LLVM SIGSEGV; E3 bounded narrow-export round trip passes BFD/LLD. Full candidate remains pending. | Required opposite-direction round-trip gate; preserve U11 meaning. |
| U12 | Visibility, personality/getter/Resume/delete binding, registration or DSO initialization order changes. | Matched system and diagnostic hidden/narrow-export variants with both unwind phases and actual owners. | Preserve observable valid behavior; DSO artifact inputs do not guarantee invariant GCC process bindings. No broad application export or opaque-context reinterpretation. | E3 resolves bounded Rust round trips. E5/E7 trace real GCC forced unwinding into local LLVM personality/getter paths and faults; no exact private-cursor-field claim. | Mixed-context failures block a default change; not expected-green negative controls. |
| U13 | Plain versus C -fexceptions pthread_exit/deferred cancel/cleanup or libc loading changes. | Same [native C worker](../tests/fixtures/unwind-coexistence/native_load_controls.c), system baseline: exact cleanup count, join result, cancellation value and native backtrace. | All worker frames C; inspect engine/personality/getter/resume owners and lazy GCC loads; no Rust-frame forced cancellation. | E4 plain9/9 pass, EH GCC 3/3 pass; hidden cancel and narrow-export exit/cancel SIGSEGV. E5 helper-present BFD/LLD still fail. E6/E7 Zig C exit/cancel also fail on host glibc 2.39; actual Rust/Zig-driver control pending. | Active compatibility blocker. Aero 182/local catches/direct libgcc absence do not waive it. |
| U14 | Compiler/helper/personality/GNU version changes; undefined helper, split runtime or raised floor. | System native/Rust symbols and member maps, including i128/math, C personality and all twelve Aero unwind imports. | Retain non-unwind helpers; validate names, versions, DSO owners and actual call semantics. Handle explicit GNU references intentionally; never blanket-drop libgcc. | E1 has twelve versioned unwind imports; E4 adds __gcc_personality_v0/Resume. E5 selects a helper yet still fails. Zigbuild rewrites -lgcc_s; no semantic parity follows. | Required completeness/floor/binding gate; unresolved-symbol lists alone are insufficient. |
| U15 | Clock/provider/ABI changes while replacing runtime; legacy clocks or real RT regress. | Published clock default, mixed legacy/default refs and timer/real RT fixtures with actual version owners. | Preserve all five libc defaults, explicit old versions, real RT APIs and faithful upstream data. | Published 0.3 clock evidence retained; not fresh unwind-candidate qualification and no clock-setting fixture executed. | Required clock regression gate on affected floors/GNU BFD/LLD. |
| U16 | Archive/state/order/linker overrides or symbol-version flags change; retention/binding changes. | Same groups/repeated archives/whole-archive/static-dynamic/push-pop/as-needed fixtures under baseline. | Keep caller order/groups/state and response/script refusal boundaries; no new forced as-needed or unrelated helper selection. | Classifier preserves argv; complete runtime substitution/state tests pending. | Required link-state gate, including GNU BFD/LLD, PIE/non-PIE, LTO and final stripped/unstripped output identity. |
| U17 | Source/container/explicit provider/fullsysroot/header/runtime change; host input selected accidentally. | Selected raw libc and declared independent provider/header roots; baseline fullsysroot and symlink controls. | Read-only inputs, exact target/provenance and no host/archive fallback; references are not implicit compile inputs. | E9 scoped construction GO only. Fullsysroot contained-alias/multiarch staging and application route qualification remain open. | Required affected-route gate; preserve current system/fullsysroot behavior rather than silently removing it. |
| U18 | Malformed/wrong target/thin archive/unsafe path/cache corruption or concurrent publication. | Corrected preparation fixtures and read-only snapshots; known-good generation as baseline. | Refuse unsupported input/cache state; consumer invalidation never writes/repairs producer cache; no hostile-tool sandbox claim. | E9 fixes earlier containment defect within explicit staging scope. Recorded-validator API is design, not implemented/qualified here. | Required corruption/containment gate before integration; no silent repair/fallback. |
| U19 | Same-triple host/target, C/CMake or mixed lib/bin workspace changes. | Existing host build/proc-macro/native SDK and shared CMake controls; actual link-stream classification. | Host/shared/partial/intermediate runtime selection stays baseline; no eager LLVM prerequisites for system/extension-only users. | Host bypass/classifier are present. Candidate lazy activation/mixed-workspace/CMake/runtime preservation pending. | Required isolation gate; native build-system freshness is separately demonstrated, not inferred from Cargo. |
| U20 | Recipe/profile/header/tool/resource/libc/archive/cache/config changes, including same-path/mtime. | [Approved lifecycle v2 DESIGN](evals/2026-11-02-llvm-lifecycle-design-v2.md), exact same-targetdir before/after artifacts and no-probe controls. | Separate R/I/A/E; every distinct observed change advances E even pending; missing receipt unknown; bind final output digest/length. No cache repair. Plain Cargo explicit refresh; CMake/Ninja same-directory relink. | E10 contract-content GO only; ledger/guards/automatic managed freshness are not implemented. Recorded API clarifications are design feedback. | Required future lifecycle gate; no baseline or contract approval counts as a test pass. |
| U21 | Fresh consumer/runtime/compiler/link recipe changes; apparent application parity hides closure gaps. | Pinned Aero 9c248300f1e2ab20ce34dc4b227eb322e33e4c93, old 0.3/candidate/Zig and unchanged strict comparator. | Retain182 behavior cases plus separate attributed import/member/closure assessment; preserve comparator and honest tool/source pins. | E1 clock-era old 0.2.4/new0.3/Zig182 match but strict exits1; not a fresh0.4 runtime pass. Same-fixture actual Rust/Zig C-EH comparison is next, not complete. | Required final exact-pair consumer gate when authorized; no baseline-only/unchanged matrix now. |
| U22 | Release or runtime security/source/tool/glibc bundle update; stale embedded code or mismatched archive. | Exact source/registry/package/locked install and published archive/tag/docs records; compare affected consumer outputs. | Every bundle/update needs disposition; security fixes rebuild archives AND relink/redeploy consumers. Track source/patch/archive/output provenance, licenses and declared compiler closure. | Preparation/proposals are not release GO; no0.4 version/lock/publication. Tier3 deferred and prebuilt/runtime legal gates remain separate. | Required release gates plus affected U01–U21 on glibc 2.17/2.22/new inputs. Unavailable tier3 is an explicit gap, not routine blocker. |

### Evidence key and scope

- **E1 — actual Aero clock-era comparison:** run [41378111334](https://github.com/rustcommons/cargo-libc/actions/runs/41378111334), Cargo source `29582bc7af35cba1f741f989b47bc520979e13ae`, Aero `9c248300f1e2ab20ce34dc4b227eb322e33e4c93`; Zig0.17.0/cargo-zigbuild0.23.4/Rust1.99.1. Both Cargo variants matched 182/182 behavior; strict comparisons exited1 on requirements. [Source/artifact audit](evals/2026-11-03-zig-unwind-parity.md), not current-runtime or old-SLES execution.
- **E2 — first native coexistence:** [retained proof](evals/2026-11-02-unwind-boundaries.md#initial-native-cross-dso-proof--later-2026-11-02), packet SHA `841673515218814d49825314791e57c403ca331309a59bd43ad819e36f61d283`. Rust 1.90/x86 hostSDK; capture labeled GNU actually used LLD 20.1.8. GCC U07/U10/U11 pass, hidden U10 abort/U11 segfault. Broad export is diagnostic only.
- **E3 — corrected narrow exports:** [follow-up](evals/2026-11-02-unwind-boundaries.md#corrected-minimal-exports-and-native-c-eh-follow-up--2026-11-02), packet SHA `b9c9434aae3fcdeace10a0869075b7d0560181996151c9771a07538bd6316031`. Actual BFD 2.42/LLD20.1.8 and eighteen unwind-only definitions pass bounded U07/U10/U11, not general interoperability.
- **E4 — C EH:** same follow-up and [machine record](evals/2026-11-02-unwind-minimal-and-c-eh.json), packet SHA `c6e1b66ef47aa5e1509b7553ecdb29e297c6e6d234ba7cd74cba62226bcfea46`; fixture SHA `309df88609a84558680e544d3be77c9935077b6ac650bf0726bdca1106336766`. Plain 9/9 pass; EH GCC 3/3 pass while hidden cancel and narrow-export exit/cancel fail under BFD. No Rust worker frames.
- **E5 — helper-present native failure:** reviewer-supplied 2026-11-03 synthesis; packet SHA `d37ee43cf662249185972ba56b96324afcaf2b87219c374ad1ef4fcdeb492fba`, helper archive SHA `b96227b64097a311305eab92966607770a6441c4df72dbf967a4fcf5d9ad1237`. Selected/hidden helper still fails exit/cancel on BFD/LLD. Raw packet is not stored/audited here; see [attribution/limits](evals/2026-11-03-zig-unwind-parity.md#reviewer-supplied-native-findings).
- **E6/E7 — actual Zig C and GDB:** reviewer-supplied packet SHA `daab0ea02a5c2c56d359f75e67034656d9f270c2eb7128719d6048a7bc0863bf`; binary SHA `14a27cb5d6d8cf26a51e2a808d73b7b69d6bddc714868169dc7fe5d9ca6fe519`. Zig 0.17.0/vendorClang22.1.8, target glibc2.22, explicit `-lunwind`, host glibc2.39: backtrace passes, exit/cancel SIGSEGV; original no-`-lunwind` failure retained. `native-zig-pthread-exit.gdb.log` closes the exact-stack gap: real GCC `_Unwind_ForcedUnwind` -> Zig-local `__gcc_personality_v0` -> local LLVM `_Unwind_GetLanguageSpecificData` -> `__unw_get_proc_info` -> stack-address SIGSEGV. This is C-fixture/host2.39 evidence, **not** Rust-driver/Aero/glibc2.22-runtime/all-Zig evidence or private-field decoding. Raw log/packet is not stored here; attribution and remaining gap are in the linked audit.
- **E8 — output classifier:** [operand audit](evals/2026-11-02-linker-operand-grammar.md), exact `cce9718b8b07d363c85ba6b83b06ae4e87b854ff`, scoped source GO plus parent x86 Rust 1.90/Clang18 nine-test pass. No runtime selection/compatibility follows.
- **E9 — producer preparation:** exact [558f9cf](https://github.com/rustcommons/linux-abi-tools/tree/558f9cf6ba017c9b926d6c1390494ad0c1c8e525), independent scoped source/API/package GO after c7 containment NO-GO. It covers explicit construction/provenance/cache/materialized-input scope, not linking/defaults, complete compiler closure or fullsysroot staging.
- **E10 — lifecycle/read-only validation:** [approved v2 DESIGN](evals/2026-11-02-llvm-lifecycle-design-v2.md), canonical SHA `436853b803d1e30c439a828853a77e06eee3efa3a542cbb9996cdc24a2ff8fb5`; [three validator contract clarifications](evals/2026-11-03-zig-unwind-parity.md#design-feedback-kept-separate). Contract-content approval is not implementation/test evidence. Plain Cargo refresh and native same-directory relink remain future demonstrations.

These cases support [CL-REQ-028–030](REQUIREMENTS.md#requirements); they do not
renumber those IDs or accept a new public runtime contract. Record release/update
outcomes with exact source/commands and justified scope. Old target-floor imports
or a modern host/container run are not tier3 old-system qualification. Existing
prebuilt-provider legal clearance and static-runtime redistribution/licensing
review remain independent gates.

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

Durable [coexistence/C fixtures](../tests/fixtures/unwind-coexistence/README.md)
and `scripts/probe-native-c-unwind.py` provide diagnostic commands for both C
compile variants with archive hashes, explicit BFD/LLD, raw ELF/loader/failure
capture. The copied minimal Rust wrapper explicitly selects the final linker.
They remain prototype host-SDK diagnostics and must not run as broad matrices
while runtime-source/helper research is unresolved.

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
