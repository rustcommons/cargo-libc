![cargo-libc: an orange gear and Cargo-to-ELF diagram](docs/assets/banner.svg)

# cargo-libc

**Build for the libc you choose.**

[![CI](https://github.com/rustcommons/cargo-libc/actions/workflows/ci.yml/badge.svg)](https://github.com/rustcommons/cargo-libc/actions/workflows/ci.yml)
![Rust 1.90+](https://img.shields.io/badge/Rust-1.90%2B-orange)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
![Status: prototype](https://img.shields.io/badge/status-prototype-yellow)

Cargo integration for selecting a libc build environment and inspecting ELF
requirements. **Initial scope: x86_64 Linux and glibc 2.17 or later.**
Other libc implementations are planned. This is the renamed `cargo-glibc`
prototype; the commands are now `cargo libc` and the helper is `libc-linker`.

The companion [libc-abi-tools](https://github.com/rustcommons/libc-abi-tools)
ports the glibc metadata work of **Zig's contributors** to Rust. That generated
provider now integrates through experimental `--provider` selection for 2.17. See [attribution](NOTICE.md), the
[implementation plan](docs/ROADMAP.md), and the [migration record](docs/MIGRATION.md).

This project replaces the Zig dependency in a deliberately narrow build path:
build an **x86-64 dynamically linked Rust executable** with Cargo/rustc, a host
Clang driver, a host native linker, and development files from an older target
system. Its coordinator, linker helper, Cargo message reader, and ELF inspector
are written in Rust and use only the Rust standard library.

**Status: working prototype, not a complete cargo-zigbuild replacement.** It
consumes a real sysroot. It does not yet generate glibc ABI stub libraries, stage
a distribution SDK, configure native C/C++ dependencies, or prove that an
application runs on SLES. See [VALIDATION.md](VALIDATION.md) for exactly what was
tested and [ARCHITECTURE.md](ARCHITECTURE.md) for the next component.

## What “Rust native” means here

| Responsibility | Implementation in this project |
| --- | --- |
| Parse commands and coordinate Cargo | Rust `cargo-libc` executable |
| Select the target's sysroot and GCC runtime | Rust configuration and input probes |
| Forward rustc's native link command | Rust `libc-linker` executable; no shell |
| Compile Rust application code | Your installed `rustc` |
| Select CRT objects and invoke the native linker | Your installed Clang driver |
| Produce the final ELF executable | Your selected linker; GNU `ld.bfd` was tested |
| Supply old glibc development inputs | Your target sysroot |
| Inspect dynamic version requirements | This project's Rust ELF reader |

There is no Zig, Python runtime, PyPI package, or third-party Cargo dependency in
the tool. Clang and the native linker remain external executables. The project
does not attempt to replace Rust's compiler backend or reimplement glibc.

## Requirements

- x86-64 Linux build host.
- Rust/Cargo 1.90 or later. Validation used Rust 1.99.0; the declared minimum was
  not separately tested. Build mode uses the stable `-C linker-features=-lld`
  option to keep the selected linker in control.
- A host Clang driver supporting `--gcc-install-dir`; validation used Clang
  18.1.3. Pass its executable path with `--cc`.
- A host native linker. The default is `/usr/bin/ld.bfd`.
- For building against an older glibc: a coherent target development sysroot,
  including glibc headers, development libraries, startup objects, loader, and
  the target's GCC runtime development files.

The `inspect` subcommand needs only this tool and the input ELF file; it does
not invoke Clang, the linker, or `readelf`.

## Build and install from this source

From the directory containing this README:

```sh
cargo test --locked --offline
cargo install --path . --locked --offline
cargo libc --help
```

This installs both `cargo-libc` and `libc-linker` into Cargo's bin directory.
They must remain together because the coordinator locates its sibling helper.
The package has `publish = false`; these commands install the local source.
Ensure Cargo's bin directory is on your shell's `PATH`.

To use the project without installation:

```sh
cargo build --locked --offline --release
./target/release/cargo-libc --help
```

## Use for Rocky Linux 9 → SLES 12 SP5

These commands assume that the target is x86-64 and that you have already
assembled a SLES 12 SP5 development sysroot. **The 2.22 value is a ceiling to
check, not a command that transforms Rocky's glibc into glibc 2.22.**

Keep the sysroot in a separate, versioned directory. Runtime libraries alone
are insufficient. The input probes expect:

- `usr/include/features.h` with a glibc version no newer than the ceiling;
- `Scrt1.o`, `crt1.o`, `crti.o`, `crtn.o`;
- `libc.so`, `libc_nonshared.a`;
- `crtbeginS.o`, `crtendS.o`, and `libgcc_s.so` selected from the target runtime
  installation;
- the target loader at `lib64/ld-linux-x86-64.so.2` inside the sysroot.

Other libraries needed by rustc or the application must also be present.
`libc.so` may be a development linker script; it is not the same file as the
runtime `libc.so.6`. An extracted SDK must have working internal symlinks.
Probed symlinks that resolve outside the sysroot are rejected.

Use the **actual directory containing the target's `crtbeginS.o`** for
`--gcc-runtime`; do not copy the placeholder below literally. No executable
from that old GCC installation is launched. The old runtime files are inputs
to the current build tools.

### 1. Check the target inputs

```sh
cargo libc doctor \
  --sysroot /opt/sysroots/sles12sp5 \
  --gcc-runtime /opt/sysroots/sles12sp5/ACTUAL-GCC-RUNTIME-DIRECTORY \
  --glibc 2.22 \
  --cc /usr/bin/clang \
  --ld /usr/bin/ld.bfd
```

The output lists the chosen compiler, linker, header version, and resolved
startup/runtime inputs. A successful probe checks those specific inputs;
it is not a recursive validation of every library or linker script.

### 2. Build the application

Run this in your application's Cargo project, replacing `myapp` with its
executable name:

```sh
cargo libc build \
  --sysroot /opt/sysroots/sles12sp5 \
  --gcc-runtime /opt/sysroots/sles12sp5/ACTUAL-GCC-RUNTIME-DIRECTORY \
  --glibc 2.22 \
  --cc /usr/bin/clang \
  --ld /usr/bin/ld.bfd \
  --artifact target/sles12sp5/x86_64-unknown-linux-gnu/release/myapp \
  -- --release --locked --bin myapp --target-dir target/sles12sp5
```

Arguments before `--` configure this tool. Arguments after `--` go to
`cargo build`. Cargo is always given the explicit target
`x86_64-unknown-linux-gnu`, even on an x86-64 GNU/Linux host.

`--artifact` selects one executable to inspect. Its canonical path must match
an executable in Cargo's `compiler-artifact` messages from this invocation;
cached outputs count. An unrelated existing file is rejected. This prototype
checks one selected executable, rather than automatically checking every
workspace artifact.

`--dry-run` performs the input probes and prints the Cargo command without
building. It does not check the selected artifact.

### 3. Inspect a binary independently

```sh
cargo libc inspect \
  target/sles12sp5/x86_64-unknown-linux-gnu/release/myapp \
  --max-glibc 2.22
```

For machine-readable inspection:

```sh
cargo libc inspect ./myapp --max-glibc 2.22 --json
```

The report includes the interpreter, `DT_NEEDED`, RPATH/RUNPATH, and version
requirements grouped by their referenced library. It reads the ELF program
headers and dynamic metadata directly, so it also works when section headers
have been removed.

A successful check means **the file's recorded mandatory `GLIBC_*` version
requirements do not exceed the selected ceiling**. It does not mean that the
whole application is SLES compatible. Inspect its deployed shared libraries
as well, and test the application on the intended SLES environment.

`GLIBCXX_*`, `CXXABI_*`, and `GCC_*` requirements are displayed but are not
covered by the glibc ceiling. Version-level weak requirements are reported
separately. That flag is distinct from a weak individual symbol. Required
nonnumeric tags such as `GLIBC_PRIVATE` or `GLIBC_ABI_DT_RELR` fail the numeric
ceiling check because a numeric comparison cannot establish their support.

## How the build is controlled

The Rust helper invokes the configured Clang driver with rustc's arguments and
then appends these profile selections:

```text
--target=x86_64-unknown-linux-gnu
--sysroot=<target filesystem>
--gcc-install-dir=<target GCC runtime directory>
--rtlib=libgcc
-fuse-ld=<host linker executable>
```

Clang's `--gcc-install-dir` selects GCC runtime inputs; it does not select the
linker executable. That is why the two paths are independent. GNU `ld.bfd`
running on Rocky can consume development libraries from SLES.

The helper path is supplied through `-C linker=...` in
`CARGO_ENCODED_RUSTFLAGS`, with an explicit Cargo `--target`. The target linker
is intentionally not set with `CARGO_TARGET_*_LINKER`: Cargo's historical
same-triple behavior can apply that configuration to host build scripts and
procedural macros. The included example exercises both kinds of host artifact.

Additional target flags select the GCC-style linker interface, disable Rust's
automatic LLD selection, and choose the baseline `x86-64` CPU. Profile paths
and a stable configuration discriminator appear in rustc flags so path/config
changes affect Cargo's fingerprints. This discriminator is not a cryptographic
integrity check. **Changing tool or SDK contents in place requires a clean
build or a new `--target-dir`.**

The tool refuses preexisting nonempty `RUSTFLAGS` and
`CARGO_ENCODED_RUSTFLAGS`. Its managed encoded flags take precedence over Rust
flags in Cargo configuration files; those flags are not merged. Cargo arguments
that override the target, configuration, or JSON message format are rejected.

Ambient C/C++ include and library search overrides are removed from the Clang
invocation. This is useful hygiene, not a sandbox: build scripts and explicit
native link arguments can still introduce other files.

## Current limits

- Build coordination is for x86-64 Linux dynamically linked executable targets.
  `inspect` accepts x86-64 ELF64 little-endian executable/shared objects, up to
  256 MiB, within the documented parser limits in `src/elf.rs`.
- Native C/C++ build scripts, `cc`, CMake, and `pkg-config` are not automatically
  configured. Applications with such dependencies require additional target
  configuration and dependency inspection.
- The tool does not resolve the transitive shared-library closure, validate
  C++ runtime versions, inspect CPU instructions/kernel use, or discover
  libraries loaded later with `dlopen`.
- Build verification requires `/lib64/ld-linux-x86-64.so.2` as the interpreter
  and rejects a direct RPATH/RUNPATH entry into the configured build sysroot.
  It does not model the complete runtime search behavior.
- There is no automatic ABI database, stub generator, SDK download, container
  management, cross-architecture support, or target runtime test runner.
- `build` consumes Cargo's JSON messages internally and prints a human report.
  Use `inspect --json` when you need JSON. It is not a transparent replacement
  for Cargo's stdout protocol.

## Exit codes

For `inspect`, 0 means parsing and any requested direct check succeeded, 2
means the requested direct requirement check failed, and 1 means an input or
format error. For `build`, a failed Cargo invocation preserves Cargo's numeric
exit code; configuration/inspection errors return 1, and a failed post-build
requirement/interpreter/path check returns 2. Signal-only termination maps to 1.

## Source map and references

- `src/cli.rs`: command parsing.
- `src/toolchain.rs`: target input probes, target flags, Cargo execution, linker
  driver invocation, and selected-artifact checks.
- `src/cargo_output.rs`: bounded JSON parsing for Cargo artifact identity.
- `src/elf.rs`: bounded ELF dynamic metadata reader.
- `src/report.rs`: numeric glibc assessment and human/JSON reports.
- `src/version.rs`: numeric version parsing and ordering.
- `examples/hello`: Rust executable with a host build script and procedural
  macro, plus a thread at runtime.

Primary documentation used for the design:

- [Cargo configuration and Rust flags](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)
- [Cargo's same-triple host linker behavior](https://doc.rust-lang.org/cargo/reference/unstable.html#target-applies-to-host)
- [Cargo JSON artifact messages](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages)
- [rustc code generation/linker options](https://doc.rust-lang.org/rustc/codegen-options/index.html)
- [Clang command-line options](https://clang.llvm.org/docs/ClangCommandLineReference.html)
- [Zig's glibc support design](https://codeberg.org/ziglang/zig/src/branch/master/lib/libc/glibc/README.md)
- [LSB ELF symbol versioning](https://refspecs.linuxfoundation.org/LSB_3.1.1/LSB-Core-generic/LSB-Core-generic/symversion.html)

The source in this package is MIT licensed. No glibc implementation, headers,
ABI database, or SLES runtime files are included.
