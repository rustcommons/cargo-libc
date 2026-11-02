# Proposed 0.4.0 executable unwinder investigation

Status: **proposal for review, not an approved runtime design or release**
(2026-11-02). No implementation, flag, default, archive distribution, producer
version or release date is committed. Current 0.3.0 behavior remains authoritative.
This document records the requested investigation, not new accepted requirements.

## Goal and unchanged boundaries

Investigate removing the `libgcc_s.so.1` runtime dependency from **standalone
executables** while retaining `panic=unwind` and dynamic glibc. Do not substitute
`panic=abort`, `crt-static`/static glibc, or a relaxed consumer comparator to make the result
pass. Preserve the five-clock libc default, explicit legacy clock versions,
actual RT APIs, target/host separation, caller as-needed policy, groups and linker
state. No general library-argument reorder is proposed.

The [0.3.0 consumer record](RELEASE-0.3.0.md#exact-consumer-comparison) establishes
the motivation: 182 behavior cases pass, but the unchanged strict Zig comparator
still fails with `libgcc_s.so.1` and pre-existing non-clock differences. Removing
one dependency would not by itself establish strict equivalence or glibc-only
transitive dependency closure.

The [current provider link implementation](../src/toolchain.rs) passes rustc's
arguments through and also supplies GCC runtime libraries for target C links.
`-static-libgcc` alone does not override an explicit Rust `-lgcc_s`; with
`-nodefaultlibs`/`-nostdlib`, driver defaults are not a substitute for auditing the
actual final link. Capture completed Clang/GNU/LLD commands before choosing a
mechanism. See [GCC link options](https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html)
and [Rust codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html).

## Existing planning probes, not qualification

The supplied [native planning probes](evals/2026-11-02-unwinder-plan-probes.json)
(2026-11-02) show installed Rust 1.90 rejecting
`-C link-self-contained=+unwind` without unstable options. Do not promise a stable
one-flag solution, silently raise the MSRV, or require nightly to evade this result.
The [rustc option reference](https://doc.rust-lang.org/rustc/codegen-options/index.html#link-self-contained)
is context; the exact 1.90 probe establishes the observed supported-toolchain limit.

GCC 13's `-print-file-name` resolves `libgcc.a` and `libgcc_eh.a` in its x86-64
runtime directory. That establishes location only: no static-unwind member,
glibc-floor, link, panic, shared-library or runtime qualification was performed.
The official GCC link-options reference also states that `-static-libgcc` is
ignored with `-nodefaultlibs` and recommends shared libgcc for exceptions crossing
DSO boundaries. These facts motivate separate executable/extension designs, not
a claim that either archive is ready to ship.

## Candidates to compare before a design decision

| Candidate | Required investigation | Decision still open |
| --- | --- | --- |
| Selected GCC static unwind/runtime archives, including `libgcc_eh.a`/`libgcc.a` as appropriate | Discover through the selected target GCC/Clang toolchain, validate actual members/architecture, exception support, archive dependencies, build provenance/license and PIC constraints. Measure their real GLIBC imports; the build host's GCC age does not prove target-floor safety. | Which archive/member combination preserves Rust unwind semantics and avoids runtime loads of libgcc_s? Do not blindly replace every runtime library or use an arbitrary host archive. |
| Explicitly validated LLVM libunwind alternative | Pin provenance/configuration and demonstrate matching target ABI, exception handling, threading, backtrace and glibc-floor behavior; identify any accompanying compiler-runtime requirement. Validate actual archive PIC/build inputs and license/distribution constraints. | Whether it can meet the same supported targets without new unsupported runtime dependencies or broadening scope. LLVM is a candidate, not an approved replacement. |

Compare both on the same inputs before selecting either. An unavailable or
unvalidated archive must produce a clear decision/refusal, not silent host fallback.
Provider source/container, explicit-provider and full-sysroot routes need separately
defined discovery/containment rules: a minimal generated provider is not a full
runtime SDK. Keep user inputs read-only and retain reproducible provenance.
No downloaded/bundled unwinder or new dependency is authorized by this plan.

## Shared libraries and user interface remain undecided

`cdylib`, Python/PyO3 extensions and plugins can share a process and unwind/runtime
state with other DSOs. They need separate treatment, including symbol visibility,
interposition, duplicate unwinder instances, PIC/text relocations, exception-frame
registration and actual supported language/FFI boundaries. A standalone-executable
result is **not** a universal static-unwinder default for extensions. Preserve
existing shared-library behavior until a separately reviewed design and tests
justify a change; do not allow unwinding through an ABI that forbids it.

A user product decision is pending: static unwind opt-in versus default for
standalone executables. Both remain under consideration; neither is selected. No flag or environment/TOML key is specified here.
After prototype evidence, obtain a product decision on user-facing semantics,
unsupported-input handling and migration before implementing a release policy.

## Proposed evidence and acceptance gates

These are work to perform, not passing results:

1. **Baseline and discovery:** record exact 0.3.0/candidate refs, tool versions,
   target/provider/sysroot, compiler traces, archive hashes and member provenance.
   Compare GCC and LLVM candidates on 10–20 realistic executable/shared-library
   cases before choosing; retain failed cases and explain any excluded route.
2. **Link and ABI:** use GNU ld and LLD with Rust 1.90 (plus the supported current
   toolchain). Inspect all dynsym imports, relocations, version-needed owners,
   DT_NEEDED, text relocations, search paths and the tested applications’ actual
   bounded load closure. Prove no
   libgcc_s dependency or delayed `dlopen` reintroduction for the tested executable
   workloads, no unwanted GLIBC-floor increase and no unexplained non-unwind
   binding changes. Verify both existing 2.17 and consumer 2.22 ceilings; do not
   confuse a low measured import tag with qualification for an older glibc.
3. **Unwind behavior:** test real panic/catch_unwind, destructor cleanup during
   unwind, nested catches, thread spawn/join/panic and useful backtraces. Separate
   capture/display and runtime loading observations; library absence in DT_NEEDED
   alone is insufficient. Do not run clock-setting fixtures.
4. **Integration and migration:** exercise Rust build/run/test/install, direct C
   and CMake, provider/sysroot discovery, native host tools, library groups and
   push/pop/static/dynamic/as-needed state. Preserve clock owners, mixed legacy
   references and genuine RT APIs. A policy/archive/content change must enter
   target fingerprints and generated shell/TOML identity; demonstrate old/new
   relinking in the same target directory without manual cleaning. Do not freeze
   a config/cache format before the design decision.
5. **Shared-library regressions:** build/load existing `cdylib` and PyO3/maturin
   fixtures, including multiple DSOs and host dependencies where meaningful;
   verify their existing unwind/runtime contracts remain intact. Document PIC,
   interposition and cross-boundary limits, rather than infer them from an
   executable passing catch_unwind.
6. **Real consumer:** rerun the pinned Aero1553 old/candidate/Zig comparison with
   182 behavior cases and fresh ELF/import/closure evidence. Keep its original
   strict comparator unchanged and report any remaining differences. Do not
   reuse the 0.3.0 clock-only comparison as unwinder qualification.
7. **Release gates after approval:** one exact candidate's applicable portable,
   MSRV, native, package and clean unpacked locked-install checks; independent
   review and normal merge; coordinator publication, official archive/docs.rs,
   published installation, immutable tag and reviewed GitHub notes. Use the
   [release policy](RELEASING.md); no crate release occurs in this documentation PR.

Tier 3 actual old-system execution remains deferred and must be stated as a gap;
modern-kernel containers do not supply it. Prebuilt-provider legal gates remain
unchanged. If prototype results contradict the goal or require unacceptable
shared-library compromises, return the decision rather than ship a workaround.

## Version and scope decisions

A prospective cargo-libc **0.4.0** is appropriate for an approved runtime/linking
contract change under the existing minor-version policy; no version bump is made
now. A producer minor release is needed only if the chosen design actually needs
a new public producer API. No raw ABI/source-data release, producer-wide lockstep
bump or unrelated library update is assumed. Acceptance checks observe the
resulting tested app closure; a general dependency-closure product feature is
outside this proposal.

Keep optional unrelated fixes out of this release unless their necessity and
compatibility are independently justified. Track any approved follow-up in the
[roadmap](ROADMAP.md), keep user-visible work in [Unreleased](../CHANGELOG.md), and
record proposed-versus-implemented status in the [trace index](REQUIREMENTS.md).
