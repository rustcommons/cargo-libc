# Clock linking investigation — 2026-11-02

This is prerelease evidence, not a qualification or release announcement.
The proposed 0.3.0 default prefers libc for `clock_gettime`, `clock_getres`,
`clock_settime`, `clock_nanosleep` and `clock_getcpuclockid`. There is no proposed
user option or environment policy switch. No product implementation, version
bump, provider-data change or unwinder change is included in this investigation.

**Architecture decision, 2026-11-02:** production argument reordering is
abandoned. The subsequent coordinating Linux diagnostic reports loss of mixed
ordinary/legacy versions under GNU ld in all four tested call/address forms,
including the strong-libc variant. Making the five librt clock aliases
compatibility-only preserves both versions under GNU ld and LLD. The approved
direction is an explicit derived profile in the existing producer builder/render
and cache API, not a raw ABI-data edit or Cargo-side provider implementation.
The exact producer API/ref and consumer integration are pending; this report does
not claim those new profile gates passed. The disposable ordering script and
workflow mode are retired; their immutable sources and failed-run evidence remain
linked below.

**Overlay boundary refinement, 2026-11-02:** the approved consumer integration
uses a producer-owned read-only copy of the selected RT ELF, changing only the
five GNU version default bits after validating the libc/RT pair. The confirmed
API is `provider::link_profile::prepare(libc, librt, cache_root,
LinkProfile::LibcClocksV1) -> Result<Overlay, String>`, returning `search_dir`
(optional), both raw input SHA-256 values and the profile ID. The producer owns
ELF validation, derivation and atomic content-addressed cache creation. This
applies to known automatic/explicit provider stubs and safely discovered full
sysroot ELFs, without changing user files or descriptor v1. A prefix covers
named RT libraries; controlled matching full-path RT inputs are substituted at
the same position. Ambiguous inputs/scripts must not silently bypass policy.
Consumer scaffolding uses a deliberately failing private adapter pending the
exact producer ref; it is not an executable 0.3.0 candidate. Response/script
handling and actual linker-selected sysroot closure remain bounded-verification
work, not established support from the scaffold tests.
Scaffold checks on 2026-11-02: Rust 1.90 `cargo test --lib clock:: --locked
--offline` passes four tests for prefix/state preservation, in-place RT path
substitution, symlink containment and fail-closed adapter behavior;
`cargo clippy --all-targets --locked --offline -- -D warnings` and formatting
checks pass. These are host/scaffold checks, not producer ELF or native release
qualification. No full release suite has been run for this incomplete adapter.
The subsequent input-classification correction uses the existing bounded generic
program-header ELF inspector with optional `DT_SONAME`, rather than the strict
glibc export-evidence loader. Valid DSOs without SONAME are accepted; text linker
inputs are classified by format, not filename extension. Caller OS strings are
retained, including non-UTF-8 Unix paths through direct and compiler-forwarded
arguments. Six focused clock tests and sixteen ELF-inspector tests pass on Rust
1.90, including sectionless DSO fixtures with/without SONAME and arbitrary-name
RT/script inputs. These fixtures do not replace real native DSO/linker gates.

## Assumptions and decisions

| Question | Evidence and current limit |
| --- | --- |
| Can deferring librt after existing libc select all five defaults? | Fourteen bounded native provider-C comparisons below pass. This does not prove arbitrary argument rewriting safe. |
| Is retaining caller linker state necessary? | The no-as-needed/as-needed transition needs scoped restoration to retain librt. No global as-needed policy is authorized. |
| Are explicit old versions preserved? | **Not in the mixed address-reference case below. This is an unresolved release blocker.** |
| Can unknown archives be crossed safely? | No: the separate Linux investigation demonstrates changed `timer_create` ownership. Group membership alone does not make an archive safe to cross. |
| Does provider C expose libc as a caller argument? | Not necessarily. `provider_link` in `src/toolchain.rs` appends libc when `default_libraries=true`. The native experiment uses the final Clang-produced command, including these defaults. |
| Does this establish full-sysroot, Rust or container normalization? | No. The provider's minimal sysroot is not a full development sysroot; implicit Clang libraries and each supported linking path still need implementation and bounded verification. |
| Is a user-visible opt-out planned? | No. The approved scope removed the proposed option/configuration mode. Unsafe unsupported forms need an explicit reviewed diagnostic, not silent fallback. |
| Is provider cache invalidation needed for argument ordering alone? | No provider-cache format or data change is proposed. An internal Cargo profile fingerprint salt and a same-target-directory upgrade/relink regression are still required. |

