# Compare a target libgcc runtime

**Unreleased implementation.** This option is not in cargo-libc 0.4.0. It needs
the companion `libc-abi-tools::content_sha256` API to be released and adopted
from crates.io before registry-only release qualification. No version, lockfile,
runtime selection or link policy is changed by this candidate.

```sh
cargo libc inspect ./app --target-libgcc /copied-target/libgcc_s.so.1
cargo libc inspect ./app --target-libgcc /copied-target/libgcc_s.so.1 \
  --max-glibc 2.22 --json
```

Copy the actual matching-architecture runtime from the deployment environment,
or obtain it from that environment's packages/SDK. The file path is explicit;
the checker never substitutes the build host's runtime. An unversioned linker
script such as `libgcc_s.so` is not the runtime ELF: supply `libgcc_s.so.1` itself.
The option is inspect-only, requires a nonempty path and cannot be repeated.
Native paths, including non-UTF-8 Unix paths, are supported. `--` still ends
option parsing. No new build, run, test or environment option is introduced.

## What is compared

The checker reads two files, without loading or executing either one. Both are
limited to supported x86-64 ELF64 little-endian Linux/System V metadata. The
runtime must be ET_DYN with exact SONAME `libgcc_s.so.1`. It follows an explicitly
supplied symlink to a regular referent, but never follows dependency names,
linker scripts or arbitrary search paths. It performs no compiler/package probe,
network lookup, `ldd`, `dlopen`, IFUNC execution or runtime test.

The mandatory version nodes recorded with exact owner `libgcc_s.so.1` are checked
independently of dynamic symbols. In a versioned target, a missing mandatory node fails even when only
weak symbols, or no symbols, reference it. The symbol comparison then checks
each owner-attributed function import against an externally visible, hash-table
reachable definition with the exact name and version. Higher-numbered versions
and parent nodes do not substitute for an exact pair. An explicit old version
can match a nondefault export; GNU versym's nondefault bit is distinct from ELF
hidden visibility.

Symbol binding and version-node weakness remain separate. A missing WEAK symbol
can be optional; a weak version-node flag cannot turn a GLOBAL symbol optional.
FUNC/IFUNC definitions establish function-interface availability only, without
calling a resolver or checking a function signature. Object/TLS, canonical PLT,
copy/unsupported representations and ambiguous definitions are unresolved,
rather than silently omitted. Versionless fallback behavior is not modeled.
A wholly unversioned target (no DT_VERDEF) with an otherwise potentially
matching unversioned function is therefore unresolved, not declared incompatible
solely because its version nodes are absent. Other independently proven failures
retain precedence.

Ownership comes from recorded version indices and `vn_file`, not `_Unwind_*`
spelling or `GCC_*` labels. Other recorded owners are listed as outside scope.
Unversioned imports have no recorded library owner: strong unowned imports keep
the result unresolved, while weak unowned hooks remain explicitly excluded.
No owner-attributed requirements means unresolved, not a vacuous match.

The supplied runtime's own NEEDED, RPATH/RUNPATH and version requirements are
reported separately. `--max-glibc` or `--distro` applies the existing direct
glibc assessment to both the application and this supplied runtime. The optional
kernel/heuristic scan and `--strict`/`--verbose` still apply only to the primary
artifact. They do not change GCC matching rules.

## Results and exit statuses

| Result | Meaning |
| --- | --- |
| `match` | All enumerated mandatory owner-attributed nodes/pairs are present in this supplied file; optional absences and exclusions remain visible. |
| `missing` | Validated relevant metadata proves a missing mandatory interface, or the supplied ELF's architecture, SONAME or type is incompatible. |
| `unresolved` | Inputs, attribution, representation or bounded enumeration cannot establish the comparison. No incomplete table is treated as empty. |

| Exit | Meaning with this option |
| --- | --- |
| 0 | GCC comparison matches and independent requested checks pass. |
| 2 | Known GCC mismatch, primary-artifact glibc/scan failure, or supplied-runtime direct glibc failure. |
| 3 | GCC comparison unresolved, without an independent failing check. |
| 1 | CLI/primary artifact error, or an actual reporting I/O error under existing output rules. |

