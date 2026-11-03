# Aero/Zig parity and native cleanup evidence — 2026-11-03

Read-only artifact/source audit plus explicitly attributed reviewer synthesis.
No new runtime tests, consumer edits, selector, integration, version or release
changes. The [canonical corner-case inventory](../UNWINDER-RELEASE-CHECKLIST.md#permanent-acceptance-cases-and-current-automation)
owns U01–U22; this record supplies dated provenance and coverage limits rather
than another requirements specification. Current system unwind stays unchanged.

## Actual Aero comparison

Saved [run41378111334](https://github.com/rustcommons/cargo-libc/actions/runs/41378111334)
used Cargo source `29582bc7af35cba1f741f989b47bc520979e13ae` and consumer
`9c248300f1e2ab20ce34dc4b227eb322e33e4c93`. The retained artifact ZIP SHA-256 is
`fc2430f375145f818366535ef8036daa4f8144fa792ff775d38c8a1e808113ff`.
The [machine audit](2026-11-03-zig-unwind-parity.json) records binary hashes, actual
version output, link flags, direct imports/owners and comparison stdout.
Binary hashes were checked against the saved `binaries.sha256`, without executing
the x86 binaries on this aarch64 inspection host.

| Saved variant | Direct NEEDED in recorded order | Direct unwind requirements | Comparison scope |
| --- | --- | --- | --- |
| cargo-libc0.2.4 | libgcc_s, librt, libpthread, libdl, libc | Twelve `_Unwind_*` imports owned by libgcc_s.so.1, GCC3.0/3.3/4.2.0 |182/182 behavior matches Zig; strict comparator exits1. |
| candidate 0.3.0 at29582bc | libgcc_s, libpthread, libdl, libc | Same twelve versioned GCC unwind imports |182/182 behavior matches Zig; strict comparator exits1 on requirements. |
| Zig 0.17.0 + cargo-zigbuild0.23.4 | libc, libpthread, libdl | No dynamic unwind imports **or defined dynamic unwind exports** in saved ELF | Reference application binary; no plugin/C forced-unwind or panic qualification follows. |

All three used Rust **1.99.1** (`58cf8f9c3`, rustc LLVM 23.1.1), not the native
Rust 1.90 coexistence toolchain. Cargo paths used Clang 18.1.3, GCC13.3/BFD2.42;
Zig target was `x86_64-unknown-linux-gnu.2.22`. The run used a modern hosted kernel,
not an actual SLES12/glibc2.22 runtime. Both comparison binaries' newest direct
GLIBC import was2.18. This is a direct-import observation, not full process closure.

The saved Rust driver arguments include `-lgcc_s`, `-nodefaultlibs`, `--as-needed`,
PIE, thin LTO/codegen-units1 and stripping. Zig used cargo-zigbuild's generated
`zigcc` wrapper; Rust's pre-wrapper log still contains `-lgcc_s`. Actual
[cargo-zigbuild0.23.4 source](https://docs.rs/crate/cargo-zigbuild/0.23.4/source/src/zig/linker_args.rs)
rewrites that argument to `-lunwind`. Its downloaded registry source archive has
SHA `0b0c2fe7d5f1a782047272e3fc1cd44214c450776bbde564bf92b9776dc2d781`,
VCS `4258db97f6ac4bd1fc6e5c714d3f8d01db5c35e8`. The original artifact does not
retain Zig's final linker command/member map, so exact selected runtime-member
identity cannot be reconstructed merely from the Rust log.

The pinned [Zig0.17 archive builder](https://codeberg.org/ziglang/zig/src/tag/0.17.0/src/libs/libunwind.zig)
builds PIC/native-only unwind code with hidden visibility and includes vendored
`gcc_personality_v0.c`. The Linux preparation helper builds unmodified LLVM 23.1.4
libunwind without that C personality. The matching Zig Python-wheel personality
member was read by bounded HTTP ranges and byte-matched to the tagged source;
its SHA is in the machine record. This does not verify the whole original hosted
wheel/executable checksum, and the runtime revisions/build contexts are not
assumed identical. No downloaded tool was installed or executed for this audit.

The twelve Aero imports are Backtrace, DeleteException, GetDataRelBase, GetIP,
GetIPInfo, GetLanguageSpecificData, GetRegionStart, GetTextRelBase,
RaiseException, Resume, SetGR and SetIP (each prefixed `_Unwind_`). Zig instead
imports additional libc helpers such as dladdr, fflush/fprintf/fwrite, rwlock
operations, snprintf and stderr. These need member-attributed floor evidence in
any future candidate comparison, not an invariant that all nonclock imports
remain identical. Preserve the consumer's strict comparator unchanged.

The [unchanged comparator source](https://github.com/joey-huckabee/aero1553/blob/9c248300f1e2ab20ce34dc4b227eb322e33e4c93/scripts/compare-binaries.py)
checks stdout/stderr/status for decode/count/dump/options/errors/help/version,
golden recordings and conformance inputs. It does not deliberately exercise C
pthread cleanup, cross-DSO unwinding, PyO3, nested panic/resume or the first
runtime loader transition. Its direct NEEDED/undefined-symbol check is not a
resolved transitive/lazy-load trace. The artifact lacks resolved DSO copies and
loader observations for all 182 cases; no universal absence of transitive libgcc_s
is inferred from Zig's direct ELF list.

## Existing native evidence and remaining fair comparison

[Existing E2–E4 proofs](2026-11-02-unwind-boundaries.md#corrected-minimal-exports-and-native-c-eh-follow-up--2026-11-02)
use Rust 1.90 and native hostSDK controls. Hidden LLVM fails valid U10/U11;
corrected narrow exports pass those bounded Rust round trips on BFD/LLD.
For the identical native C worker, plain controls pass but C `-fexceptions`
cleanup/exit/cancel fails in the recorded LLVM variants while GCC passes.
Loader logs bind `__gcc_personality_v0` to dynamic GCC and can show GCC getter
references interposing on the LLVM host. The safe interpretation is a measured
mixed-owner compatibility problem, not an approved private-context/layout fix.

Aero 182 success and these failures are not equivalent coverage. None establishes
that every LLVM recipe or every Zig application fails. The next discriminating
control is a **single matched native fixture** using the same GCC-built C EH
worker object, same Rust toolchain/profile and actual Rust/cargo-zigbuild driver
versus the recorded prototype recipe and system oracle. Keep pthread-exit/cancel
worker stacks entirely C and local Rust catches inside their runtime; do not
manufacture an ABI-violating panic/cancellation case. Retain failures at compile,
link and runtime separately. Capture the real wrappers/linker commands/member
maps, direct exports/imports and runtime loader owners before interpreting results.
This bounded probe remains separate work; it has not run here and is not a new
matrix or a reason to relax existing DSO/Python requirements.

## Reviewer-supplied native findings

The delivered `cornercase-review-coverage.md` SHA
`af39cdfc81578e8fc366c47e431b6324366860ccc48de5806894e08330b82b65` identifies these additional packets and inspection results.
They were supplied as independent synthesis; their raw packets/GDB log are not
stored or newly audited in this checkout. [Machine record](2026-11-03-zig-unwind-parity.json)
preserves exact supplied digests and this attribution.

- **E5:** packet SHA `d37ee43cf662249185972ba56b96324afcaf2b87219c374ad1ef4fcdeb492fba`;
  helper archive `b96227b64097a311305eab92966607770a6441c4df72dbf967a4fcf5d9ad1237`.
  Selected/hidden personality still yields native exit/cancel SIGSEGV on BFD/LLD.
  A missing personality alone therefore does not explain away the recorded failure.
- **E6:** packet SHA `daab0ea02a5c2c56d359f75e67034656d9f270c2eb7128719d6048a7bc0863bf`;
  actual Zig C binary SHA `14a27cb5d6d8cf26a51e2a808d73b7b69d6bddc714868169dc7fe5d9ca6fe519`;
  unchanged fixture SHA `309df88609a84558680e544d3be77c9935077b6ac650bf0726bdca1106336766`.
  Zig 0.17.0/vendorClang22.1.8, target glibc2.22, explicit `-lunwind` on host glibc2.39:
  backtrace passes, pthread-exit/cancel SIGSEGV. Original no-`-lunwind` link failure
  is retained, not discarded as a pass.
- **E7:** `native-zig-pthread-exit.gdb.log`, 2026-11-03, same verified E6 binary.
  The exact-stack gap is **closed for that C fixture**: GCC `_Unwind_ForcedUnwind`
  -> Zig-local `__gcc_personality_v0` -> local LLVM `_Unwind_GetLanguageSpecificData`
  -> `__unw_get_proc_info` -> stack-address SIGSEGV. This is not actual Rust-driver,
  Aero, glibc 2.22-runtime/old-kernel or all-Zig evidence. Exact private cursor-field
  and instruction decoding remain unclaimed; no opaque ABI tricks are proposed.

## Design feedback kept separate

[Canonical lifecycle v2](2026-11-02-llvm-lifecycle-design-v2.md) is archived
verbatim, SHA `436853b803d1e30c439a828853a77e06eee3efa3a542cbb9996cdc24a2ff8fb5`.
Its approved content remains DESIGN: R/I/A/E, each distinct observed-change
invalidation, missing-receipt unknown state, final-output byte binding, no cache
repair, explicit plain-Cargo refresh and future same-directory CMake/Ninja relink.
No ledger, execution guard or automatic content freshness is implemented.

Three recorded-validator clarifications were sent as **design feedback**:
decode retained records despite missing current inputs; compare tokens only
within their record binding and avoid phantom E changes when safely rebasing;
make opaque record/expected I/A/outer-receipt binding checkable. The proposed
read-only helper is not runtime integration or a new application policy. This
work is not the priority over the active interoperability blocker.
