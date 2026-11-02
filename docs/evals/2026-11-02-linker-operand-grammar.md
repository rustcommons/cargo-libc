# Completed-linker operand audit — 2026-11-02

Follow-up to the [completed compiler-stream correction](2026-11-02-link-output-classification.md)
at `bae662729b7cdee6d1d3a80841a1dc9283dc39d9`. The parent reports its exact
Rust 1.90/x86-64/Clang 18 check passed six tests including 26 native controls;
that historical result does **not** cover this subsequent operand fix.

## Finding and bounded grammar

`--out-implib -shared` names an import-library output file; it does not select a
shared output. The old operand list missed this option. The correction shares
one declared value-option table between classification and both effective named
input discovery passes, removing their duplicated lists. Compiler-driver arity
is still left to Clang's actual completed command. No raw-driver flag list was
expanded, and no caller argument is reordered or rewritten by classification.

The table was audited against GNU binutils 2.40 `ld/lexsup.c`/installed help and
LLVM 18.1.3 `lld/ELF/Options.td`. See the [inventory and hashes](2026-11-02-linker-operand-grammar.json).
Sources: [GNU option rules](https://sourceware.org/binutils/docs/ld/Options.html),
[GNU 2.40 table](https://gnu.googlesource.com/binutils-gdb/+/refs/tags/binutils-2_40/ld/lexsup.c),
[LLD 18.1.3 table](https://github.com/llvm/llvm-project/blob/llvmorg-18.1.3/lld/ELF/Options.td).
This is a bounded supported grammar, not a replacement parser for every linker
version, target, abbreviation, plugin or extension.

- Declared separate-value options consume their next token regardless of spelling.
  Joined `=value` forms do not consume the following token. Optional-value forms
  take values only after `=`. Known ordinary state/group/export/strip/ELF controls
  remain accepted; unknown options report their spelling and unqualified arity.
- Common compiler-emitted `-Lpath`, `-lname` and decimal `-Olevel` are accepted.
  Other undeclared joined short spellings and single-dash long aliases refuse;
  use a declared separate or equals spelling. This avoids interpreting a named
  option as an arbitrary short-option prefix. For example, declared `-mllvm`
  consumes an operand instead of masquerading as joined `-m`.
- Standalone `--` refuses explicitly. No assumed end-marker semantics can let
  subsequent mode-looking names escape input validation. Use `./-name` for
  dash-prefixed input filenames. Scripts/responses and unsupported compiler
  startup overrides retain their existing refusal boundaries.

## Focused evidence and limits

Final native capture: `/tmp/cargo-classify-native-arity-final-source`.
Archive: `/tmp/cargo-linker-operand-grammar-final-20261102.tar.gz`, SHA-256
`89f4211dee3ab8877cf4a499f237ae1c4127b71cebb61c86dbab6072f485d368`.
Exact working-source hashes are in the JSON and saved capture metadata.

Stable Rust 1.90, aarch64 Linux, Clang 16, GCC 12 reference and BFD 2.40:

- Eight pure classification tests plus the explicit native test passed. Every
  declared value option is exercised with `-shared`, `-r`, `-pie`, `--` and an
  unknown-looking operand, with missing-value and equals-form controls. Seven
  optional-value options have next-mode and equals-value controls. Unknown and
  end-marker refusal diagnostics have explicit tests.
- Thirty real Clang/GCC-reference ELF/RUNPATH controls passed, including both
  `-Wl` and `-Xlinker` import-library filenames spelled like output modes.
  Commands run inside the evidence directory, so those filenames cannot create
  files in the checkout. Existing shared/partial startup assertions remain.
- Three **actual rustc system-runtime links** passed: default PIE executable,
  non-PIE executable and cdylib. The wrapper records Clang's completed linker
  stream and then executes the unchanged compiler invocation. Classification
  agrees with real ELF type/PIE flags and leaves the argument vectors unchanged.
  Relocatable completed streams are covered by the native C controls. These
  representative default streams do not certify every external linker option.
- Eight clock/trace, one provider-startup and two C-phase regressions passed;
  formatting, all-target Clippy with warnings denied, diff checks and the local
  release-doc checker passed. No full suite, provider matrix or workflow dispatch.

Run the native test using the [existing reproduction command](2026-11-02-link-output-classification.md#native-x86-64-reproduction)
and a fresh evidence directory. It now includes the rustc links and requires
`rustc` on PATH (or `RUSTC`). Parent x86 rerun of this exact correction is pending.
System unwind remains the default; no LLVM selection, API, runtime compatibility,
publication or release qualification follows from these classifier results.
