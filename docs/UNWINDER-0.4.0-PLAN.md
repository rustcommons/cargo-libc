# Cargo-libc 0.4.0 executable LLVM unwinder integration

Status, 2026-11-02: **implementation authorized and under development, not released
or qualified**. The user selected statically linked LLVM libunwind by default for
supported standalone target executables, preserving panic=unwind and dynamic
glibc. This supersedes the earlier proposal's GCC-versus-LLVM and opt-in/default
questions. No public runtime knob is planned. Product version and production lock
remain unchanged until the exact implementation is reviewed and release-qualified.

LLVM-default wiring remains paused after native C EH failures. The
[initial proof](evals/2026-11-02-unwind-boundaries.md#initial-native-cross-dso-proof--later-2026-11-02)
blocked hidden LLVM; the [corrected follow-up](evals/2026-11-02-unwind-boundaries.md#corrected-minimal-exports-and-native-c-eh-follow-up--2026-11-02)
passes Rust round trips with narrow unwind exports on BFD/LLD but still crashes
C `-fexceptions` cleanup paths. No full-suite/consumer dispatch proceeds before a
concrete native helper/context solution. Broad Rust export remains diagnostic only.

## Concrete consumer integration

Classify the final link, not the Cargo command, filename suffix or an entire
workspace. The first source increment shares `link_output::classify` between
Rust provider dispatch and target C classification; it reads direct/forwarded
controls and preserves argument bytes. This increment does **not** yet select an
LLVM archive. Final completed-Clang command verification must confirm the chosen
kind before the runtime policy executes.

| Actual unit/output | Runtime selection for 0.4.0 | Isolation boundary |
| --- | --- | --- |
| Target bin, example, unit/integration/bench/doctest harness, install output; C/CMake executable probes | Validated static LLVM unwind at the final executable link | No Rust panic=abort or static glibc workaround. PIE/non-PIE classification is separate from executable versus DSO. |
| Target cdylib/dylib, PyO3/maturin extension, C/CMake shared output | Preserve existing artifact/runtime link inputs and exports | No executable-only policy injected into DSO links. Loaded unwind bindings can resolve to host LLVM; preserve valid behavior through qualified host-process binding. |
| rlib/staticlib/object compile, partial/relocatable link | No executable-only runtime injection | Shared intermediate code can feed both bin and DSO in one workspace; do not contaminate compilation flags or intermediate archives. |
| Native build script/proc macro/host C | Existing native compiler/runtime | Preserve the target-only rustflags and build-script marker; equal host/target triples do not identify a target unit. |

The user-facing default is selected, but the exact link mechanism remains gated
on the producer API and real cross-DSO evidence. Runtime changes belong only in
final helpers (`libc-linker`, target `libc-cc`), including C's implicit default
libraries. No global `--unwindlib`/`-static-libgcc` flag or linker search prefix may
silently alter shared outputs or mask a named library. Preserve original input
order, groups/repeated archives, static/dynamic/push-pop/as-needed and clock overlay
semantics; use precise substitution and verify the completed command.

The previous `-static-libgcc` shortcut does not override explicit Rust `-lgcc_s`,
and driver defaults may be ignored under `-nodefaultlibs`/`-nostdlib`. Stable Rust
1.90 rejected `-C link-self-contained=+unwind` in the supplied
[planning probe](evals/2026-11-02-unwinder-plan-probes.json); neither nightly nor an
MSRV increase is a solution. GCC static archives are a bounded mechanism reference,
not a competing product decision. Never select system nongnu libunwind, an arbitrary
host archive or a runtime merely because a similarly named file exists.

## Producer helper needs and private consumer adapter

Producer research/preparation owns pinned LLVM source/build inputs, validated
archive assembly, content cache, immutable atomic creation and notices. Cargo must
reuse that helper rather than duplicate a CMake/LLVM builder. Required returned
information (exact API still pending):

- Target triple/features, selected libc/floor and original provider identity;
  archive path(s), strong content/member hashes and policy/build identity.
- Source revision/tarball digest, patches/configuration, builder/tool versions,
  PIC/EH/thread/visibility decisions and license/provenance metadata.
- Archive visibility and supported narrow unwind-export policy must be part of
  content/build identity. A compile-hidden archive cannot simply be made
  interposable with broad link export; actual compatible binding must be tested.
- Complete provided/required unwind symbol inventory, compiler helper/personality
  strategy, member attribution and supported boundaries; controlled archives
  must not have escaped thin-member dependencies or an unverified host fallback.
- Read-only cached inputs that can be revalidated by helpers and exported through
  generated shell/TOML private configuration; missing/corrupt/incompatible inputs
  must refuse rather than silently select dynamic GCC or another runtime.

The twelve versioned Aero baseline imports are `_Unwind_Backtrace`,
`_Unwind_DeleteException`, `_Unwind_GetDataRelBase`, `_Unwind_GetIP`,
`_Unwind_GetIPInfo`, `_Unwind_GetLanguageSpecificData`, `_Unwind_GetRegionStart`,
`_Unwind_GetTextRelBase`, `_Unwind_RaiseException`, `_Unwind_Resume`, `_Unwind_SetGR`
and `_Unwind_SetIP`. Name completeness is necessary, not sufficient: exercise
actual contexts/encodings, cleanup/resume and deletion. LLVM's implementation of
some base getters can be path-sensitive; do not infer parity from symbol names.

Aero is not the whole helper contract. Preserve non-unwind compiler helpers
(i128 division/remainder/conversions and actual native C helper calls). Native C
`-fexceptions`/cleanup can need `__gcc_personality_v0`; the producer's LLVM 23.1.4
source investigation notes that its libunwind snapshot lacks Zig's vendored
`gcc_personality_v0.c`. A qualified compiler-rt/personality or other supported
helper strategy must be explicit. Follow-up proof confirms the gap: plain C controls pass, while `-fexceptions`
private cancellation and minimal-export exit/cancellation segfault. The C object
needs `__gcc_personality_v0`, still supplied by libgcc_s; C stacks have no Rust
frames. [Exact failed cases and fixtures](evals/2026-11-02-unwind-boundaries.md#corrected-minimal-exports-and-native-c-eh-follow-up--2026-11-02)
are mandatory regressions, not optional controls. No blanket removal of
`-lgcc`/`-lgcc_s`, guessed
host helper archive or accidental dynamic fallback is acceptable. Unsupported
explicit versioned/runtime overrides need a specific diagnostic, not rebinding.

## Visibility and cross-DSO design gate

Zig's pinned builder source sets `_LIBUNWIND_HIDE_SYMBOLS`, hidden visibility and
`_LIBUNWIND_IS_NATIVE_ONLY`; that is a mechanism reference, not proof of safe
GCC/LLVM context mixing. The exact Zig binary/package provenance must be recovered
in fresh comparison rather than equating a reference source to old Zig 0.17.0.

A static archive exporting unwind symbols may interpose on existing DSOs; hiding
it can instead leave a DSO personality/getter bound to GCC while the initiating
unwinder uses an LLVM context. Neither choice qualifies by inspection alone.
Measure exact `_Unwind_*` and Rust/native personality bindings in search and cleanup
phases, including resumes and exception deletion. The first native marker probe
shows exported executable definitions overriding a DSO; archive hiding avoids that
marker interposition. It is **not** actual unwind-context compatibility evidence.

The permanent U10/U11 fixtures must round-trip a panic to a catch in its originating
Rust runtime: executable catch -> DSO cleanup bridge (C-unwind) -> executable panic,
and the reverse DSO-origin path. Verify exact drop order, payload and continued use.
Do not require catching a foreign-runtime panic or unwind across ordinary C ABI.
Preserve baseline-supported valid cross-DSO behavior; a concrete failure is a design
blocker to surface with bindings, not permission to silently drop plugin support.
Multiple DSOs/load orders and real PyO3/maturin regressions are separate gates.

Initial host-SDK evidence (LLD selected despite a GNU capture label) now
demonstrates this blocker: GCC-runtime U07/U10/U11 all pass;
hidden static LLVM passes U07 but aborts U10 and segfaults U11. Rebuilding a
nonhidden archive and broadly exporting the host passes those three cases with
versioned DSO/GCC getter calls interposed to host LLVM. Broad Rust/application
symbol export is not an acceptable final policy. The corrected narrow-export experiment now passes U07/U10/U11 on BFD and LLD,
but native C EH controls still fail; no complete binding/helper design is qualified. The DSO artifact can remain unchanged while its process bindings differ. The contract preserves valid
observable unwind behavior, **not** an invariant that loaded DSOs always bind GCC.
See the [verified proof and exact variants](evals/2026-11-02-unwind-boundaries.md#initial-native-cross-dso-proof--later-2026-11-02).

Bound the runtime promise: ordinary tested standalone Rust paths should have no
direct libgcc_s or dynamic unwind runtime, but a deliberately loaded existing DSO
or native libc backtrace/pthread control may legitimately load GCC in candidate
and Zig. Observe direct/transitive/short-lived lazy loads and explain the actual
workload rather than promise that every possible process API is libgcc-free.
No general dependency-closure product feature is added.

## Configuration, content identity and relinking

Export a private runtime policy/content identity alongside existing clock/provider
fields, with a stable policy salt and complete validated archive hashes. Helpers
must verify regenerated configuration against the actual selection. A same-target-
directory upgrade must relink old executable outputs without manual cleaning,
then remain fresh for unchanged inputs; archive content changes at the same path
must invalidate it. Output kind selects the link-only branch; do not make one
workspace-wide mode force shared units to the executable policy. Preserve Cargo's
own output-kind identity and include the conditional runtime policy in target
fingerprints without changing shared intermediate code generation.

Source/container, explicit descriptor and full sysroot routes require separately
validated target-runtime preparation/discovery; minimal providers are not full
sysroots. The [canonical release checklist](UNWINDER-RELEASE-CHECKLIST.md) defines
exact data/source override qualification and every-bundle/update gates. The final
runtime-dependent driver and exported field/API spelling are pending producer
integration; baseline-only checks cannot qualify a changed dataset.

## Qualification and release sequence

Use the [U01–U22 checklist](UNWINDER-RELEASE-CHECKLIST.md) with exact refs, saved
failure output, maps/import/version joins, archive-member attribution and bounded
loaded-library tracing. Fresh LLVM libc helper imports require supported-floor
proof and explanation, not the old clock-only invariant for all non-clock imports.
The parent will run old0.3/new0.4/Zig at unchanged Aero
`9c248300f1e2ab20ce34dc4b227eb322e33e4c93`: all 182 behavior cases, original strict
comparator unchanged, separate explanatory unwind assessment. Do not dispatch
before the final runtime/source pair exists.

Run focused dispatch/probe checks early; cap workers. Once the runtime is integrated,
run one full applicable native/portable/package/unpacked locked-install suite on
frozen exact source, independent review and final real-registry adoption. Existing
release policy then governs coordinator publication, official archive, published
install, immutable tag, docs.rs, changelog and reviewed GitHub notes. No merge,
version bump, tag or publication is authorized in this implementation stage.
No raw ABI/source-data change or producer-wide lockstep bump is assumed; a new
producer minor is needed only for the new public helper API actually used.

Tier 3 old-system execution and prebuilt legal clearance remain explicit gaps.
Current development evidence and remaining blockers are in the
[dated probe record](evals/2026-11-02-unwind-boundaries.md); they are not a completed
0.4.0 qualification or universal DSO/FFI compatibility claim.
