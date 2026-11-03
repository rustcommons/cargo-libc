# Current libgcc deployment documentation checks — 2026-11-03

Documentation/source/read-only inspection evidence, separate from unwinder
research. Baseline main `bcd20d5634d201ccd18590c0b1e29cba305f69b2`, package 0.3.0.
No code, packaged README, version, dependency, lock, workflow or runtime policy
changed. See the maintained [deployment guide](../USER-GUIDE.md#deployment-runtime-prerequisites).
This record does not qualify a target machine or add a GCC validation feature.

## Saved Aero imports

[Run 41378111334](https://github.com/rustcommons/cargo-libc/actions/runs/41378111334),
2026-11-02: Cargo source `29582bc7af35cba1f741f989b47bc520979e13ae`, Aero
`9c248300f1e2ab20ce34dc4b227eb322e33e4c93`, Rust 1.99.1. The saved old 0.2.4 and
candidate 0.3.0 reports have exactly these twelve libgcc-owned imports. Names,
versions and library owners were compared with `separate-clock-assessment.json`
and the saved `readelf` dynsym/version-needs views for **both** Cargo variants.

| Symbol | Required version | Supplying library |
| --- | --- | --- |
| `_Unwind_Backtrace` | `GCC_3.3` | `libgcc_s.so.1` |
| `_Unwind_DeleteException` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_GetDataRelBase` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_GetIP` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_GetIPInfo` | `GCC_4.2.0` | `libgcc_s.so.1` |
| `_Unwind_GetLanguageSpecificData` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_GetRegionStart` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_GetTextRelBase` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_RaiseException` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_Resume` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_SetGR` | `GCC_3.0` | `libgcc_s.so.1` |
| `_Unwind_SetIP` | `GCC_3.0` | `libgcc_s.so.1` |

Maximum observed GCC label: **GCC_4.2.0**; highest required numeric GLIBC: **2.18**.
This is neither “install GCC 4.2” nor a universal cargo-libc/SDK requirement. Provider
reference-table provenance is a different fact from the subset this application
uses. Both Cargo variants match 182/182 behavior against Zig, but the unchanged
strict requirements comparator exits 1. No actual old-SLES runtime/kernel or
arbitrary target runtime was qualified by this modern-host run.

Saved artifact ZIP SHA-256:
`fc2430f375145f818366535ef8036daa4f8144fa792ff775d38c8a1e808113ff`.
Candidate binary SHA-256:
`9843ac75bad2e74d0f5417767256f6c16269e435cc414292ce677d28caca2c6c`.
Old binary SHA-256:
`2884c480365ffe8f3afd99c522602e98821c7415b3fed1689670323134f4d544`.
Both binaries match the saved `binaries.sha256`. No saved executable was run for
this documentation check.

## Current source and command checks

- `src/elf.rs` records NEEDED, search paths and per-library version requirements;
  it is not a complete required-symbol/export matcher or a loader resolver.
- `src/report.rs::assess` considers `GLIBC_*` only. Text/JSON still list GCC version
  needs. JSON explicitly reports `transitive_dependencies_checked:false` and
  `runtime_tested:false`. `--max-glibc` success is not GCC/runtime validation.
- Resolved registry `linux-abi-scan 0.3.3` was checked through locked/offline Cargo
  metadata. Its `glibc_checks` explicitly skips GCC/GLIBCXX version labels; strict
  warning handling and verbose display do not add a GCC floor/export check.
  Corresponding [current producer source](https://github.com/rustcommons/linux-abi-tools/blob/b5eb64899fa5e5eed2c33c7ad3e42e5df945cadc/crates/linux-abi-scan/src/lib.rs)
  retains that scope. `simulate.rs::host_glibc` also filters nonweak `GLIBC_*`;
  it is not fail-closed GCC validation and the real loader can still fail.
- Resolved `libc-abi-tools 0.4.0`'s GCC 4.8.5 reference TSV is byte-identical to
  [the inspected producer table](https://github.com/rustcommons/linux-abi-tools/blob/b5eb64899fa5e5eed2c33c7ad3e42e5df945cadc/src/libgcc_s-gcc-4.8.5.tsv):
  138 definitions, 11 version nodes through GCC_4.8.0. Generated-provider function
  stubs trap and are never deployment runtimes; this does not describe every
  full-sysroot link. No exact deployment compiler/package version follows.

The cached CLI was 0.2.2, so only the current main CLI was rebuilt for command
verification, with existing dependencies and two workers:

```sh
CARGO_BUILD_JOBS=2 cargo build --bin cargo-libc --locked --offline
target/debug/cargo-libc --version
target/debug/cargo-libc inspect /path/to/saved/new --max-glibc 2.22 --json
target/debug/cargo-libc inspect /path/to/saved/new --max-glibc 2.22
```

The rebuilt CLI reports 0.3.0. Read-only inspection of the above candidate ELF exits 0,
reports direct glibc check `pass`, highest GLIBC 2.18 and GCC_3.0/GCC_3.3/GCC_4.2.0,
while transitive/runtime fields stay false. The source/command check ran on an
**aarch64 inspection host reading x86_64 ELF**, not a target execution test.

The guide's two executable `readelf` commands were run on that exact saved ELF.
The two runtime-library commands were syntax-checked against the local aarch64
libgcc DSO only; that different-architecture file was **not** used as a compatible
runtime for the Aero binary. Outputs are retained under
`/tmp/cargo-libgcc-deployment-doc-checks/`. RPM is absent on this inspection host;
`rpm -q libgcc_s1` and `rpm -ql libgcc_s1` are documented read-only target queries,
source-checked against the [RPM manual](https://rpm.org/docs/4.20.x/man/rpm.8),
not claimed as executed on SUSE. No runtime package or local tool was installed.
No Rust suite, provider/native matrix or deployment compatibility test was run.

## Primary references and bounded package example

[GCC low-level runtime](https://gcc.gnu.org/onlinedocs/gccint/Libgcc.html) explains
compiler-support/exception routines. [GCC ABI history](https://gcc.gnu.org/onlinedocs/libstdc++/manual/abi.html)
distinguishes SONAME release identity from cumulative symbol-version nodes and
lists GCC_4.2.0 with GCC 4.2. Neither package labels nor the largest version node
alone proves the complete required export set.

The official [SUSE image changes for 2025-01-03](https://publiccloudimagechangeinfo.suse.com/google/sles-12-sp5-v20250103-x86-64/package_version_changes.html)
list libgcc_s1 `13.3.0+git8781` (build 1.16.1) for that SLES 12 SP5 Google image.
This is a dated image record, not a claim all SLES installations contain it or
that their glibc version determines the GCC package version.

For the brief Zig explanation, registry cargo-zigbuild 0.23.4 archive SHA
`0b0c2fe7d5f1a782047272e3fc1cd44214c450776bbde564bf92b9776dc2d781`,
VCS `4258db97f6ac4bd1fc6e5c714d3f8d01db5c35e8`, was inspected:
`src/zig/linker_args.rs` maps `-lgcc_s` to `-lunwind`. Zig 0.17's tagged
[src/libs/libunwind.zig](https://codeberg.org/ziglang/zig/src/tag/0.17.0/src/libs/libunwind.zig)
builds its bundled target archive. The toolchain explanation is an inference from
that mechanism, not a quotation of maintainer intent or a universal safety/no-GCC-
loading guarantee. It does not change cargo-libc's present deployment contract.

Documentation verification: the guide, trace and this record were rendered with
GitHub's Markdown API; local file/heading links and all twelve rendered pair rows
were checked. Official GCC pages were retrieved through the source reader; direct
HTTP clients received403 here, so no direct-HTTP200 result is claimed for them.
SUSE/RPM/Zig source pages returned200; producer guide/source paths were checked
through GitHub's Contents API. The release-doc checker and diff checks passed.
