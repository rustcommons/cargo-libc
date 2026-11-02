# Executable unwinder boundary probes — 2026-11-02

Initial development evidence, not a 0.4.0 release or LLVM qualification. Main
baseline is `bcd20d5634d201ccd18590c0b1e29cba305f69b2` (published runtime 0.3.0).
The working source increment adds final-output classification and this native
probe; the runtime adapter is not yet wired. [Machine-readable results](2026-11-02-unwind-boundaries.json)
record the exact probe-driver hash and dirty worktree context rather than claiming
that baseline commit already contained it.

## Assumptions and measured results

- A final executable and DSO cannot share one workspace-wide runtime switch.
  Stable native rustc 1.90 commands actually captured two executable links
  (program/test), two shared links (cdylib/dylib), and **zero** rlib/staticlib linker
  calls. This supports final-link dispatch, not all Cargo/maturin/output cases.
- Exported executable archive definitions can interpose on DSOs. Three small
  **synthetic marker** cases passed: exported marker gives executable/DSO 7/7;
  archive hiding gives 7/11, also under no-as-needed. This is ELF ownership proof
  only, not LLVM/GCC opaque-context or personality compatibility.
- Native Rust baseline plain return, catch cleanup, nested catches/cleanup,
  panicking thread join, forced/formatted backtrace and uncaught panic status
  all passed (six cases; uncaught exit 101). Snapshot maps and raw stdout/stderr
  are saved, but snapshots miss transient dlopen/dlclose and are not a complete
  observed closure. No LLVM archive was selected.
- Actual native C `-fexceptions`/cleanup object imports `_Unwind_Resume` and
  `__gcc_personality_v0`, besides the deliberately unresolved callback.
  Covering twelve Aero `_Unwind_*` imports alone is not helper completeness.
  No native cleanup runtime or static personality implementation was tested.

Host: aarch64 Linux/glibc 2.36, Rust 1.90, GCC 12.2 and GNU binutils. This host
is **not** cargo-libc's supported x86-64 target/runtime qualification. Exact tool
output is in the JSON. Parent's equipped x86-64 WSL/Clang18/GCC13 must qualify
the final producer/runtime/consumer pair and valid DSO round trips separately.

## Commands and retained failures

```sh
RUSTUP_TOOLCHAIN=1.90.0 python3 scripts/probe-unwind-boundaries.py \
  --rustc /home/cloudroom-agent/.cargo/bin/rustc --cc /usr/bin/gcc \
  --evidence /tmp/cargo-libc-unwind-boundaries-20261102-ready
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --lib link_output --locked --offline
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --lib cc::tests:: --locked --offline
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --lib provider_links_tell_shared --locked --offline
CARGO_BUILD_JOBS=2 cargo clippy --all-targets --locked --offline -- -D warnings
```

Seven focused Rust tests, formatting and all-target Clippy passed. The full Rust
suite is reserved for the meaningful final runtime-integrated source, not this
scaffold. Native results are nine passing baseline/marker cases plus captured
output-kind and C-symbol checks. Command/fixture/readelf/output files and their
hash manifest remain outside the package under the named evidence directory.

An earlier capture attempt at `/tmp/cargo-libc-unwind-boundaries-20261102-final`
failed before Rust links because `-C linker-features=-lld` is unstable on this
**aarch64** target in Rust 1.90. The corrected portable probe uses stable
`-C linker-flavor=gcc`; it does not enable unstable options. The failed output
is retained and is not counted as a pass or an x86-64 product defect.

## Release blockers and next evidence

Exact producer LLVM archive API/build/config/content/provenance is pending. No
nightly/system libunwind, guessed archive, dynamic fallback or raw data edit is
used here. The selected executable default must preserve non-unwind helpers and
valid baseline-supported DSO/Python behavior; hidden/exported policies need real
context/personality ownership tests, not a marker test alone.

The [canonical checklist](../UNWINDER-RELEASE-CHECKLIST.md) maps U01–U22 to current
commands and precise gaps, including full runtime-dependent source/data overrides,
all twelve Aero symbols, static personality support, transient loaded-library
tracing, GNU/LLD, C-unwind round trips U10/U11, PyO3, same-targetdir config/content
identity, every new glibc bundle/floor and final registry/package/install gates.
Parent's fresh old0.3/new0.4/Zig 182-case capture waits for the final pair; strict
comparison stays unchanged and legitimate LLVM helper imports need member/floor
attribution. No consumer job, release action or full matrix was dispatched.


## Initial native cross-DSO proof — later 2026-11-02

The supplied `unwind040-first-native-coexistence-proof.tar.gz` SHA-256 is
`841673515218814d49825314791e57c403ca331309a59bd43ad819e36f61d283`.
The checksum and all 1,440 entries in its `PARENT-SHA256.json` were verified
read-only, without executing or modifying its binaries. A compact
[audit/result record](2026-11-02-native-unwind-coexistence.json) retains archive,
fixture/build identities, exact statuses and representative versioned bindings.
This is native x86-64 WSL / Rust 1.90 with host SDK; LLVM 23.1.4 archives
were rebuilt using Clang 18 and glibc 2.17 headers. The capture labels its groups
`gnu` and records PATH GNU ld 2.42, but the actual driver arguments retain rustc’s
`-fuse-ld=lld`; the resulting host ELF `.comment` identifies LLD 20.1.8. The
prototype wrapper only appends a linker choice for `PROBE_LINKER=lld` and does not
force GNU for `gnu`. Treat these results as the observed LLD-selected capture;
GNU qualification requires corrected explicit selection/verification. It is not a completed
2.17 runtime/provider-floor or cargo-libc candidate qualification.

| Prototype policy | U07 internal DSO catch | U10 executable-origin round trip | U11 DSO-origin round trip |
| --- | --- | --- | --- |
| Existing GCC baseline | Exit 0 | Exit 0 | Exit 0 |
| Hidden static LLVM | Exit 0 | SIGABRT (-6) | SIGSEGV (-11) |
| Hidden archive plus broad host export control | Exit 0 | SIGABRT (-6) | SIGSEGV (-11) |
| Rebuilt nonhidden archive, hidden-at-link control | Exit 0 | SIGABRT (-6) | SIGSEGV (-11) |
| Rebuilt nonhidden archive plus broad host `--export-dynamic` | Exit 0 | Exit 0 | Exit 0 |

The hidden/default design is therefore a **concrete blocker**, not an unverified
hypothesis. Removing direct libgcc_s and passing an internal catch did not preserve
valid cross-DSO C-unwind behavior. The nonhidden/broad-export diagnostic shows
versioned DSO and libgcc_s `_Unwind_*` getter bindings resolving to host LLVM;
hidden controls leave those DSO bindings on GCC. This supports the split-owner
context risk, but does not establish a complete context/ABI compatibility proof.
Broad export of unrelated Rust/application symbols is unacceptable as production
policy; do not promote it merely because these three cases pass.

Preserve DSO artifact/runtime link selection; distinguish that from **host-process
symbol binding**, where existing DSOs can resolve unwind calls to the executable's
LLVM definitions. “DSOs always use GCC” is not the contract. Preserve valid observable
behavior and verify actual personality/getter/resume/delete owners, load orders
and qualified symbol-export boundaries. The pending experiment is a minimal unwind
export policy with no unrelated Rust exports, not a private LLVM default or broad
`--export-dynamic` workaround.

Private-default wiring is paused. No full suite, runtime/consumer workflow or
release action was dispatched for this evidence-only update. U10/U11/U12 remain
blocked on a concrete binding design and fresh native proof; all other runtime,
helper, floor, DSO/PyO3 and registry gates remain as listed in the checklist.