For example, all 12 GCC pairs can match while the supplied runtime requires
GLIBC_2.34 above `--max-glibc 2.22`. GCC status stays `match`; the separately
labeled runtime glibc assessment fails and the command returns 2. A runtime's
own glibc failure can likewise return 2 while an optional symbol parser is
unresolved. Missing/unreadable/malformed target files produce a structured
unresolved result, preserving successful primary ELF inspection. A separately
validated missing mandatory version node in a versioned target still returns 2
when optional symbol enumeration fails; unknown symbol inventory is not reported
as absence. Invalid version hashes/index records cannot establish node absence.

Without `--target-libgcc`, existing inspect output and exits are preserved and
no optional exact-symbol parsing is required.

## JSON and file identity

With `--json`, existing root fields remain and `gcc_abi_comparison` is added only
when requested. Schema version 1 includes:

- `status`, `scope`, separate `version_enumeration_complete` and symbol
  `enumeration_complete` flags, and stable per-row `reason_code`s;
- independent `version_requirements` and `requirements` inventories, required
  flags, binding/type/version evidence and exact matching definitions;
- `other_owner_imports`, `unowned_imports`, and mandatory/optional node/pair counts;
- `artifact_identity` and `target_identity`: conventional full SHA-256 and byte
  length of the exact inspected buffers, supplied/resolved display paths, lossy
  display flag and raw ELF identity fields (explicitly little-endian raw numbers);
- `target_runtime_metadata`: the separate direct inspection of the runtime,
  including its own direct glibc assessment;
- bounded findings for unavailable/malformed/unsupported metadata and limits.

An unreadable/incomplete file has no fabricated digest. Display paths are not
identity. A digest identifies inspected bytes; it does not authenticate an
implementation or promise that a later loader reads the same bytes. Concurrent
hostile filesystem replacement and loader resolution are not protected against.

## Bounds and limits

Primary reads retain the 256 MiB limit; explicit runtime reads are limited to
64 MiB. Each optional parser permits at most 262,144 dynamic symbols, 65,536
metadata records, 1,024 program headers and 16 MiB of accumulated strings. The
comparison accepts at most 32,768 combined symbol entries to bound report size;
larger inventories are unresolved, never truncated, with separately validated
node failures still reported. Dynamic relocation checks have a 262,144-entry
budget across REL/RELA/PLT tables. Both bounded input buffers are currently retained
during comparison; these are file-read limits, not a whole-process memory limit.
Hashing, metadata and report construction need additional working memory. SysV/GNU hash evidence must
provide a complete consistent symbol count. Section headers are not required;
unsupported or contradictory metadata is unresolved. No count is guessed from
EOF. SHA-256 uses the companion crate's existing std-only implementation.

**A match is not a safe-to-run certificate.** It does not verify the target
loader's selected file, transitive dependencies, plugins, CPU/kernel behavior,
runtime implementation authenticity or unwinder interoperability. A link-only
interface stub can expose matching interfaces without implementing a runtime;
this structural check does not distinguish it from a genuine implementation.
The known mixed GCC/LLVM cleanup and cross-DSO issues remain outside this check.

For GCC release/package/SONAME versus symbol-namespace distinctions, see the
[canonical runtime history](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/GCC-RUNTIME.md).
The node check and symbol lookup are separate in glibc's primary implementation:
[version checks](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-version.c)
and [symbol lookup](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-lookup.c).
This checker intentionally does not emulate every loader fallback.

## Maintainer validation

`scripts/check-target-libgcc.py BIN EVIDENCE_DIR` links small GNU fixtures (plus LLD when available) and
inspects them; it never executes their payloads. Unit tests cover bounded parsing,
version-index joins, weak/node distinctions and conservative matching. The Linux
CI job runs the static fixtures without installing another toolchain.

Until the hash API is published, use the source-only qualifier with clean,
committed Cargo and producer checkouts:

```sh
python3 scripts/qualify-target-libgcc.py CARGO_CHECKOUT PRODUCER_CHECKOUT EVIDENCE_DIR
```

It snapshots both exact Git heads, patches only a disposable consumer copy,
records source identities and runs Rust 1.90 tests/docs, CI-pinned 1.99 formatting/Clippy plus native static
fixtures. Its patch is development evidence, not registry release qualification.
Final adoption must use the real published dependency minimum/checksum and pass
unpatched package/install checks before a release. No deployment qualification
or new platform support follows from these tests.
