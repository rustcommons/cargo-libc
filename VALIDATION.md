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

Rust 1.99.0/Clang 18.1.3/GNU binutils 2.42 passed both prebuilt and source-built glibc 2.17 Cargo examples in WSL. Host build scripts and procedural macros ran; target threads passed under each real loader. Highest direct GLIBC requirement was 2.16; no RPATH/RUNPATH. Independent link-map and runtime-path audits passed. Clang 21/23 hosted jobs live in the companion libc-abi-tools workflow and remain unobserved. See [provider contract](docs/PROVIDER-CONTRACT.md).
