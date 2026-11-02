# Completed compiler-stream output classification — 2026-11-02

Independent PR #14 correctness fix while the runtime/default product decision is
pending. LLVM selection, an opt-in flag/default, versions, locks and existing
regression fixtures are unchanged. This is not an unwind compatibility gate or a
release qualification; no full matrix or workflow was dispatched.

## Finding and correction

At `59006f559a2116c86b6b26f87a60187b4c9a270d`, the classifier mixed driver tokens
and forwarded linker arguments. In a link with
`-Wl,-rpath -fPIC -Wl,-shared`, the driver removes `-fPIC`; the linker sees
`-rpath -shared`, so `-shared` is a RUNPATH value, not an output mode. The analogous
`-Xlinker -rpath -fPIC -Xlinker -r` also stays executable, with RUNPATH `-r`.
The old classifier selected Shared/Relocatable incorrectly and ignored PIE controls.

The fix reuses the existing canonical-selected-linker Clang `-###` trace parser.
Only its completed **linker** argv enters `link_output::classify`; C phase checks
no longer decide final kind. No new approximation/list of driver-only flags is
used. The existing linker operand grammar protects mode-looking values; missing,
unexpanded or incompatible partial/shared/PIE controls refuse. Direct driver
precedence belongs to the compiler, while actual linker mode order is retained.

Rust/C helper command assembly now probes kind before choosing provider startup
inputs, then verifies the completed assembled command agrees. Shared providers
keep `crtbeginS.o` and no executable `crt1.o`/loader. Partial links bypass provider
startup/default libraries; the driver receives `-r` so it does not inject them.
For sysroot shared links the driver receives `-shared` to match its CRT choice to
the verified output. Caller linker tokens/libraries/groups remain in place.

Provider executables retain their existing non-PIE contract through a final driver
`-no-pie`, rather than deleting every raw `-pie` token (which could be an operand).
Forwarded PIE still reaches the linker and is diagnosed for unsupported provider
PIE; a shared-to-executable override refuses ambiguous compiler CRT selection.
The actual mode is recorded first, not silently discarded or called shared.

## Focused evidence

[Machine-readable record](2026-11-02-link-output-classification.json) includes exact
working-source hashes, tool versions and native archive digest. The run used
stable Rust 1.90 on aarch64 Linux, Clang 16.0.6, GCC 12.2 and GNU BFD. It does not
establish x86-64 provider or LLVM runtime qualification. GCC compilation is
**reference evidence only**; the product driver remains Clang.

- Five pure linker-stream tests passed, plus provider-startup/operand preservation,
  two C phase tests and eight clock/trace regressions: sixteen focused tests.
- The explicitly invoked native test passed 13 controls under Clang and the same
  13 under GCC reference: actual ELF ET_EXEC/ET_DYN/ET_REL, PIE flag, RUNPATH and
  effective flags are checked. Clang classification is compared with those ELFs;
  aligned shared/partial commands also prove no injected executable CRT, and no
  CRT/default-library injection for partial output. Raw and aligned evidence are
  separately retained before assertions.
- Cases include the reported rpath/driver-only-flag traps, mode-looking PIE values,
  direct versus forwarded PIE/shared precedence, and real relocatable controls.
  Clang direct `-shared -pie` produces a DSO, while GCC produces PIE; this is why
  driver precedence must not be hard-coded into the classifier.
- Forwarded PIE after shared selection is observed as executable PIE. Production
  startup policy refuses its ambiguity; provider PIE also refuses. Ordinary
  supported shared/partial selection retains appropriate startup behavior.
- Formatting, all-target Clippy with `-D warnings`, documentation links and the
  local release-doc checker passed. No full Rust/provider/kernel/consumer matrix.

The native test is explicitly ignored in portable tests because it needs existing
native compilers/readelf and a fresh evidence directory; it **was run explicitly**
for the result above. An earlier attempt under `/tmp/cargo-classify-native-v1`
failed because the GCC-reference fixture used Clang's absolute `-fuse-ld` spelling.
The corrected fixture uses GCC's linker-name spelling and retained that failure;
it was not a product failure or a waived test. The final run is
`/tmp/cargo-classify-native-frozen`. Both directories are saved in
`/tmp/cargo-classification-native-frozen-20261102.tar.gz`, SHA-256
`a0ee8060e2c69527b063835a6a6ce6fec0bec79bee218e1d7d7a085cf0ee8849`.

## Native x86-64 reproduction

Using the parent's existing WSL tools, with a fresh output path and no installs:

```sh
RUSTUP_TOOLCHAIN=1.90.0 CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 \
  CARGO_LIBC_TEST_CLANG=/usr/bin/clang-18 \
  CARGO_LIBC_TEST_GCC=/usr/bin/gcc CARGO_LIBC_TEST_LD=/usr/bin/ld.bfd \
  CARGO_LIBC_LINK_OUTPUT_EVIDENCE=/tmp/classifier-native-FRESH \
  cargo test --lib native_completed_stream_matches_elf_and_runpath --locked --offline \
  -- --ignored --nocapture
```

This checks native compiler-stream/output semantics, not the actual full provider
SDK/runtime integration. The latter and every native unwind/C-EH fixture remain
separate named gates; they must not be inferred from ELF classification success.
Runtime direction still awaits the user; no recommendation is treated as selection.
