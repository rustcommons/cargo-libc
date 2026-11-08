# cargo-libc requirements and documentation index

## Scope and sources

This index retains historical release evidence and records published 0.5.0
static target-libgcc inspection with unchanged diagnostics/clock/unwinder behavior; it is not a backlog or universal compatibility claim. Current
released source/tag: `1761260a52e192e55f774602ca05add64bd88994` / `v0.5.0`.
Published 0.2.4 runtime/API behavior is unchanged from 0.2.3/0.2.2. Documentation audit: 2026-11-01, baseline main
`1ca27da9086ebf6cf19a6f2c0e858108b38cac46`; the immutable 0.2.4 source/tag:
`889515b191ead51303b1321de858ec4e2209722c` / `v0.2.4`. The original 0.2.2 source
`b87563c6be6faf89ac26055f88fb520991125b8b` and evidence remain unchanged.

Authority and sources are [AGENTS.md](../AGENTS.md), the maintained
[user guide](USER-GUIDE.md), [provider contract](PROVIDER-CONTRACT.md),
[release policy](RELEASING.md), and explicit decisions in the
[roadmap](ROADMAP.md). Source/manifests and workflow inspection determine what is
implemented; dated test evidence supports only its named scope.
[FEATURE-REVIEW.md](FEATURE-REVIEW.md) retains its existing F01–F36/E01–E16 IDs
and proposal dispositions. A proposed acceptance test is not a requirement or a
passed test. No proposal is accepted merely by appearing here.

Evidence shorthand used below:

