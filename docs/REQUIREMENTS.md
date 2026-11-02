# cargo-libc requirements and documentation index

## Scope and sources

This index retains the published 0.2.4 baseline and records the approved 0.3.0
clock-profile candidate separately; it is not a backlog or universal compatibility claim.
Published 0.2.4 runtime/API behavior is unchanged from 0.2.3/0.2.2. Documentation audit: 2026-11-01, baseline main
`1ca27da9086ebf6cf19a6f2c0e858108b38cac46`; current immutable released source/tag:
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

- **S30:** [0.3.0 source candidate](RELEASE-0.3.0.md), Cargo `8e2a7d47529bc940673d6ec3e6e8325ffac7089d`, producer `1b3b51c8afad88e05b0dc238c15f5ee879f94184`; [run 41371076867](https://github.com/rustcommons/cargo-libc/actions/runs/41371076867) canceled while queued after a shell-format harness correction; replacement/standalone native evidence pending. Local Rust 1.90 focused clock tests/Clippy passed with a disposable root-only source patch; this is not registry/package evidence.

## Requirements

| ID | Requirement and source | Implementation / documentation | Verification evidence | Status and gaps |
| --- | --- | --- | --- | --- |
| CL-REQ-001 | Limit managed application builds to x86_64 Linux, upstream glibc 2.17–2.44, dynamically linked executables and `cdylib`. [Guide: limits](USER-GUIDE.md#limits); AGENTS scope; F27–F31 remain proposals. | `Toolchain::new/from_provider` in [toolchain.rs](../src/toolchain.rs); release list in [provider.rs](../src/provider.rs); [architecture](../ARCHITECTURE.md). | Audit of platform guards/release list; R22 source providers 2.17/2.28. | Implemented—partial verification; the 0.2.2 native release run did not rerun every glibc/provider/Python combination. Other hosts/architectures/libcs, static glibc and provider PIE are outside scope. |
| CL-REQ-002 | Setup requires Rust >=1.90 and suitable native tools; install all five sibling executables together. [Guide: install](USER-GUIDE.md#install). | [Cargo.toml](../Cargo.toml), `sibling` and Clang probes in toolchain.rs; [README](../README.md#installation). | R22 MSRV/all-targets check and package installation; I22 `cargo install cargo-libc --version 0.2.2 --locked`, five-helper/native smoke. | Verified; named Ubuntu toolchain only, not every Rust/Clang/GCC tuple. |
| CL-REQ-003 | Version-suffixed targets are coordinator syntax; strip the suffix for Cargo, reject conflicting distro/glibc/kernel selections and unsupported releases. [Guide: choosing a target](USER-GUIDE.md#choosing-a-target); F01. | Parser in [cli.rs](../src/cli.rs); unsuffixed Cargo target in toolchain.rs. | R22 includes `versioned_target_selects_ceiling_and_preserves_cargo_arguments` and `target_selection_rejects_conflicts_and_malformed_requests`. | Verified; parser fixtures and bounded provider selection, not distro certification. |
| CL-REQ-004 | Preserve Cargo OS-string arguments while refusing managed target/config/message-format overrides; permit the supported tool/program second separators. [Cargo reference](CARGO-WORKFLOWS.md); F02. | `validate_cargo_args`, `validate_tool_args` in toolchain.rs; [guide commands](USER-GUIDE.md#commands). | R22 unit tests `cargo_cannot_override_managed_target`, `test_and_run_pass_arguments_after_the_second_separator`; `smoke-commands.sh`. | Verified; arbitrary Cargo flags/compiler combinations are not exhaustively qualified. |
| CL-REQ-005 | Host build scripts/proc macros and their C units stay native; target flags/linker apply only to target units. [AGENTS scope](../AGENTS.md#scope-and-implementation); F12. | Explicit Cargo target and encoded flags in toolchain.rs; `CARGO_CFG_CARGO_LIBC_PROFILE` in [cc.rs](../src/cc.rs); [build control](USER-GUIDE.md#how-the-build-is-controlled). | R22 `smoke-c-deps.sh` and `smoke-linux.sh` verify host/target separation; I22 installed C/CMake fixture. | Verified; same-triple x86_64 fixture, not general cross compilation. |
| CL-REQ-006 | Delegate automatic provider preparation/cache to libc-abi-tools; explicit provider and sysroot/GCC runtime are alternate input routes. [AGENTS](../AGENTS.md#scope-and-implementation); [guide providers](USER-GUIDE.md#providers). | `Selection::resolve` in cli.rs; provider loading/input probes in toolchain.rs. | R22 provider/reuse and host-sysroot smoke; parser `provider_selection_is_explicit_and_excludes_sysroot_options`. | Verified; source-provider 2.17/2.28 and host sysroot, not all input routes/releases. |
| CL-REQ-007 | Exact requested provider selection; never silently fall back after refusal. [AGENTS](../AGENTS.md#scope-and-implementation); F26. | provider.rs `validate_selection`; cli.rs mode selection delegates to producer. | R22 `selects_each_supported_release_without_fallback` and provider mismatch/too-new-symbol smoke; Audit of mode error propagation. | Verified; source selection/refusal fixtures, not arbitrary third-party provider validation. |
| CL-REQ-008 | Treat v1 descriptors as trusted local inputs; validate six fields, bounded UTF-8, absolute contained paths, required files and exact header version. Do not claim authenticated SDK integrity. [Provider contract](PROVIDER-CONTRACT.md). | provider.rs parsing/file checks, toolchain.rs probes; producer owns full ABI verification. | R22 `rejects_wrong_versions_and_ambiguous_descriptors`; Audit of canonical-path/file/header checks. | Implemented—partial verification; not authentication, relocation or complete content hashing. All external inputs remain trusted. |
| CL-REQ-009 | Fingerprints/cache keys do not promise arbitrary in-place changes are detected; regenerate config after updates and clean/change target directory when inputs change in place. Provider version changes can require fresh preparation. [Guide](USER-GUIDE.md#providers); [release cache note](RELEASE-0.2.2.md#provider-cache-refresh). | toolchain.rs `profile_fingerprint`; published libc-abi-tools 0.3.3 provider preparation hashes; candidate `clock.rs` additionally records the producer profile and actual libc/RT hashes, not all SDK contents. | Audit of recorded producer source `d483fd5` and consumer fingerprint; R22 source-provider reuse. | Implemented—partial verification; no old-cache import/offline migration or concurrent replacement guarantee. |
| CL-REQ-010 | C via cc/CMake uses target headers/link inputs while host C remains native; do not silently use host package libraries. [Guide: C dependencies](USER-GUIDE.md#c-dependencies). | cc.rs wrappers/toolchain helpers; `examples/c-deps`; F15/F18 historical proposals do not add broader adapters. | R22 `smoke-c-deps.sh` for 2.17/2.28; I22 installed C/CMake build/run/inspect. | Verified; pkg-config/bindgen/system-library integration and arbitrary build scripts are outside these fixtures. |
| CL-REQ-011 | Refuse target C++ and static linking until corresponding target runtimes exist; allow native host C++. [AGENTS](../AGENTS.md#scope-and-implementation); [guide limits](USER-GUIDE.md#c-dependencies). | cc.rs language/link checks; `libc-cxx`. | R22 C++ refusal and unit argument checks; I22 installed target C++ refusal. | Verified; evidence is refusal, not target C++ support. |
| CL-REQ-012 | `build` inspects the named Cargo-reported executable, including fresh artifacts; `install` checks every built executable after installation. [Guide build/install](USER-GUIDE.md#commands). | Cargo JSON reader [cargo_output.rs](../src/cargo_output.rs); toolchain.rs build/install/verify paths. | R22 portable Cargo JSON tests, provider reuse and command smoke. | Verified; build does not inspect every workspace output; failed install checks occur after installation and need user cleanup. |
| CL-REQ-013 | Inspect direct ELF/version requirements only, retaining older ABI tags and distinguishing weak/non-glibc requirements; no dependency-closure/runtime/CPU guarantee. [Guide: inspect](USER-GUIDE.md#inspect); AGENTS older symbol rule. | [elf.rs](../src/elf.rs), [report.rs](../src/report.rs); program-header metadata path. | R22 ELF parser tests and `newer_weak_requirement_does_not_raise_required_floor`, `does_not_treat_libstdcxx_versions_as_glibc_versions`; native inspect. | Verified; direct-file scope only; transitive libraries/plugins and runtime compatibility remain gaps. |
| CL-REQ-014 | Bound malformed ELF/scanner processing without rejecting valid large fixed tables solely for exceeding 65,536 entries. [Release correction](RUN-STDOUT-FIX.md#combined-candidate-qualification). | report.rs file bound, elf.rs metadata bounds; registry linux-abi-scan >=0.3.3. | R22 `check-malformed-scan.py`: overflow/reused-chain rejection, aliased headers accepted, Cargo 79,207 and librustc 146,741 records accepted with named time/memory limits. | Verified; specific corpus/resource bounds, not every ELF or universal parser safety. |
| CL-REQ-015 | Tier 1 scan findings are scoped static evidence; strict mode makes warnings fail. [AGENTS tiers](../AGENTS.md#scope-and-implementation); [guide tiers](USER-GUIDE.md#testing-tiers). | report.rs `floors/scan/inspect_file`, linux-abi-scan; [architecture](../ARCHITECTURE.md#artifact-inspection-and-cargo-workflows). | R22 scan entry paths and process `inspection_results_survive_closed_stdout`, scan rendering/floor unit tests. | Verified; runtime-computed syscalls, library internals and fallback execution are not proved. |
| CL-REQ-016 | `test/run` use the runner, enforce glibc ceiling/known host-version checks and simulate a kernel floor only when selected. [Guide: test, run](USER-GUIDE.md#test-run). | [simulate.rs](../src/simulate.rs) host checks/runner plan; kernel-abi-tools. | R22 host comparison/plan tests and `check-run-streams.py` local kernel case; I22 installed runner cases. | Verified; simulation uses a modern host kernel, and exercised code paths only. |
| CL-REQ-017 | Container run/test preserve intended CARGO/RUST environment, stdin, stdout and numeric child status; use trusted images because forwarded variables may contain credentials. [Guide providers](USER-GUIDE.md#providers); [release](RELEASE-0.2.2.md). | kernel-abi-tools 0.3.3 container runner; simulate.rs selection. | R22 and I22 Docker/Podman run/test: spaced variables, unrelated-variable exclusion, redirected input, JSON stdout, exit 7. | Verified; named engines/image/profile only; forwarding is not a secret sandbox. |
| CL-REQ-018 | Owned Cargo setup/dry-run diagnostics use stderr; explicit reports/config use stdout. Ignore only BrokenPipe at write/flush boundary and continue operations. [Guide tiers/reference](USER-GUIDE.md#testing-tiers). | [output.rs](../src/output.rs), toolchain.rs diagnostic calls; shared provider output delegated. | R22 seven process regressions, provider later-error regression and native `check-run-streams.py`; I22 installed stream checks. | Verified; child output/errors remain the child's own handling. |
| CL-REQ-019 | Preserve deliberate error/refusal and numeric child statuses when final stderr reporting is unwritable. [Guide: reference](USER-GUIDE.md#reference). | five executable entrypoints, output.rs; toolchain.rs/simulate.rs status paths. | R22 process tests `all_five_final_errors_preserve_status_with_unwritable_stderr`, `runner_refusal_keeps_status_two_with_unwritable_stderr`, `compiler_child_status_survives_unwritable_stderr`; native exit 7. | Verified; does not guarantee shell-style signal-number propagation; signal-status policy is follow-up review. |
| CL-REQ-020 | Both plain-Cargo formats use unsuffixed target, generated machine-specific paths, separate inspection and target-only flags; shell format refuses whitespace paths. [Guide: plain Cargo](USER-GUIDE.md#plain-cargo-and-ci). | toolchain.rs `export_environment`; `--format sh/cargo`; [command reference](CARGO-WORKFLOWS.md). | R22 `smoke-commands.sh` covers both formats, build/test/inspection and too-new-symbol refusal; Audit of shell path checks. | Verified; arbitrary external Cargo configuration merging is not qualified. |
| CL-REQ-021 | PyO3/maturin `cdylib` workflow requires separate inspection, Python ABI/tag/dependency checks and actual target tests. [Guide: Python extensions](USER-GUIDE.md#python-extensions-maturin). | toolchain.rs shared-library path; [smoke-python-ext.sh](../scripts/smoke-python-ext.sh); F11 retains separate library-type scope. | Audit of shared-link path/script and CI Python job; historical guide workflow. R22/I22 explicitly excluded Python matrix. | Implemented—partial verification; no fresh 0.2.2 Python matrix or universal manylinux claim. |
| CL-REQ-022 | Keep implementation dependencies std plus first-party registry crates, preserve input attribution/licenses. [AGENTS](../AGENTS.md#scope-and-implementation); [NOTICE](../NOTICE.md). | [Cargo.toml](../Cargo.toml), [Cargo.lock](../Cargo.lock), NOTICE; embedded producer inputs under their licenses. | Audit of manifest/lock/include list and notices; R22 registry resolution; published archive verification in release record. | Verified; technical provenance is not legal clearance for prebuilt distribution. |
| CL-REQ-023 | Independently version cargo-libc; authorized releases require applicable gates, registry-only resolution, clean package install and immutable release identity. [Release policy](RELEASING.md). | [release record](RELEASE-0.2.2.md); [manual workflow](../.github/workflows/candidate-integration.yml); scanner/kernel minima 0.3.3, other direct minima unchanged. | R22 and I22; official archive checksum, clean VCS/tag `b87563c` and docs.rs checks dated 2026-11-01. | Verified; exact recorded releases only; R23/I23 and official 0.2.3 archive/tag/docs.rs checks extend this evidence, not authorization or proof for future releases. |
| CL-REQ-024 | Real old-system tier 3 qualification is deferred, not completed and not a routine-release blocker; new old-system claims require actual environment/workload evidence. [AGENTS validation](../AGENTS.md#validation); [roadmap](ROADMAP.md#deferred-qualification-and-infrastructure). | [Guide tiers](USER-GUIDE.md#testing-tiers), RELEASING policy. | Audit of adopted 2026-11-01 policy and release exclusions; no real old-system run is claimed. | Deferred; unavailable infrastructure/evidence remains an explicit gap. |
| CL-REQ-025 | Keep prebuilt mode unavailable pending legal review. [Roadmap decision](ROADMAP.md#maintainer-decisions); [guide providers](USER-GUIDE.md#providers). | cli.rs mode delegates refusal to libc-abi-tools; [architecture](../ARCHITECTURE.md#provider-and-sysroot-inputs). | Audit of producer 0.3.3 prepare mode refusal and maintained guide; no prebuilt publication/qualification in R22. | Deferred; legal clearance and a separately approved implementation remain required. |
| CL-REQ-026 | Maintain setup-first README, authoritative guides, stable trace IDs, identifiable historical evidence and immutable package snapshots. [AGENTS documentation rules](../AGENTS.md#documentation-and-requirements-traceability). | This index/map and linked maintained guides; published README/API docs remain immutable snapshots. | Audit 2026-11-01: source/command/link/render/diff checks; shared convention page HTTP 200 and Contents API blob `1b659ff536d43bcae5b09626069e89488058ba6f` verified against central merge `c8a0b9c01aae5657dd0ef649bd37b18d81919ce5`. | Verified; bounded documentation audit only, no package release or universal behavior verification implied. |

| CL-REQ-027 | Prefer libc defaults for all five clock functions without a public clock setting; retain explicit legacy versions, genuine RT APIs, linker order/state and unwinder behavior. Approved 0.3.0 scope in [candidate record](RELEASE-0.3.0.md); [guide](USER-GUIDE.md#clock-linking). | [clock.rs](../src/clock.rs) delegates the read-only ELF copy/cache to producer `LibcClocksV1`; target prefix, controlled in-place RT substitution, generated identity and fingerprint in toolchain.rs. Raw data, user files and descriptor v1 are unchanged. | S30 focused tests/source inspection; native mixed-reference/state/RT/same-directory-upgrade run pending. Rejected reordering evidence is retained in [evals](evals/2026-11-02-clock-linking.md). | Implemented—partial verification; no published registry candidate yet. Opaque response/input scripts refuse; container preparation, complete sysroot input closure and final package/install gates remain bounded gaps. |

## Documentation map

| Path / surface | Classification | Authority, scope and update rule |
| --- | --- | --- |
| [README](../README.md) | Maintained entry guide | Purpose, prerequisites, registry setup, guide/trace links; packaged, so excluded-doc links use absolute GitHub URLs. |
| [AGENTS](../AGENTS.md), [CLAUDE shim](../CLAUDE.md) | Maintained repository instructions | AGENTS is primary; shim points only to it, never a second authority. |
| [USER-GUIDE](USER-GUIDE.md) | Maintained guide/reference | CLI, targets, setup, providers, C/Python, both plain-Cargo formats, streams/status and limits. |
| [ARCHITECTURE](../ARCHITECTURE.md#implemented-build-and-execution-flow) | Maintained design/reference + Mermaid flow | Implemented selection, host/target, artifact and runner boundaries: CL-REQ-003/005–008/010–013/015–017/020/024–025. Source-backed diagram; planned/legal/tier 3 boundaries are exclusions, not delivered arrows or new evidence. |
| [PROVIDER-CONTRACT](PROVIDER-CONTRACT.md) | Maintained format reference with historical examples | Six-field v1 consumer contract; manual exporter examples are producer development fixtures, not a new SDK. |
| [CARGO-WORKFLOWS](CARGO-WORKFLOWS.md) | Maintained restrictions + historical fixture evidence | Current argument rules; explicitly dated qualification applies to listed refs. |
| [RELEASING](RELEASING.md) | Maintained policy | Independent versions, authorization, registry/package/install checks and deferred tier 3. |
| [ROADMAP](ROADMAP.md) | Planned/deferred work and adopted decisions | Future features remain proposals until approved; prebuilt legal and real-old-system gaps stay open. |
| [FEATURE-REVIEW](FEATURE-REVIEW.md) | Historical decision inventory / proposed work | Preserve F/E IDs and historical dispositions; use maintained guide for implemented command scope. |
| [VALIDATION](../VALIDATION.md) | Historical evidence index | Keep named old outcomes unchanged; current-release pointer prevents stale pending work being read as current status. |
| [PROTOTYPE-VALIDATION](PROTOTYPE-VALIDATION.md), [MIGRATION](MIGRATION.md) | Historical evidence | Prototype report body is an immutable imported record; migration includes labeled current install guidance. |
| [CANDIDATE-INTEGRATION-0.3.2](CANDIDATE-INTEGRATION-0.3.2.md), [COMBINED-STDOUT-INTEGRATION-0.3.2](COMBINED-STDOUT-INTEGRATION-0.3.2.md) | Historical source-candidate evidence | Exact disposable source patches/refs are not current registry adoption. |
| [REGISTRY-VALIDATION-0.2.1](REGISTRY-VALIDATION-0.2.1.md), [RELEASE-0.2.1](RELEASE-0.2.1.md) | Historical release evidence | Preserve 0.2.1 archive/source/checksums; 0.2.2 is a different immutable release. |
| [RUN-STDOUT-FIX](RUN-STDOUT-FIX.md) | Historical correction / qualification evidence | Includes superseded invalid fixtures and the completed adoption pointer; do not resurrect old expected failures. |
| [RELEASE-0.2.2](RELEASE-0.2.2.md) | Historical release evidence, fixed refs | Source/registry/install/archive/tag/docs.rs evidence and exclusions; append corrections rather than invent new execution. |
| [src/lib.rs](../src/lib.rs) and public Rust doc comments / [docs.rs 0.2.2](https://docs.rs/cargo-libc/0.2.2/cargo_libc/) | Maintained source API docs / immutable published snapshot | Internal implementation entrypoints, not a stable external library API; no Rust examples changed in this audit. |
| [CI](../.github/workflows/ci.yml), [manual integration](../.github/workflows/candidate-integration.yml), [scripts](../scripts), [examples](../examples) | Executable validation definitions / fixtures | Definitions describe possible coverage; only named successful runs supply execution evidence. |
| [LICENSE](../LICENSE), [NOTICE](../NOTICE.md), [banner](assets/banner.svg) | Maintained legal/reference / presentation | Preserve attribution and licenses; banner is not part of the current package include list. |

| [RELEASE-0.2.3](RELEASE-0.2.3.md) | Historical release evidence | Official archive/VCS/checksum, registry qualification and published locked installation passed; real-browser docs.rs links/banner verified. |

| [0.2.3 render assets](assets/release-0.2.3/render-proof.json) and [screenshot](assets/release-0.2.3/docsrs-readme.png) | Historical render evidence | Captured exact docs.rs version, actual link URLs/HTTP statuses and loaded banner; excluded from package. |

| [RELEASE-0.2.4](RELEASE-0.2.4.md) | Current published release evidence | Minimal version-neutral README navigation fix; R24/I24, official archive/tag and actual docs.rs render verified; no runtime/dependency change. |

| [RELEASE-0.3.0](RELEASE-0.3.0.md), [clock investigation](evals/2026-11-02-clock-linking.md) | Candidate scope / dated investigation | CL-REQ-006–010/020/022–024/027; keep retired prototype failures and pending source/registry/native boundaries explicit. |

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