The [GNU ld options reference](https://sourceware.org/binutils/docs/ld/Options.html)
documents positional as-needed, input-state push/pop, groups and response files.
The [GCC link options reference](https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html)
documents library-order sensitivity. These explain why token movement requires
state and input-boundary checks; they do not establish a safe implementation.

## Native provider-C experiment

Tested repository ref: `4842118cbbea0c71e47ba21876f400d084e1c53b`.
[Run 41367424558](https://github.com/rustcommons/cargo-libc/actions/runs/41367424558)
**failed** the retained mixed-version assertion after fourteen passing cases.
Runner: Ubuntu 24.04 x86-64, Clang 18.1.3, GNU `ld.bfd`, glibc 2.22 source
provider, unchanged registry dependency graph. The checked-in
[`scripts/prototype-clock-order.py` at the tested ref](https://github.com/rustcommons/cargo-libc/blob/4842118cbbea0c71e47ba21876f400d084e1c53b/scripts/prototype-clock-order.py)
compiles real C objects with `libc-cc`, captures Clang's completed provider link
command through a tracing compiler proxy, and executes that command before and
after a disposable ordering change. It is not the product argument transformer.
No target clock setters are executed; this measures ELF links, not runtime ABI
qualification or actual SLES execution.

| Cases | Observation |
| --- | --- |
| all-five; all-five-no-as-needed | All five ordinary clocks bind libc at GLIBC_2.17. No-as-needed retains librt. |
| as-needed-transition; push-pop-rt | Scoped reinsertion preserves the tested RT retention state. |
| timer-api; mq-api; aio-api | Clock selection changes; the tested genuine RT and other versioned imports remain identical. |
| already-libc-first | Existing libc-first selection remains libc. |
| group; xlinker; wl; response; fullpath | The concrete captured final commands select libc. These are examples, not proof of a complete response-file/state/path parser. |
| implicit-provider-c | Works in the experimental final linker command with C's appended libc; source-level input-only normalization would miss this anchor. |
| legacy-and-default | **Fails:** the explicit GLIBC_2.2.5 address reference is silently retargeted to GLIBC_2.17. |

Every case has `input.c`, `input.o`, before/after ELFs, full readelf
symbols/relocations/version-needed/dynamic dumps, captured linker commands and
`comparison.json`. These are saved before assertions so a failing case remains
auditable. The native run's `clock-prototype-41367424558` artifact has ID
`13107916806`, size 220781 bytes and GitHub digest
`sha256:78530ce7e036fb453e468e57a3ae8e352697bae778ec5f8fd3454dfe85f3c527`.
The downloaded evidence is `/tmp/cargo-libc-clock-prototype-fourth` in the
investigation workspace; the workflow artifact is the shared evidence source.

The mixed case's input has separate `R_X86_64_64` relocations for
`clock_gettime` and `clock_gettime@GLIBC_2.2.5`. Before reordering, both resolve
to librt's GLIBC_2.2.5. Afterwards, both data relocations resolve to libc's
GLIBC_2.17; no old version survives in the complete dynsym or relocation dump.
Librt remains DT_NEEDED, which does **not** prove that the explicit binding was
preserved. This is a real semantic retarget, not an undefined-symbol parser gap.

| Saved mixed-case file | SHA-256 |
| --- | --- |
| `input.o` | `a869a3a73199ee5accaa64ba1e3c5484d8fd93e9f23c1d8456f987a95676dee3` |
| `before.elf` | `649e7018492844fe5ede1f866baa9f3608ed8a7a00fa8a3c7ea0abd3da8ebdbd` |
| `after.elf` | `4bdc641a81bad4f052948821d8b8adf597744877945e012e8479dab5c42d2065` |

Earlier runs distinguish harness failures from the semantic blocker:
[41366665691](https://github.com/rustcommons/cargo-libc/actions/runs/41366665691)
used a query path that omitted provider startup/search/runtime inputs;
[41366920621](https://github.com/rustcommons/cargo-libc/actions/runs/41366920621)
did not recognize the prefixed linker executable;
[41367146524](https://github.com/rustcommons/cargo-libc/actions/runs/41367146524)
reached the mixed-case failure but lacked the complete failed-case evidence
added for run 41367424558. No assertion was removed to obtain success.

## Separate source and cross-link investigation

The coordinating Linux investigation supplied
`/tmp/clock-linker-prototype-findings.md`, SHA-256
`fa49a564814b9713302feb0853e80e72624e722829afd76f145d23354933a02c`,
against producer ref `cdddd6499b07a134cbfc730b059fff00ccb9fb7f`.
It reports 746 bounded x86 cross-link cases (742 links and four expected static
failures), GNU cross-binutils 2.40 and LLD 23.1.1 on an aarch64 host. This is
separate supplied evidence, not a native Cargo or runtime run performed here.
It covers all 28 glibc profiles and includes intentional unsafe-order examples;
the count must not be read as 746 successful policy transformations.

That report finds default public rt overlaps only in 2.17–2.29, exactly the five
clocks and only with libc. It also shows why arbitrary archive crossings and
unsaved state changes are unsafe. Its recommendation predates the native mixed
address-reference counterexample above and does not resolve that blocker.
Recorded libc weak versus librt global binding, strong generated stubs, GNU ld
versus LLD, and call versus address reference behavior remain under investigation.
No upstream ABI table or generated export policy has been changed to mask this.

## Exact consumer comparison still pending

The approved three-way job draft is
`/tmp/consumer-clock-job-validated.yml`, SHA-256
`9d52b23874bc771cd28e8264a5ed8846d26796b199befc05d6ce67d192a1a755`.
It is reserved for an isolated `consumer-clock` workflow mode at an exact 0.3.0
candidate, which does not yet exist. No baseline-only run has been dispatched.
The draft requires candidate version 0.3.0 and compares published 0.2.4,
candidate and Zig 0.17.0/cargo-zigbuild 0.23.4 at the same public consumer ref
`9c248300f1e2ab20ce34dc4b227eb322e33e4c93`.

The original consumer comparator remains unchanged. Its strict glibc-only
dependency requirement is not met merely by changing clocks: libgcc_s remains
outside this work's scope. A separate assessment checks 182 behavior comparisons,
non-clock import/version-needed identity and new clock ownership by libc; it
does not claim strict Zig dependency equivalence or actual SLES qualification.
The draft always uploads captured evidence, including failed runs.

Local integration checks on 2026-11-02 extracted all five Bash blocks (`bash -n`)
and compiled the embedded Python without executing consumer builds. Actionlint
found one workflow-context error in the supplied draft: `runner.temp` is not
available in job-level `env`, as specified by the
[GitHub context availability table](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability).
A temporary integration copy changes only the Cargo home expression to
`${{ github.workspace }}/clock-cargo-home`, a fresh path separate from both
checkouts. This copy is `/tmp/consumer-clock-job-integration.yml`, SHA-256
`bad3ce32b59796add1442665b8176e7ee1fe69bcc46b3aa685b6baba20fed6bd`.
The existing workflow plus this isolated mode passes actionlint as a temporary
preview; it has not been installed in the repository or dispatched. The supplied
approved file is unchanged. This syntax check does not establish consumer results.

## Remaining gates

Producer review update: `1b3b51c8afad88e05b0dc238c15f5ee879f94184` is blocked
on partial PT_LOAD overlap validation and an empty-destination rename race. Its
initial local adapter compilation/tests do not count as final release gates.
The unchanged API awaits a corrected reviewed producer ref. The standalone driver
now takes an explicit fifth expected-SHA argument, checks the producer checkout
and refuses that known-blocked ref. The hosted source entrypoint refuses it too.
No native rerun or registry/checksum fabrication accompanies this script change.

Patch preflight: the exact extracted Python blocks from Cargo `9fb9d38` and the
revised driver both successfully patched real copied production manifests and
parsed with `tomllib`, including producer paths with spaces, quotes and a
backslash. The earlier block's Python newline escapes already produced actual
newlines; no literal-backslash TOML failure was reproduced. The revised block
uses an explicit multiline literal and self-validates the root-only patch and
unchanged dependencies before writing. `bash -n`, actionlint and diff checks
cover the driver/entrypoint edits; no unrelated Rust/native suite was rerun.

Do not introduce a production ordering mechanism. Integrate the producer's
derived profile, verify mixed explicit versions and define explicit-provider and
full-sysroot boundaries before implementing and reviewing the supported default.
Automatic source/container preparation must request the profile with its cache
identity; no silent fallback to an old provider is permitted. The proposed
explicit-input boundary is to diagnose affected unprofiled providers/sysroots,
rather than mutate caller files or silently retain old defaults; that boundary
still awaits a coordinating decision. Required later evidence includes
all five clocks, legacy-only and mixed-version forms, actual RT APIs, unchanged
unwinder/non-clock binding, host isolation, both plain-Cargo configuration formats,
same-target-directory upgrade relinking, affected provider/C/container paths,
the exact consumer comparison and the release policy's final package/install
checks. Tier 3 remains deferred; prebuilt-provider legal gates are unchanged.
