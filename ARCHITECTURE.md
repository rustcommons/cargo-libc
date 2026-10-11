# Architecture and the path to generated glibc ABI support

The original coordinator is now named `cargo-libc`. Its companion
[libc-abi-tools](https://github.com/rustcommons/linux-abi-tools) has begun the
metadata collection/parsing phase; the local 2.17 provider now integrates via
--provider. Authenticated SDKs and additional versions
remain future work. See [ROADMAP.md](docs/ROADMAP.md).

## The decision

Use a real target sysroot for version 0.1. The first deliverable removes Zig
from one practical build path without also implementing an assembler, linker,
C standard library, SDK distributor, and multi-architecture ABI generator.

This is a deliberately inspectable starting point. `cargo-zigbuild` itself is
already a Rust application; the capability to replace is the underlying Zig
toolchain's glibc targeting, not the language used for its Cargo wrapper.

## Components that are implemented

### Coordinator

`cargo-libc` parses a target profile and resolves paths before starting Cargo.
It rejects an obviously newer glibc header set, checks the startup and runtime
inputs selected by Clang, then supplies target-only linker flags to rustc.
It accepts an explicit executable selection and matches it against Cargo's
reported executable artifacts, including fresh/cache-hit artifacts.

The tool prints its selected files and command. It has no automatic network
access, package install step, or configuration file modification.

### Native linker helper

`libc-linker` is a second Rust binary. Cargo/rustc invokes it as a compiler
driver. It forwards arguments without passing them through a shell and invokes
the chosen Clang driver with the target profile selections.

Clang knows the startup object order, compiler runtime conventions, and native
linker invocation details. The actual linker can run on a newer build host;
the target glibc version comes from its inputs. The tested backend is GNU
`ld.bfd`.

### Artifact and requirement readers

The Cargo message reader extracts top-level executable identities from bounded
JSON without mistaking a similarly named nested field for an artifact.
The ELF reader parses program headers and dynamic metadata rather than
scraping `readelf` text or relying on section headers. It reads dependencies,
interpreter/search paths, and version requirements. The assessment module
compares numeric glibc requirements and makes no claim about unexamined runtime
dependencies.

`inspect --distro`/`--kernel` adds the tier 1 scan from rustcommons
`linux-abi-scan`. cargo-libc resolves the floors with `linux-targets`
(`Selection::resolve`, the same rules as the linux-abi-tools CLIs), passes the
bytes it already read to `linux_abi_scan::scan`, and renders the report inside
its own text and JSON output. The scan reads section headers, dynamic
symbols, and code; its kernel facts come from `kernel-abi-tools`, so it
cannot disagree with the seccomp simulation about what a kernel provides.

### Tier 2 runner

`cargo libc test` and `run` start Cargo with the same managed flags as
`build` (and, for `test`, the same flags for rustdoc), plus
`--config target.x86_64-unknown-linux-gnu.runner=["<libc-runner>"]`. The
selection reaches the helper in `CARGO_LIBC_RUN_*` variables. `libc-runner`
applies the direct glibc check to each binary and then calls
`kernel_abi_tools::local::run` (seccomp filter installed in the child before
`exec`; no root) or `kernel_abi_tools::container::run` (OCI profile), choosing
between them with `Runner::choose`, the same rules as `kernel-seccomp run`.
Host build scripts and procedural macros are not target binaries, so Cargo
never routes them through the runner.

## What a Zig-style ABI provider would add

The next provider would supply a small, generated link environment instead of
requiring the distribution's complete development sysroot. Its output would
plug into the coordinator/linker path above. It is **not implemented in 0.1**.

### 1. A versioned ABI database

For the first supported target, collect and pin at least:

- symbol names and the libraries exporting them;
- symbol versions available in each supported glibc release;
- default version choices and aliases;
- function, data-object, and TLS kinds;
- data sizes and other target-specific ABI details needed by the linker;
- library identities/SONAMEs and changes in library membership across releases.

Use upstream ABI lists and/or systematically collected metadata from trusted
reference builds. Preserve data provenance and applicable licenses. A text list
of function names and a single “introduced in” field is not enough.

Zig's libc-abi-tools repository inspired this approach and is a useful
format/design reference, but its format is not a stable protocol this project
consumes, and no data is taken from it. The ABI lists come directly from
glibc's own repository at its signed release tags, collected with
per-release and per-file provenance by
[glibc-abilists](https://github.com/rustcommons/glibc-abilists)
(LGPL-2.1-or-later). `libc-abi-tools` normalizes them into its own database;
it depends on glibc-abilists by default (`bundled-glibc-data` feature) and can
read a data directory instead.

### 2. Rust-generated link-only shared objects

Given a target and glibc ceiling, emit stub ELF shared objects that advertise
the allowed ABI. Two implementation routes are available:

| Route | Work and tradeoff |
| --- | --- |
| Emit assembly and version scripts, then invoke a host assembler/linker | Smaller first implementation; native tools handle ELF layout and hashing |
| Write ELF shared objects directly in Rust | More control and fewer subprocesses; must correctly emit sections/segments, dynamic symbols, hashes, versions, and target-specific details |

Either route can be driven entirely by Rust. The generated shared objects
would contain link-time ABI information, not glibc's implementations. They
must never be installed as application runtime libc libraries. On SLES, the
real destination libraries must provide the functions referenced by the final
executable.

Changing version strings on a completed executable is not an alternative.
Versioned names select ABI contracts; a rename cannot supply missing functions
or make a different data layout compatible.

### 3. Real startup/nonshared objects and compiler runtimes

Stub shared objects are only part of the environment. Startup files such as
`Scrt1.o`, `crti.o`, and `crtn.o`, plus `libc_nonshared.a`, contain code that can
be copied into the final executable. They need compatible implementations,
obtained from a target SDK or built from carefully selected upstream sources.
These glibc components include C and assembly; a Rust coordinator does not
make that implementation code disappear.

GCC runtime inputs such as `crtbeginS.o`, `crtendS.o`, and unwinding/compiler
builtin libraries are another separate responsibility. We can initially retain
known compatible runtime inputs instead of claiming that a libc stub generator
also replaces them.

### 4. Version-aware native headers

Pure Rust often consumes precompiled Rust libraries and calls C symbols via
FFI. A project with native C/C++ compilation also needs matching headers and
build settings. Headers may redirect functions, encode structure layouts, or
select APIs according to feature macros. Link-only stubs cannot correct a C
object already compiled against the wrong layout or declaration.

Add explicit adapters for `cc`, CMake, and `pkg-config` only with separate host
and target configurations. The compiler used for host build tools must remain
able to run on the build machine. A global `CC` or library-path override is
not a sufficient design for this distinction.

### 5. Runtime validation and dependency closure

Before calling a generated provider supported, compare it against known
reference glibc exports, link a representative test matrix, and run programs
on the oldest supported target. Include threads/TLS, unwinding, C FFI, common
native dependencies, startup behavior, library membership/version changes,
and negative tests for APIs newer than the selected ceiling.

Resolve and inspect deployed shared libraries as a separate operation. A main
executable can have old direct glibc requirements while a dependency still
requires a newer libc. Plugins and libraries discovered dynamically require
application-specific runtime coverage as well.

## Why a Rust-written linker alone is not sufficient

The linker consumes the ABI and runtime inputs; it does not create the complete
old-glibc environment from a version number. A Rust linker could be evaluated
as a future backend, but replacing GNU ld does not replace any of the ABI-data,
headers, startup, or runtime responsibilities above. Keep these components
separate so one can change without rewriting the others.

## Incremental delivery gates

1. **Current:** sysroot-backed coordinator, native helper, Cargo artifact
   matching, and direct ELF requirement inspection; host validation only.
2. **Target qualification:** exercise this path with the actual SLES 12 SP5
   SDK and run the built application on SLES. Add native dependency adapters
   needed by the application.
3. **Generated provider:** pin one architecture/version range, emit correct
   stub libraries, and supply matching startup/nonshared/runtime files.
4. **Broader support:** additional targets and glibc releases, dependency
   closure inspection, packaging, and a runtime test matrix.

The generated provider is a realistic Rust project, but it is toolchain work
with an ongoing ABI maintenance burden. This package provides working parts
that remain useful while that provider is developed.

## Primary references

- [cargo-zigbuild's Rust package](https://github.com/rust-cross/cargo-zigbuild/blob/main/Cargo.toml)
- [Zig glibc stubs, headers, and real CRT/nonshared components](https://codeberg.org/ziglang/zig/src/branch/master/lib/libc/glibc/README.md)
- [libc ABI collection tooling and dataset format](https://codeberg.org/ziglang/libc-abi-tools)
- [Cargo Rust flag precedence](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)
- [Cargo host/target linker behavior](https://doc.rust-lang.org/cargo/reference/unstable.html#target-applies-to-host)
- [Clang driver options](https://clang.llvm.org/docs/ClangCommandLineReference.html)
- [GNU linker documentation](https://sourceware.org/binutils/docs/ld/)
