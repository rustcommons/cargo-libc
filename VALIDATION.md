# Validation — cargo-libc migration

Validated 2026-10-09. The original prototype's report is preserved unchanged in
[docs/PROTOTYPE-VALIDATION.md](docs/PROTOTYPE-VALIDATION.md); it is historical evidence,
not a claim that those exact tool versions were rerun for the migration.

| Environment | Toolchain |
| --- | --- |
| Windows x86_64 | Rust/Cargo 1.99.0 |
| Ubuntu WSL x86_64 | Rust/Cargo 1.98.1, glibc 2.39, Clang 18.1.3, GNU ld 2.42, GCC runtime 13 |

## Results

- 39 unit tests passed on Windows and Linux.
- Formatting and Clippy with warnings denied passed.
- The renamed Cargo command and sibling linker helper built and ran successfully.
- `scripts/smoke-linux.sh` passed with the host sysroot `/` and glibc ceiling 2.39.
- The example ran its host build script, procedural macro, and target thread.
- Verbose rustc arguments confirmed host artifacts received no target linker;
  the target executable used `libc-linker`.
- A target output directory containing a space worked.
- A repeated cached Cargo build passed artifact validation.
- An unrelated `/bin/true` selected as the build artifact was rejected.
- Inspecting the host `/bin/true` against 2.22 rejected its GLIBC_2.34 requirement
  with exit code 2 and a failing JSON report.
- The resulting executable required at most GLIBC_2.34, with the expected
  interpreter and no RPATH/RUNPATH.

The smoke script is committed and runs in the Linux CI job. Tests use the host
SDK; no glibc 2.17/2.22 sysroot, SLES runtime, or Rocky Linux 9 host was tested.
Native dependency closure and generated compatibility libraries remain future work.

Rust 1.90 remains the declared minimum. Its dedicated CI check has not been run
locally; hosted CI results remain pending the first push.

## Experimental glibc 2.17 provider

Rust 1.99.0/Clang 18.1.3/GNU binutils 2.42 passed both prebuilt and source-built glibc 2.17 Cargo examples in WSL. Host build scripts and procedural macros ran; target threads passed under each real loader. Highest direct GLIBC requirement was 2.16; no RPATH/RUNPATH. Independent link-map and runtime-path audits passed. Hosted Clang 21.1.8 and Clang 24.0.0 development-snapshot jobs passed both real 2.17 routes on 2026-10-10. The highest directly required symbol tag of 2.16 does not establish glibc 2.16 runtime support; the tested target remains 2.17. See [provider contract](docs/PROVIDER-CONTRACT.md).