- **R22:** [registry-only run 41268879926](https://github.com/rustcommons/cargo-libc/actions/runs/41268879926),
  2026-11-01, candidate `734f9bc4205e4a5c119feaa3dd35c49a4761e1c3`, Ubuntu 24.04
  x86_64 / Rust 1.90.0 / Clang 18; exact logs, checksums and exclusions in the
  [0.2.2 release record](RELEASE-0.2.2.md#registry-only-release-validation).
- **I22:** [published-install run 41269812853](https://github.com/rustcommons/cargo-libc/actions/runs/41269812853),
  2026-11-01, released 0.2.2 commit `b87563c6be6faf89ac26055f88fb520991125b8b`;
  clean Cargo home, registry locked installation,
  installed executables and bounded native/container checks. See
  [publication evidence](RELEASE-0.2.2.md#publication-and-installed-verification).
- **R23/I23:** [0.2.3 record](RELEASE-0.2.3.md), 2026-11-01. Registry run
  [41274081431](https://github.com/rustcommons/cargo-libc/actions/runs/41274081431)
  passed at `b40b2de58e842d1d274f7882b0396ddb9612eead`, and clean published-install
  [41274796646](https://github.com/rustcommons/cargo-libc/actions/runs/41274796646)
  passed at released 0.2.3 commit `289f28d161027e86decc808fc1745ec4a5b74d39`.
  Both are bounded Ubuntu x86_64 / Rust 1.90 /
  Clang 18 evidence; dependency-doc and actual docs.rs link/banner rendering are
  verified separately in the record.
- **R24/I24:** [0.2.4 record](RELEASE-0.2.4.md), 2026-11-01: registry run
  [41275479293](https://github.com/rustcommons/cargo-libc/actions/runs/41275479293)
  passed at `896ac7b`; clean published-install
  [41276430527](https://github.com/rustcommons/cargo-libc/actions/runs/41276430527)
  passed at released `889515b`. README neutral navigation and all six images were
  checked in the actual docs.rs page; dependency/code payload is unchanged.
- **Audit:** explicit source/document/workflow inspection on 2026-11-01 at the
  baseline above. This does not imply an execution test or new native qualification.

- **N30:** [0.3.0 native composite](RELEASE-0.3.0.md#qualification-and-exact-refs),
  2026-11-02, Cargo `50c9f7129abcfdb936b876fd0048c83e71fe30d1` and producer
  `e857e43f113fd5fb02d0f2afff58b90fcfbd1ee6`: Rust 1.90 core, sixteen GNU/LLD
  clock comparisons, 68 independent named-input cases, host sysroot, same-target-directory
  upgrade and shell/TOML identity passed. Remaining command smokes resumed under
  installed 1.99; C/CMake and host-SDK smokes used 1.90. The original nested-Clippy
  stop is retained; this is composite evidence, not one successful driver exit.
- **R30:** [registry run 41378810513](https://github.com/rustcommons/cargo-libc/actions/runs/41378810513),
  2026-11-02, `fa2826832d14f1f0586f116d45d6133731bc6d7d`: Rust 1.90 full portable,
  native/provider/stream/container and clean unpacked locked-install gates passed.
  Its README predates the separately qualified correction.
- **P30:** [focused package/install run 41379965966](https://github.com/rustcommons/cargo-libc/actions/runs/41379965966),
  2026-11-02, `4ad06e5e2402a550eabdae2980a3e3517497a6bf`, package inputs equal
  `84cc60ee4a60517f160b8003e502768575d39af7`: corrected README, Rust 1.90 verified
  package, fresh Cargo home and unpacked locked/offline installation, all five
  helpers and installed C/CMake checks passed. This is not a published install.
- **A30:** [consumer run 41378111334](https://github.com/rustcommons/cargo-libc/actions/runs/41378111334),
  2026-11-02, `29582bc7af35cba1f741f989b47bc520979e13ae`: old/candidate/Zig
  comparison, 182 behavior cases each, libc clock ownership and unchanged non-clock
  bindings passed. The unchanged strict comparator still fails: libgcc_s and
  pre-existing non-clock differences remain. See the [bounded record](RELEASE-0.3.0.md#exact-consumer-comparison).

- **I30:** [published-install run 41382187049](https://github.com/rustcommons/cargo-libc/actions/runs/41382187049),
  2026-11-02, exact release `a70ce9ffac6be8611b66a2829fefb3093cb41ce6`, Ubuntu
  x86-64 / Rust 1.90 / Clang 18: fresh Cargo home, registry locked installation,
  all five helpers, source-provider C/CMake/host separation/C++ refusal, provider
  stdout and native/Docker/Podman stream/status checks passed. Official archive,
  clean VCS, non-yanked/checksum, immutable tag and actual docs.rs navigation/render
  are verified in the [publication record](RELEASE-0.3.0.md#publication-and-installed-verification).


- **Q40:** [0.4.0 candidate qualification](RELEASE-0.4.0.md), 2026-11-03,
  final native package/harness `d335f8ec52a5b1e6331be57acecab01fe7f226c7`, clean
  unpacked installation from `62347644fe1e6808b19de043c702da69077409e8` with only
  package VCS metadata differing. Native x86 WSL/glibc 2.39, Rust 1.90, Clang 18:
  official-baseline/registry/payload audit, all five helpers, C/CMake/host boundary,
  thirteen capability cases and installed streams passed. Recording container
  route only; no new musl/static-host/oldOS/LLVM qualification. This Q40 evidence is prepublication, not the later registry installation.

- **I40:** [0.4.0 publication and installed verification](RELEASE-0.4.0.md#publication-and-installed-verification),
  2026-11-03, released `aa2ea56e5d7943aeb4e0023713a089693d2103c1` / `v0.4.0`.
  Official API/index/archive and clean source verified; separate fresh Cargo home
  registry locked installation, all five helpers, C/CMake/host boundary, thirteen
  capability cases and installed streams passed on native x86 WSL/glibc 2.39,
  Rust 1.90. Proof bundle SHA256 `ba9c39009ca306758ad931607ccc0fd47bb9543190f625558808a26c597adec0`;
  all 44 member hashes checked. Recording engine only; no hosted installed run or
  real musl/static/oldOS/LLVM qualification is claimed.

- **Q50:** [0.5.0 candidate qualification](RELEASE-0.5.0.md), tested
  `c5f41485af05bd311c15f567a6bd9310a3154237`, source equal reviewed `972783d`.
  Actual registry graph, Rust 1.90 102 library/nine process/zero doctests,
  pinned 1.99 fmt/Clippy, verified 28-file package, clean unpacked locked all-five
  installation, C/CMake/host boundary/thirteen capability cases, clean-env resumed
  streams/installed 36 GNU checker cases and real 0.4 legacy comparison passed.
  Original driver RUSTDOCFLAGS refusal/exit 1 is retained: composite, not a single
  all-green run. Portable Linux/Windows CI41684307122 and native Windows MSVC
  match 0/missing 2/versionless 3/full SHA controls are separate evidence. Packet
  SHA256 `461de6aff75d0009e11cd5d038346db123012aa70acc9067d485eb9c25894322`,
  all 228 member hashes verified. No LLD/real container/musl/static/oldOS/LLVM or
  deployment-safety qualification. This historical candidate evidence predates the separate published installation I50.

- **I50:** [0.5.0 publication and installation](RELEASE-0.5.0.md#publication-and-installed-verification),
  released `1761260a52e192e55f774602ca05add64bd88994` / annotated `v0.5.0`.
  Official/index/API checksum and exact payload/VCS, docs.rs API/actual README/
  banner/navigation verified. Separate fresh Cargo-home native x86 WSL GNU
  Rust 1.90 registry locked installation, all five helpers, checker 36 plus actual
  0.4 legacy comparison, C/CMake/host separation/local simulation/13 capability cases/streams
  passed, no failure/resume. Proof SHA256
  `f6d3ef73bc03bc0b5998af3519f5367792045558b213e474807a41ab04977472`,
  all 177 member hashes checked. Recording engine only, no hosted published-install
  run or added real-container/musl/static/oldOS/LLVM/runtime-safety qualification.


## Requirements

| ID | Requirement and source | Implementation / documentation | Verification evidence | Status and gaps |
| --- | --- | --- | --- | --- |
| CL-REQ-001 | Limit managed application builds to x86_64 Linux, upstream glibc 2.17–2.44, dynamically linked executables and `cdylib`. [Guide: limits](USER-GUIDE.md#limits); AGENTS scope; F27–F31 remain proposals. | `Toolchain::new/from_provider` in [toolchain.rs](../src/toolchain.rs); release list in [provider.rs](../src/provider.rs); [architecture](../ARCHITECTURE.md). | Audit of platform guards/release list; R22 source providers 2.17/2.28. | Implemented—partial verification; the 0.2.2 native release run did not rerun every glibc/provider/Python combination. Other architectures and target libcs, static glibc and provider PIE are outside scope; non-glibc build hosts are unqualified, not rejected by distro name. |
| CL-REQ-002 | Build setup requires Rust >=1.90 and suitable native tools; install all five sibling executables together. Deployment normally needs compatible glibc and matching-architecture libgcc_s.so.1, not the compiler; check required symbol/version/owner pairs against the actual selected runtime. [Guide: install](USER-GUIDE.md#install), [deployment prerequisites](USER-GUIDE.md#deployment-runtime-prerequisites). | [Cargo.toml](../Cargo.toml), `sibling` and Clang probes in toolchain.rs; [README](../README.md#installation). Generated-provider GCC stubs are link-only; runtime/package/SONAME distinctions and read-only checks are documented in the guide. | R22 MSRV/all-targets check and package installation; I22 `cargo install cargo-libc --version 0.2.2 --locked`, five-helper/native smoke. | Verified; named build/setup gates only. [2026-11-03 documentation/source/inspection audit](evals/2026-11-03-libgcc-deployment.md) bounds the deployment example; no universal target-runtime presence, export compatibility or package-version mapping is certified. |
| CL-REQ-003 | Version-suffixed targets are coordinator syntax; strip the suffix for Cargo, reject conflicting distro/glibc/kernel selections and unsupported releases. [Guide: choosing a target](USER-GUIDE.md#choosing-a-target); F01. | Parser in [cli.rs](../src/cli.rs); unsuffixed Cargo target in toolchain.rs. | R22 includes `versioned_target_selects_ceiling_and_preserves_cargo_arguments` and `target_selection_rejects_conflicts_and_malformed_requests`. | Verified; parser fixtures and bounded provider selection, not distro certification. |
| CL-REQ-004 | Preserve Cargo OS-string arguments while refusing managed target/config/message-format overrides; permit the supported tool/program second separators. [Cargo reference](CARGO-WORKFLOWS.md); F02. | `validate_cargo_args`, `validate_tool_args` in toolchain.rs; [guide commands](USER-GUIDE.md#commands). | R22 unit tests `cargo_cannot_override_managed_target`, `test_and_run_pass_arguments_after_the_second_separator`; `smoke-commands.sh`. | Verified; arbitrary Cargo flags/compiler combinations are not exhaustively qualified. |
| CL-REQ-005 | Host build scripts/proc macros and their C units stay native; target flags/linker apply only to target units. [AGENTS scope](../AGENTS.md#scope-and-implementation); F12. | Explicit Cargo target and encoded flags in toolchain.rs; `CARGO_CFG_CARGO_LIBC_PROFILE` in [cc.rs](../src/cc.rs); [build control](USER-GUIDE.md#how-the-build-is-controlled). | R22 `smoke-c-deps.sh` and `smoke-linux.sh` verify host/target separation; I22 installed C/CMake fixture. | Verified; same-triple x86_64 fixture, not general cross compilation. |
| CL-REQ-006 | Delegate provider preparation/cache and the derived clock profile to libc-abi-tools; explicit provider and sysroot/GCC runtime are alternate input routes. [AGENTS](../AGENTS.md#scope-and-implementation); [guide providers](USER-GUIDE.md#providers). | `RawConfig::resolve` in cli.rs; provider loading/input probes in toolchain.rs; contained ELF discovery and root profile adapter in clock.rs. | N30 actual selected-provider/host-sysroot and C links; R30 source-provider reuse and container execution; parser `provider_selection_is_explicit_and_excludes_sysroot_options`; CL-REQ-027. | Implemented—partial verification; named selected inputs are covered by N30/R30, but clock integration with container-provider preparation was not separately rerun; no general SDK compatibility claim. |
| CL-REQ-007 | Exact requested provider selection; never silently fall back after refusal. [AGENTS](../AGENTS.md#scope-and-implementation); F26. | provider.rs `validate_selection`; cli.rs mode selection delegates to producer. | R22 `selects_each_supported_release_without_fallback` and provider mismatch/too-new-symbol smoke; Audit of mode error propagation. | Verified; source selection/refusal fixtures, not arbitrary third-party provider validation. |
| CL-REQ-008 | Treat v1 descriptors as trusted local inputs; validate six fields, bounded UTF-8, absolute contained paths, required files and exact header version. Do not claim authenticated SDK integrity. [Provider contract](PROVIDER-CONTRACT.md). | provider.rs parsing/file checks, toolchain.rs probes; producer owns full ABI verification. | R22 `rejects_wrong_versions_and_ambiguous_descriptors`; Audit of canonical-path/file/header checks. | Implemented—partial verification; not authentication, relocation or complete content hashing. All external inputs remain trusted. |
| CL-REQ-009 | Detect the clock pair/profile through content identity and an internal policy salt; regenerate saved config after updates. Other arbitrary in-place SDK changes are not fully detected and may require cleaning/changing target directory. Provider version changes can require fresh preparation. [Guide](USER-GUIDE.md#clock-linking); [release cache note](RELEASE-0.2.2.md#provider-cache-refresh). | toolchain.rs `profile_fingerprint`; existing provider preparation hashes; clock.rs delegates actual libc/RT hashing and immutable derived-cache validation to the producer. | N30 old 0.2.4 to candidate relink in the same target directory, helper paths and descriptor; shell/TOML identity roundtrip; R30 provider reuse. Registry producer source equivalence and exact checksums are recorded in RELEASE-0.3.0. | Implemented—partial verification; the pair-specific content identity is not a full SDK hash, authenticated inventory or promise of old-cache import/offline migration. |
| CL-REQ-010 | C via cc/CMake uses target headers/link inputs while host C remains native; do not silently use host package libraries. [Guide: C dependencies](USER-GUIDE.md#c-dependencies). | cc.rs wrappers/toolchain helpers; `examples/c-deps`; F15/F18 historical proposals do not add broader adapters. | R22 `smoke-c-deps.sh` for 2.17/2.28; I22 installed C/CMake build/run/inspect. | Verified; pkg-config/bindgen/system-library integration and arbitrary build scripts are outside these fixtures. |
| CL-REQ-011 | Refuse target C++ and static linking until corresponding target runtimes exist; allow native host C++. [AGENTS](../AGENTS.md#scope-and-implementation); [guide limits](USER-GUIDE.md#c-dependencies). | cc.rs language/link checks; `libc-cxx`. | R22 C++ refusal and unit argument checks; I22 installed target C++ refusal. | Verified; evidence is refusal, not target C++ support. |
| CL-REQ-012 | `build` inspects the named Cargo-reported executable, including fresh artifacts; `install` checks every built executable after installation. [Guide build/install](USER-GUIDE.md#commands). | Cargo JSON reader [cargo_output.rs](../src/cargo_output.rs); toolchain.rs build/install/verify paths. | R22 portable Cargo JSON tests, provider reuse and command smoke. | Verified; build does not inspect every workspace output; failed install checks occur after installation and need user cleanup. |
| CL-REQ-013 | Inspect direct ELF/version requirements only, retaining older ABI tags and distinguishing weak/non-glibc requirements; no dependency-closure/runtime/CPU guarantee or GCC-floor/export validation. [Guide: inspect](USER-GUIDE.md#inspect), [deployment checks](USER-GUIDE.md#check-the-actual-executable-and-target-runtime); AGENTS older symbol rule. | [elf.rs](../src/elf.rs), [report.rs](../src/report.rs); program-header metadata path. | R22 ELF parser tests and `newer_weak_requirement_does_not_raise_required_floor`, `does_not_treat_libstdcxx_versions_as_glibc_versions`; native inspect. [2026-11-03 current CLI/source check](evals/2026-11-03-libgcc-deployment.md) confirms GCC metadata is listed while GLIBC alone is ceiling-checked; strict/verbose do not enable GCC validation. | Verified; direct-file scope only; transitive libraries/plugins and runtime compatibility remain gaps. |
| CL-REQ-014 | Bound malformed ELF/scanner processing without rejecting valid large fixed tables solely for exceeding 65,536 entries. [Release correction](RUN-STDOUT-FIX.md#combined-candidate-qualification). | report.rs file bound, elf.rs metadata bounds; registry linux-abi-scan >=0.3.3. | R22 `check-malformed-scan.py`: overflow/reused-chain rejection, aliased headers accepted, Cargo 79,207 and librustc 146,741 records accepted with named time/memory limits. | Verified; specific corpus/resource bounds, not every ELF or universal parser safety. |
| CL-REQ-015 | Tier 1 scan findings are scoped static evidence; strict mode makes warnings fail. [AGENTS tiers](../AGENTS.md#scope-and-implementation); [guide tiers](USER-GUIDE.md#testing-tiers). | report.rs `floors/scan/inspect_file`, linux-abi-scan; [architecture](../ARCHITECTURE.md#artifact-inspection-and-cargo-workflows). | R22 scan entry paths and process `inspection_results_survive_closed_stdout`, scan rendering/floor unit tests. | Verified; runtime-computed syscalls, library internals and fallback execution are not proved. |
| CL-REQ-016 | `test/run` use the runner, enforce glibc ceiling/known host-version checks and simulate a kernel floor only when selected. [Guide: test, run](USER-GUIDE.md#test-run). | [simulate.rs](../src/simulate.rs) host checks/runner plan; kernel-abi-tools. | R22 host comparison/plan tests and `check-run-streams.py` local kernel case; I22 installed runner cases. | Verified; simulation uses a modern host kernel, and exercised code paths only. |
| CL-REQ-017 | Container run/test preserve intended CARGO/RUST environment, stdin, stdout and numeric child status; use trusted images because forwarded variables may contain credentials. [Guide providers](USER-GUIDE.md#providers); [release](RELEASE-0.2.2.md). | kernel-abi-tools 0.3.3 container runner; simulate.rs selection. | R22 and I22 Docker/Podman run/test: spaced variables, unrelated-variable exclusion, redirected input, JSON stdout, exit 7. | Verified; named engines/image/profile only; forwarding is not a secret sandbox. |
| CL-REQ-018 | Owned Cargo setup/dry-run diagnostics use stderr; explicit reports/config use stdout. Ignore only BrokenPipe at write/flush boundary and continue operations. [Guide tiers/reference](USER-GUIDE.md#testing-tiers). | [output.rs](../src/output.rs), toolchain.rs diagnostic calls; shared provider output delegated. | R22 seven process regressions, provider later-error regression and native `check-run-streams.py`; I22 installed stream checks. | Verified; child output/errors remain the child's own handling. |
| CL-REQ-019 | Preserve deliberate error/refusal and numeric child statuses when final stderr reporting is unwritable. [Guide: reference](USER-GUIDE.md#reference). | five executable entrypoints, output.rs; toolchain.rs/simulate.rs status paths. | R22 process tests `all_five_final_errors_preserve_status_with_unwritable_stderr`, `runner_refusal_keeps_status_two_with_unwritable_stderr`, `compiler_child_status_survives_unwritable_stderr`; native exit 7. | Verified; does not guarantee shell-style signal-number propagation; signal-status policy is follow-up review. |
| CL-REQ-020 | Both plain-Cargo formats use unsuffixed target, generated machine-specific paths, separate inspection and target-only flags, including the same private clock identity; shell format refuses whitespace paths. [Guide: plain Cargo](USER-GUIDE.md#plain-cargo-and-ci). | toolchain.rs `plain_cargo`; `--format sh/cargo`; [command reference](CARGO-WORKFLOWS.md); clock.rs exported original/overlay paths and content identity. | N30 shell/TOML private-field roundtrip and regenerated configuration; R30 both formats, build/test/inspection and too-new-symbol refusal; source inspection of shell whitespace checks. | Verified; N30/R30 cover generated formats and the new clock fields. Arbitrary external configuration merging is outside that evidence. |
| CL-REQ-021 | PyO3/maturin `cdylib` workflow requires separate inspection, Python ABI/tag/dependency checks and actual target tests. [Guide: Python extensions](USER-GUIDE.md#python-extensions-maturin). | toolchain.rs shared-library path; [smoke-python-ext.sh](../scripts/smoke-python-ext.sh); F11 retains separate library-type scope. | Audit of shared-link path/script and CI Python job; historical guide workflow. R22/I22 explicitly excluded Python matrix. | Implemented—partial verification; no fresh 0.2.2 Python matrix or universal manylinux claim. |
| CL-REQ-022 | Keep implementation dependencies std plus first-party registry crates, preserve input attribution/licenses. [AGENTS](../AGENTS.md#scope-and-implementation); [NOTICE](../NOTICE.md). | [Cargo.toml](../Cargo.toml), [Cargo.lock](../Cargo.lock), NOTICE; embedded producer inputs under their licenses. | Audit of manifest/lock/include list and notices; R22 registry resolution; published archive verification in release record. | Verified; technical provenance is not legal clearance for prebuilt distribution. |
| CL-REQ-023 | Independently version cargo-libc; authorized releases require applicable gates, registry-only resolution, clean package install, immutable identity and consistent changelog/versioned notes/live GitHub Release. [Release policy](RELEASING.md); [shared rule](../AGENTS.md#release-documentation). | [changelog](../CHANGELOG.md), [versioned notes](releases/HISTORY.md), [release record](RELEASE-0.2.2.md), release-doc checker/workflow; [manual workflow](../.github/workflows/candidate-integration.yml); scanner/kernel minima 0.3.3, root libc-abi-tools minimum 0.4.0 and linux-targets minimum 0.3.0. | R22/I22, R23/I23, R24/I24 and R30/P30/I30; exact official archive checksums, clean VCS/tags and docs.rs checks are recorded for each release. | Verified: bounded named registry/source gates plus eight-note documentation completion. Normal merge `9925f61` preserves reviewed inputs; [bootstrap run 41387805648](https://github.com/rustcommons/cargo-libc/actions/runs/41387805648) passed before notes were published, and [full live check 41387908379](https://github.com/rustcommons/cargo-libc/actions/runs/41387908379) passed. At the 2026-11-02 backfill check, all eight bodies matched committed notes, original tags were unchanged, v0.3.0 was latest and zero assets were attached; I40 separately records the later 0.4.0 release. Historical runtime-evidence gaps remain; no product checks were rerun for backfill. |
| CL-REQ-024 | Real old-system tier 3 qualification is deferred, not completed and not a routine-release blocker; new old-system claims require actual environment/workload evidence. [AGENTS validation](../AGENTS.md#validation); [roadmap](ROADMAP.md#deferred-qualification-and-infrastructure). | [Guide tiers](USER-GUIDE.md#testing-tiers), RELEASING policy. | Audit of adopted 2026-11-01 policy and release exclusions; no real old-system run is claimed. | Deferred; unavailable infrastructure/evidence remains an explicit gap. |
| CL-REQ-025 | Keep prebuilt mode unavailable pending legal review. [Roadmap decision](ROADMAP.md#maintainer-decisions); [guide providers](USER-GUIDE.md#providers). | cli.rs mode delegates refusal to libc-abi-tools; [architecture](../ARCHITECTURE.md#provider-and-sysroot-inputs). | Audit of producer 0.3.3 prepare mode refusal and maintained guide; no prebuilt publication/qualification in R22. | Deferred; legal clearance and a separately approved implementation remain required. |
| CL-REQ-026 | Maintain setup-first README, authoritative guides, stable trace IDs, identifiable historical evidence and immutable package snapshots. [AGENTS documentation rules](../AGENTS.md#documentation-and-requirements-traceability). | This index/map and linked maintained guides; published README/API docs remain immutable snapshots. | Audit 2026-11-01: source/command/link/render/diff checks; shared convention page HTTP 200 and Contents API blob `1b659ff536d43bcae5b09626069e89488058ba6f` verified against central merge `c8a0b9c01aae5657dd0ef649bd37b18d81919ce5`. | Verified; bounded documentation audit only, no package release or universal behavior verification implied. |

| CL-REQ-027 | Prefer libc defaults for clock_gettime, clock_getres, clock_settime, clock_nanosleep and clock_getcpuclockid without a public clock setting; retain explicit legacy versions, genuine RT APIs, linker order/state and unwinder behavior. Released 0.3.0 scope in [release record](RELEASE-0.3.0.md); [guide](USER-GUIDE.md#clock-linking). | [clock.rs](../src/clock.rs) delegates the read-only ELF copy/cache to producer `LibcClocksV1`; [clock_named.rs](../src/clock_named.rs) checks completed-Clang named selection/state and bounded standard wrappers; target prefix, controlled in-place RT substitution, generated identity and fingerprint in toolchain.rs. Raw data, user files and descriptor v1 are unchanged. | N30 all-five/mixed-version GNU/LLD ELF ownership, real RT/state/host-sysroot and upgrade cases; R30 registry/native gates; P30 final README/package/install; A30 actual consumer; I30 published installation. Rejected reordering evidence is retained in [evals](evals/2026-11-02-clock-linking.md). | Implemented—partial verification; named N30/R30/P30/A30/I30 gates passed for published 0.3.0. Opaque custom scripts/responses and unproven linker-default fallbacks refuse. Container-provider clock preparation, general dependency closure and tier 3 remain unqualified. |
| CL-REQ-031 | Diagnose capabilities by operation, distinguishing build provenance, observed host evidence, selected glibc inputs and local/container execution; no install-time/distro rejection or silent preparation fallback. [Guide](USER-GUIDE.md#host-capabilities-and-execution-routes); approved capability-diagnostics scope, 2026-11-03. | [capability.rs](../src/capability.rs), doctor and build diagnostics, local PT_INTERP preflight in simulate.rs. Inspect/help/version retain probe-free paths. | Focused mapped-glibc/musl/unknown, missing/present/unknown interpreter and container independence fixtures; stdout/process regressions. Exact `d6b5f78fb31f7f6a5d8c604a313c78039573f9aa`: [CI 41633902033](https://github.com/rustcommons/cargo-libc/actions/runs/41633902033) and [PR CI 41633910012](https://github.com/rustcommons/cargo-libc/actions/runs/41633910012), 15/15 each plus both release-doc checks (32 total); merged `748ffa23274792dd21e1cb4cd812271f1f61e1a3` has the same tree. Q40/I40 and [0.4.0 notes](releases/v0.4.0.md) record status migration, separate candidate/published installation passes, official archive identity and exact payload carry-forward. | Implemented—partial verification; observations are not runtime certification. Real musl/static-host operations, target dependency closure and tier 3 remain unqualified. |

Release-documentation audit, 2026-11-02: official archives for all eight tags
match registry checksums, clean VCS and tagged source; see [history](releases/HISTORY.md).
Local release-doc/syntax/link checks apply only to this documentation candidate.
GitHub Releases were absent at the initial inspection. CL-REQ-023 now records
successful hosted bootstrap and full live-body/latest verification; historical
runtime-evidence gaps are preserved.
The [deferred unwinder plan](UNWINDER-0.4.0-PLAN.md) has no assigned release;
its historical filename does not describe the [0.4.0 diagnostics release](releases/v0.4.0.md).
It is not a confirmed new runtime requirement or additional verification evidence.
| CL-REQ-032 | Compare explicitly supplied target libgcc against exact recorded direct owner/version nodes and function imports; preserve weak distinctions, incomplete results, independent GLIBC checks and legacy inspect behavior. User-approved checker scope, 2026-11-03; [contract](TARGET-LIBGCC.md). | [gcc.rs](../src/gcc.rs), optional [exact parser](../src/elf/exact.rs), inspect-only CLI; read-only companion byte digest. | Focused matcher/parser/CLI unit tests and [native static fixtures](../scripts/check-target-libgcc.py); [source qualifier](../scripts/qualify-target-libgcc.py). | Implemented—partial verification: released 0.5.0; reviewed source 972783d / source evidence below, published libc-abi-tools 0.5.0 is adopted from registry; Q50/I50 distinguish candidate composite/native Windows controls from actual fresh published-registry installation; exact release/tag/archive/docs.rs are verified with bounded scope. No loader/runtime/plugin/transitive or old-system certification. |

| CL-REQ-033 | Reject duplicate cargo-libc-owned value options, including identical repeats, before resolving inputs; keep switches idempotent, defaults and forwarding boundaries unchanged. User-requested CLI consistency work, 2026-11-07; [scope and migration](USER-GUIDE.md#duplicate-value-options-060); [AGENTS](../AGENTS.md#scope-and-implementation). | [cli.rs](../src/cli.rs), shared configuration parser and inspect ceiling parser. | Parser tests `singleton_paths_reject_same_and_different_values_on_every_config_command`, `single_paths_keep_defaults_and_explicit_values`, `repeated_switches_and_passthrough_remain_unchanged`; process regression [cli_options.rs](../tests/cli_options.rs) checks stderr, empty stdout and status 1 before work. | Implemented—partial verification; released 0.6.0, minor-release contract change; [qualification record](RELEASE-0.6.0.md) separates candidate and published-install evidence. Focused portable parser/process coverage; delegated provider parser, native linking and deployment qualification are outside this change. |

## Documentation map

| Path / surface | Classification | Authority, scope and update rule |
| --- | --- | --- |
| [README](../README.md) | Maintained entry guide | Purpose, prerequisites, registry setup, guide/trace links; packaged, so excluded-doc links use absolute GitHub URLs. |
| [AGENTS](../AGENTS.md), [CLAUDE shim](../CLAUDE.md) | Maintained repository instructions | AGENTS is primary; shim points only to it, never a second authority. |
| [USER-GUIDE](USER-GUIDE.md) | Maintained guide/reference | CLI, build setup versus deployment runtime, targets/providers, C/Python, both plain-Cargo formats, streams/status and limits. CL-REQ-002/013 deployment claims have [dated source/command evidence](evals/2026-11-03-libgcc-deployment.md). |
| [ARCHITECTURE](../ARCHITECTURE.md#implemented-build-and-execution-flow) | Maintained design/reference + Mermaid flow | Implemented selection, delegated clock overlay/cache, host/target, artifact and runner boundaries: CL-REQ-003/005–013/015–017/020/024–025/027. Source-backed diagram; named verification gaps and planned/legal/tier 3 boundaries are exclusions, not delivered arrows or new evidence. |
| [PROVIDER-CONTRACT](PROVIDER-CONTRACT.md) | Maintained format reference with historical examples | Six-field v1 consumer contract; manual exporter examples are producer development fixtures, not a new SDK. |
| [CARGO-WORKFLOWS](CARGO-WORKFLOWS.md) | Maintained restrictions + historical fixture evidence | Current argument rules; explicitly dated qualification applies to listed refs. |
| [RELEASING](RELEASING.md) | Maintained policy | CL-REQ-023: independent versions, authorization, registry/package/install, changelog/GitHub notes and bootstrap checks; deferred tier 3. |
| [CHANGELOG](../CHANGELOG.md), [versioned notes and history audit](releases/HISTORY.md) | Maintained user-facing history / dated reconstruction | CL-REQ-023: all eight tags, original registry dates, source identities, upgrade impact and attributed evidence/gaps; notes backfill is absent at old tags. |
| [Release-doc checker](../scripts/check-release-docs.py), [workflow](../.github/workflows/release-docs.yml) | Executable documentation checks | CL-REQ-023: local structure/current-version coverage; published mode checks exact bodies/tags/latest. Definitions and local success do not prove hosted bootstrap or publication. |
| [Deferred unwinder plan; historical filename](UNWINDER-0.4.0-PLAN.md) | Planned investigation / unapproved runtime design | CL-REQ-010/011/020/023/027 are existing boundaries to preserve, not evidence for the proposal. Archive/interface/default and shared-library decisions remain open; no new requirement ID or shipped capability. |
| [ROADMAP](ROADMAP.md) | Planned/deferred work and adopted decisions | Future features remain proposals until approved; prebuilt legal and real-old-system gaps stay open. |
| [FEATURE-REVIEW](FEATURE-REVIEW.md) | Historical decision inventory / proposed work | Preserve F/E IDs and historical dispositions; use maintained guide for implemented command scope. |
| [VALIDATION](../VALIDATION.md) | Historical evidence index | Keep named old outcomes unchanged; current-release pointer prevents stale pending work being read as current status. |
| [PROTOTYPE-VALIDATION](PROTOTYPE-VALIDATION.md), [MIGRATION](MIGRATION.md) | Historical evidence | Prototype report body is an immutable imported record; migration includes labeled current install guidance. |
| [CANDIDATE-INTEGRATION-0.3.2](CANDIDATE-INTEGRATION-0.3.2.md), [COMBINED-STDOUT-INTEGRATION-0.3.2](COMBINED-STDOUT-INTEGRATION-0.3.2.md) | Historical source-candidate evidence | Exact disposable source patches/refs are not current registry adoption. |
| [REGISTRY-VALIDATION-0.2.1](REGISTRY-VALIDATION-0.2.1.md), [RELEASE-0.2.1](RELEASE-0.2.1.md) | Historical release evidence | Preserve 0.2.1 archive/source/checksums; 0.2.2 is a different immutable release. |
| [RUN-STDOUT-FIX](RUN-STDOUT-FIX.md) | Historical correction / qualification evidence | Includes superseded invalid fixtures and the completed adoption pointer; do not resurrect old expected failures. |
| [RELEASE-0.2.2](RELEASE-0.2.2.md) | Historical release evidence, fixed refs | Source/registry/install/archive/tag/docs.rs evidence and exclusions; append corrections rather than invent new execution. |
| [src/lib.rs](../src/lib.rs) and public Rust doc comments / [docs.rs 0.3.0](https://docs.rs/cargo-libc/0.3.0/cargo_libc/) | Maintained source API docs / immutable published snapshot | Internal implementation entrypoints, not a stable external library API; no Rust examples changed in this audit. |
| [CI](../.github/workflows/ci.yml), [manual integration](../.github/workflows/candidate-integration.yml), [scripts](../scripts), [examples](../examples) | Executable validation definitions / fixtures | Definitions describe possible coverage; only named successful runs supply execution evidence. |
| [LICENSE](../LICENSE), [NOTICE](../NOTICE.md), [banner](assets/banner.svg) | Maintained legal/reference / presentation | Preserve attribution and licenses; banner is not part of the current package include list. |

| [RELEASE-0.2.3](RELEASE-0.2.3.md) | Historical release evidence | Official archive/VCS/checksum, registry qualification and published locked installation passed; real-browser docs.rs links/banner verified. |

| [0.2.3 render assets](assets/release-0.2.3/render-proof.json) and [screenshot](assets/release-0.2.3/docsrs-readme.png) | Historical render evidence | Captured exact docs.rs version, actual link URLs/HTTP statuses and loaded banner; excluded from package. |

| [RELEASE-0.2.4](RELEASE-0.2.4.md) | Historical release evidence | Minimal version-neutral README navigation fix; R24/I24, official archive/tag and actual docs.rs render verified; no runtime/dependency change. |

| [RELEASE-0.5.0](RELEASE-0.5.0.md), [candidate identities](evals/2026-11-03-target-libgcc-candidate.json) | Current release / candidate and publication evidence | CL-REQ-032; Q50/I50 distinguish prepublication composite/native Windows from separate published installation, exact archive/tag/docs.rs/source identity, bounded memory/runtime exclusions. |
| [RELEASE-0.4.0](RELEASE-0.4.0.md), [package inventory](evals/2026-11-03-diagnostics-package-inventory.json) | Historical release / candidate and publication evidence | CL-REQ-002/005/013/020/023/031; Q40/I40 distinguish prepublication versus fresh registry installation, exact source/archive/tag/docs.rs identities and native scope/gaps. |
| [RELEASE-0.3.0](RELEASE-0.3.0.md), [clock investigation](evals/2026-11-02-clock-linking.md) | Historical release evidence / dated investigation | CL-REQ-006–010/020/022–024/027; qualified N30/R30/P30/A30 and published I30 evidence; preserve retired prototype failures and explicit remaining scope limits. |
| [TARGET-LIBGCC.md](TARGET-LIBGCC.md) | Maintained guide/reference | Authoritative two-file checker contract, parameters, statuses, JSON, bounds and bounded source/candidate/published qualification; CL-REQ-032. |

## Maintenance

Keep `CL-REQ-001` through `CL-REQ-027` stable. Add new IDs without renumbering,
reusing or converting unapproved F/E proposals into requirements. When behavior,
scope, CLI, prerequisites or policy changes, update the affected row, authoritative
guide, implementation reference and named evidence/gap together. `Verified` means
only the explicitly named bounded evidence; use `Implemented—partial verification`,
`Planned`, `Deferred` or `Not applicable` when appropriate, with the reason.

Validate documentation through source/CLI and workflow comparison, relative links
and anchors, rendered Markdown and diff review. Do not regenerate data or rerun
unrelated product suites for prose. Compile changed Rust examples only when they
change. Repository edits do not update existing crates.io archives or versioned
API documentation. The immutable 0.2.2 README uses relative links to excluded guides/banner and
presents CI coverage as running on every push. This audit fixes the repository
README links and distinguishes workflow definitions from actual run evidence.
Those presentation corrections can accompany a future separately authorized
release; this trace index alone requires none. No Rust API source/example change is proposed. A subsequent decision authorized
[documentation-only 0.2.3](RELEASE-0.2.3.md) specifically to distribute the corrected
README links on docs.rs, where the published relative guide/banner links fail;
crates.io's link rewriting does not repair that docs.rs snapshot. The trace index
alone was not a reason for publication. Registry adoption, exact candidate gates, normal merge, official publication,
docs.rs render and published installation for 0.2.3 passed (R23/I23, 2026-11-01); 0.2.2 records and package contents stay immutable.

The initial audit observed HTTP 404 for the shared documentation standard while
it was being introduced. Resolution, 2026-11-01: the
[shared standard](https://github.com/rustcommons/.github/blob/main/docs/DOCUMENTATION.md)
is live after central merge `c8a0b9c01aae5657dd0ef649bd37b18d81919ce5`. The GitHub
page returns HTTP 200; Contents API on main and that exact commit identifies blob
`1b659ff536d43bcae5b09626069e89488058ba6f`. Its required headings, table columns,
CL-REQ prefix, status vocabulary, authority and evidence rules were checked
against this index. The exact shared AGENTS section remains unchanged.
Cross-repository review may clarify links, but does not authorize changes outside
this repository or broaden qualification.

Freshness follow-up, 2026-11-01: published 0.2.3 still labels its 0.2.2 evidence
link as current. The completed [0.2.4 release](RELEASE-0.2.4.md) replaces that
label/link with version-neutral release-policy navigation. Dated version states
are maintained in [the release index](RELEASING.md#release-records). No new
requirement ID or behavior change is introduced; prior snapshots remain immutable.
