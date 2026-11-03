# Native unwind coexistence regression fixtures

Diagnostic fixtures retained from the two 2026-11-02 native proof packets, not
production linker policy or an approved runtime archive. The archived source of
`bridge.rs`, `host.rs`, `link.py`, `run.py` and `native_load_controls.c` is preserved
byte-for-byte; hashes and packet identities are in the
[follow-up audit](../../../docs/evals/2026-11-02-unwind-minimal-and-c-eh.json).
The canonical [release checklist](../../../docs/UNWINDER-RELEASE-CHECKLIST.md)
tracks actual qualification; these files do not make its pending rows green.

`bridge.rs` and `host.rs` exercise internal DSO catch (U07) and valid C-unwind
round trips back to the initiating Rust runtime (U10/U11). `link.py` substitutes
one explicit Rust `-lgcc_s` for the caller's archive plus non-unwind `-lgcc` in a
**prototype only**, with hidden and `_Unwind_*`-export variants. It explicitly
appends `-fuse-ld=bfd` or `lld` to correct the initial capture's label ambiguity.
The export pattern currently exposes 18 definitions, not just Aero's twelve;
actual dynsym/owner evidence and the archive/configuration still require review.

On an equipped native x86-64 host, use an independently verified, **nonhidden**
prototype LLVM archive (the harness copies it as `llvm-unwind-probe.a`). No system
libunwind discovery or downloads occur:

```sh
RUSTUP_TOOLCHAIN=1.90.0 PROBE_RUSTC=/home/joey/.cargo/bin/rustc \
  PROBE_CC=/usr/bin/gcc PROBE_LINKER=gnu python3 tests/fixtures/unwind-coexistence/run.py \
  "$VERIFIED_NONHIDDEN_ARCHIVE" /tmp/unwind-roundtrip-FRESH local
```

Repeat with `PROBE_LINKER=lld`. The harness records all variants and deliberately
returns nonzero while hidden negative controls fail; read each case rather than
turn its overall exit into a release pass. These are host-SDK mechanism tests,
not provider/glibc-floor or final cargo-libc qualification. `run.py - FRESH` is a
GCC baseline only; it cannot qualify the candidate. The wrapper needs executable
permission; no source inputs are mutated.

`native_load_controls.c` has only C worker frames. It tests glibc backtrace,
C-only pthread_exit and deferred pthread_cancel with exactly one cleanup and a
checked result. No cancellation crosses Rust or ordinary C FFI boundaries. Compile
it **both without and with `-fexceptions`**: the EH form emits
`__gcc_personality_v0` and exposed failures masked by the plain form.

The parameterized driver reproduces both compile variants with baseline, hidden
and minimal-export controls. Pass archive hashes verified against preparation
records; all archives remain test inputs outside production configuration:

```sh
python3 scripts/probe-native-c-unwind.py --linker bfd --cc /usr/bin/gcc \
  --hidden-archive "$VERIFIED_HIDDEN_ARCHIVE" --hidden-sha256 "$HIDDEN_SHA256" \
  --minimal-archive "$VERIFIED_NONHIDDEN_ARCHIVE" --minimal-sha256 "$NONHIDDEN_SHA256" \
  --evidence /tmp/unwind-native-c-FRESH
```

Run from repository root; repeat LLD when the helper/context design is ready.
This uses whole-archive as a diagnostic control, not a production proposal. It
checks case status/output, disables core dumps, bounds child execution, retains
build commands/maps/ELF imports/exports/versions and loader logs including failures,
and exits nonzero if a required case fails. A source/graph preflight or a plain-C
pass must not waive the EH failures. Do not run these probes as a broad release
matrix while LLVM wiring is paused.