Hosted evidence: [companion CI](https://github.com/rustcommons/cargo-libc/actions/runs/38055844632) at commit 7980d63873282cb9fef68bcaf08f8123546387ac and [provider/runtime CI](https://github.com/rustcommons/linux-abi-tools/actions/runs/38055872917) at commit 084d2b6316a3af6ab16f3d25cdd67187e0b852c3. Rust 1.99 Windows/Linux, Rust 1.90 all-targets check, and conventional sysroot smoke tests passed. Actual old distro/kernel qualification remains open.

## GNU glibc 2.22 extension

Local Rust 1.99.0 / Clang 18.1.3 / binutils 2.42 Cargo validation passed with the pinned GNU 2.22 source-built inputs. Host build scripts/procedural macros and the target thread ran, incompatible 2.17 selection failed, and all shared link/runtime inputs stayed in approved trees. Highest directly required GLIBC symbol tag was 2.18; this is not qualification for a 2.18 runtime. Both original 2.17 routes were rechecked successfully. Hosted Clang 21/24 qualification passed with Rust 1.99 on 2026-10-10 for both release profiles. Actual SLES qualification is separate.

Extended [provider/runtime evidence](https://github.com/rustcommons/linux-abi-tools/actions/runs/38058240922) qualified companion commit 695ac536a791650d0d351c74562e008c47a82392 with producer commit 39101a0bdf9d824e3fb89931552b0a771fee5e0e. Cargo rejected memfd_create through the target linker for both 2.17 routes and the 2.22 source route.

## GNU glibc 2.18 qualification

Consumer commit `edfca5b9f9fb811069f302e8d2d5ef2f99f7fd33` accepts explicit 2.18
provider selection with the unchanged six-field descriptor. Local Rust 1.99.0,
Clang 18.1.3, and GNU binutils 2.42 passed Cargo host build/proc-macro separation,
target threading, real 2.18 loader execution, link/runtime path auditing,
mismatched-selection rejection, and actual too-new memfd_create linker failure.
Both 2.17 routes and source-built 2.22 passed again. The highest direct GLIBC
import tag for this 2.18 executable was 2.18; matching-runtime execution was
actually tested, separately from that import measurement.

All six [hosted provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38066726167)
passed under Clang 21/24 for 2.17/2.18/2.22 with producer commit
`75945c278187899c537af636fdc311a07e27132f`. The producer also qualified the
pthread default-attribute boundary, exhaustive isolated linking and defaults,
and five focused real-library behavior suites for each real input profile.
[Companion CI](https://github.com/rustcommons/cargo-libc/actions/runs/38066613985)
passed Windows/Linux Rust 1.99 tests, Rust 1.90 checks, and conventional sysroot
smoke tests. Distribution/kernel, native dependency, and SDK qualification remain
separate work. The next provider in the producer roadmap is 2.19.

## Shared CI provider inputs

Producer `8939827c12bf0354b81c8c2f2707d0a2b7985b1b` passed all three shared
preparation jobs and six Rust 1.99/Clang 21 or 24 application jobs for
2.17/2.18/2.22 ([run](https://github.com/rustcommons/linux-abi-tools/actions/runs/38069527581)).
The unchanged consumer pin `edfca5b9f9fb811069f302e8d2d5ef2f99f7fd33` generated
new runner-local descriptors from restored target headers/CRT/stubs/real runtimes
and explicitly selected local GCC startup files. Cargo host build/proc-macro,
target threading, real execution, path auditing, mismatched-version and newer-API
negative tests passed. Both 2.17 routes are retained. This qualifies CI transfer
and rematerialization; user SDK publication remains separate work; version-suffixed parsing is qualified below.

## Versioned target syntax

Consumer ed4c997 passed [companion CI](https://github.com/rustcommons/cargo-libc/actions/runs/38074429107).
Producer cbf31dd pinned that revision and passed [all nine provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38076305526): three preparations and six Rust 1.99/Clang 21 or 24 application jobs across 2.17/2.18/2.22, including both 2.17 routes.
Suffix builds retained ordinary Cargo target artifact paths, host scripts/proc macros, target threading, direct ELF/path checks and real-loader execution. Provider mismatch, suffix/explicit-version conflict and too-new memfd_create rejection passed. SDK acquisition, new platforms and actual old-kernel/distribution qualification remain separate.

## Cargo workflow qualification (F02)

Producer c442310 with consumer 6b07260 passed [all nine provider jobs](https://github.com/rustcommons/linux-abi-tools/actions/runs/38077381465) under Rust 1.99 and Clang 21/24 across glibc 2.17/2.18/2.22, including both 2.17 input routes. Workspace/exclusion, binary/example and package selection, feature/no-default-feature options, release/custom profiles, locked/offline and fresh cached builds passed. Real runtimes executed all three fixture programs. Selecting an existing executable absent from the current Cargo invocation was rejected. The wrapper still validates one explicitly selected executable, not every workspace output.

## GNU glibc 2.19 qualification

Consumer `2a6d0e57f915c774d71b1261b6707014d2bde47e` accepts explicit 2.19 provider
selection with the unchanged six-field descriptor; its CI passed. Locally (Rust
1.99.0, Clang 18.1.3, GNU binutils 2.42) and in producer `fb5d99f977a34d8f75c455b947a22f83ef57ea35`'s
[provider workflow](https://github.com/rustcommons/linux-abi-tools/actions/runs/38142660054), all twelve jobs
passed, including both Clang 21/24 jobs for 2.19: Cargo host build/proc-macro
separation, target threading, real 2.19 loader execution, path auditing, the
tier 1 scan, the tier 2 local run under a Linux 3.10 filter, `check`,
mismatched-selection and conflicting `--glibc` rejection, and the too-new
memfd_create linker failure. glibc 2.19 adds no versioned interfaces on x86_64,
so no new boundary fixture applies. Distribution (Ubuntu 14.04, Debian 8),
kernel, native dependency, and SDK qualification remain separate.

## GNU glibc 2.20 qualification

Consumer `8444108c1b83e72ef484b95f756308de34725844` accepts explicit 2.20 provider
selection with the unchanged six-field descriptor; its CI passed. Locally (Rust
1.99.0, Clang 18.1.3, GNU binutils 2.42) and in producer `5ed1e12856f98566dfb2c61d0c64ce7922306b27`'s
[provider workflow](https://github.com/rustcommons/linux-abi-tools/actions/runs/38144128128), all fifteen jobs
passed, including both Clang 21/24 jobs for 2.20: Cargo host build/proc-macro
separation, target threading, real 2.20 loader execution, path auditing, the
tier 1 scan, the tier 2 local run under a Linux 3.10 filter, `check`,
mismatched-selection and conflicting `--glibc` rejection, and the too-new
memfd_create linker failure. glibc 2.20 adds no versioned interfaces on x86_64;
its measured default changes (libpthread `vfork` became compatibility-only) are
covered by the producer's exhaustive default checks. Distribution, kernel,
native dependency, and SDK qualification remain separate.

## GNU glibc 2.21 qualification

Consumer `16a464ea88aa3b54d652583303fe6ed6c4d26e58` accepts explicit 2.21 provider
selection with the unchanged six-field descriptor; its CI passed. Locally (Rust
1.99.0, Clang 18.1.3, GNU binutils 2.42) and in producer `85b69c8ad27be4ae486e6723ca0698a06ba96c92`'s
[provider workflow](https://github.com/rustcommons/linux-abi-tools/actions/runs/38147732823), all eighteen jobs
passed, including both Clang 21/24 jobs for 2.21: Cargo host build/proc-macro
separation, target threading, real 2.21 loader execution, path auditing, the
tier 1 scan, the tier 2 local run under a Linux 3.10 filter, `check`,
mismatched-selection and conflicting `--glibc` rejection, and the too-new
memfd_create linker failure. The producer's new fixture confirms that 2.21's
compatibility-only `sigvec` cannot be linked by new code. Every release from
2.17 through 2.22 now has a qualified provider. Distribution, kernel, native
dependency, and SDK qualification remain separate.

## Every glibc release, 2.17 through 2.44

Consumer `bfe04693f6685770b47c9346e32de556a8029f4e` accepts provider descriptors for every release in
the pinned ABI lists and links `libpthread_nonshared.a` only before 2.28. Producer
`ae68bf63f5ddde7fc7852e76eab83ad2c1beb2c0`'s [provider workflow](https://github.com/rustcommons/linux-abi-tools/actions/runs/38158161339) passed all 84 jobs: 28
preparations and 56 Rust 1.99/Clang 21 or 24 applications. For every release, Cargo
host build/proc-macro separation, target threading, real loader execution, path auditing,
the tier 1 scan, the tier 2 local run under a Linux 3.10 filter, `check`,
mismatched-selection and conflicting `--glibc` rejection, and a matrix-selected too-new
linker failure passed. Executables built with a 2.34 or later provider require glibc 2.34
(`__libc_start_main@GLIBC_2.34` from `crt1.o`). Distribution, kernel, native dependency,
and SDK qualification remain separate.

## Host glibc check for test/run (2026-10-11)

cargo-libc `b809806500415b2053e26b4dedc55847fe5fb054` makes `libc-runner` compare a binary's
non-weak `GLIBC_*` requirements with the version definitions of the libraries the
loader would use whenever the binary runs on the host userland (no runner, or the
local runner), and refuse it with exit code 2 when a version is missing. Unit tests
cover the comparison, loader-order search paths and the new DT_VERDEF parser; the
Linux smoke test shows the passing path. Producer `1dabe9e761d41cfb8742b3a121ab3909c8efb0c2`'s
[84-job provider run](https://github.com/rustcommons/linux-abi-tools/actions/runs/38166901453) adds an end-to-end check for every provider newer
than the CI host's glibc 2.39: a program using `sched_getattr@GLIBC_2.41` built with
the 2.41–2.44 providers was refused ("the host's libraries lack libc.so.6
GLIBC_2.41"), while every provider's normal local run still passed. Releases whose
libc defines no version beyond the host skip that check.

## C dependencies with cc and CMake (2026-10-12)

cargo-libc `cef1ad2d2d7953d7c44e1fb88ddf1e3a47b507d0`'s [CI run](https://github.com/rustcommons/cargo-libc/actions/runs/38329386232)
built and ran `examples/c-deps` with automatic providers for every release 2.17–2.44
(Rust 1.99, Clang 21, CMake from the Ubuntu 24.04 runner). For each release the
executable needed no glibc version above the release. Under the tier 2 local runner
it reported that C compiled by the `cc` crate and by CMake saw that release's headers;
CMake's `check_function_exists(memfd_create)` failed before 2.27 and succeeded from
2.27; and the proc macro's C code saw the host's 2.39 headers. The same C code built
with plain Cargo needed `__isoc23_strtol@GLIBC_2.38`. C++ for the target was refused.
The Linux smoke test ran the example with `--sysroot /` too. C++, pkg-config and
system-library dependencies remain unqualified.

## Plain Cargo refuses a too-new symbol (2026-10-14)

cargo-libc `cc94dc3a8432f9e2a261ef70dfd020c67485195a`'s [CI run](https://github.com/rustcommons/cargo-libc/actions/runs/38555909208)
built `examples/too-new`, which calls `memfd_create` (`GLIBC_2.27`), with plain
`cargo build` configured by `cargo libc env`, in both the `sh` and `cargo` formats, for
every release 2.17–2.44 (Rust 1.99, Clang 21). For each release before 2.27 the link
failed with an undefined reference to `memfd_create`, as it does with `cargo libc
build`; from 2.27 on it linked and needed no glibc version above the release.

## Container runner in SLES 12 SP5 userland (2026-10-14)

cargo-libc `a3a2d38dcc23e107c730e8f9fef515c2030188c1`'s [CI run](https://github.com/rustcommons/cargo-libc/actions/runs/38562062202)
(job `container-runner`, Rust 1.99, Clang 21, podman with static crun 1.28) built with
the `sles-12-sp5` preset (glibc 2.22 provider, Linux 4.12 kernel profile) and ran under
`--runner container` in `registry.suse.com/suse/sles12sp5:latest`: `examples/hello`
(thread result 42) and its unit test (with `--nocapture` passed to the harness) each
reported runtime glibc 2.22, and `examples/c-deps` reported glibc 2.22 headers for C
compiled by `cc` and CMake and ran on glibc 2.22. This is tier 2: SLES's real userland
on the runner's modern kernel, with older kernels simulated by seccomp; it is not a
SLES 12 SP5 host.

## linux-abi-tools 0.3.2 source candidate (2026-10-31)

The isolated [candidate integration report](docs/CANDIDATE-INTEGRATION-0.3.2.md)
records a passing x86_64 run against an exact producer commit, including glibc
2.17/2.28 providers, C dependencies, plain Cargo and the host-sysroot path.
Production dependency requirements and locks remain unchanged; registry adoption
and release qualification are separate gates.

## Combined stdout fixes with revised 0.3.2 candidate (2026-10-31)

The [combined integration report](docs/COMBINED-STDOUT-INTEGRATION-0.3.2.md)
records passing consumer/provider closed-stdout regressions and bounded x86_64
smokes at producer `53d3c6f` and consumer `b8ff22a`. It supersedes the earlier
source-candidate evidence for the stdout fixes and documents the corrected
fixture-only failure. Production dependencies remain unchanged; registry adoption
and publication are still gated.

## cargo-libc 0.2.1 registry-dependent package (2026-10-31)

The [registry release validation](docs/REGISTRY-VALIDATION-0.2.1.md) records passing
portable and bounded x86_64 checks at `c7fcd04`, with all four consumed producer
packages resolved from crates.io at 0.3.2. The proposed package also passed a clean
unpacked locked installation and installed-helper smoke checks. Published
cargo-libc installation and release identity verification follow publication.

## Published cargo-libc 0.2.1 (2026-10-31)

The [release record](docs/RELEASE-0.2.1.md) verifies official archive checksum and
source commit `767c16a`, the matching `v0.2.1` tag and successful docs.rs build.
[Run 41245509023](https://github.com/rustcommons/cargo-libc/actions/runs/41245509023)
passed clean `cargo install cargo-libc --version 0.2.1 --locked` on x86_64 and
installed command/helper, provider stdout and C/CMake build/run/inspect/refusal
checks. This is published registry-installation evidence, not old-system
qualification.
