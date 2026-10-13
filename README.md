![cargo-libc: an orange gear and Cargo-to-ELF diagram](docs/assets/banner.svg)

# cargo-libc

**Build for the libc you choose.**

[![CI](https://github.com/rustcommons/cargo-libc/actions/workflows/ci.yml/badge.svg)](https://github.com/rustcommons/cargo-libc/actions/workflows/ci.yml)
![Rust 1.90+](https://img.shields.io/badge/Rust-1.90%2B-orange)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
![Status: pre-release](https://img.shields.io/badge/status-pre--release-yellow)

A Cargo subcommand that builds Rust programs on a current x86_64 Linux machine for
an older glibc, 2.17 through 2.44. It links against exactly what the release you
choose defines, checks what the result requires, and can run your tests under a
simulation of the target's kernel. Other C libraries are planned.

```sh
cargo libc build --target x86_64-unknown-linux-gnu.2.17 \
  --artifact target/x86_64-unknown-linux-gnu/release/myapp -- --release
cargo libc test --distro rhel-7 --runner local
```

**Read the [user guide](docs/USER-GUIDE.md)** for installation, choosing a target,
every command, C dependencies, the testing tiers, CI use and troubleshooting.

## What it does

- **Links for one glibc release.** The provider for a release has its headers and
  startup objects, plus link-only stubs of glibc's libraries with exactly that
  release's symbols and versions. An interface newer than your target fails at link
  time, on your machine. cargo-libc prepares the provider itself: by default it
  compiles glibc's startup sources, which are built into the tool, with your GCC.
  It verifies every object against the build CI qualified, and caches the
  provider. See
  [Using glibc providers](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/USING-PROVIDERS.md).
- **Checks the result.** `build` and `install` check that the executables they
  produce require no newer `GLIBC_*` version, and `inspect` checks any ELF file.
  With a kernel floor they also run a static scan (tier 1, the least reliable
  tier).
- **Simulates the target kernel.** `test` and `run` run every binary through a
  seccomp filter, or a container, that makes syscalls the target kernel lacks
  return `ENOSYS` (tier 2).
- **Covers the whole Cargo workflow.** It supports `build`, `check`, `clippy`,
  `doc`, `rustc`, `test`, `run` and `install`, and `env` configures plain Cargo
  for CI or other tools.
- **Builds C dependencies for the target.** C built through the `cc` crate or
  CMake gets the target's headers and links against its stubs. Build scripts and
  procedural macros stay native.

It is written in Rust with only the standard library and first-party rustcommons
crates. It uses your `rustc`, a Clang driver and GNU `ld`, and needs no Zig,
Python or downloaded SDK.

## Install

cargo-libc is not on crates.io yet:

```sh
git clone https://github.com/rustcommons/cargo-libc
cargo install --path cargo-libc --locked
```

Requirements are an x86_64 Linux host, Rust 1.90 or later, a Clang that supports
`--gcc-install-dir`, GNU binutils, and GCC. See
[Install](docs/USER-GUIDE.md#install). The five installed executables (`cargo-libc`,
`libc-linker`, `libc-runner`, `libc-cc`, `libc-cxx`) must stay in one directory.

## Status

This is a pre-release. Hosted CI, on every push:
- builds, checks, runs and inspects with the automatic provider for every release
  2.17–2.44, including C dependencies and every command;
- in [linux-abi-tools](https://github.com/rustcommons/linux-abi-tools), qualifies
  container and source providers for every release
  ([PROVIDERS](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/PROVIDERS.md)).

What CI does not do is run your program on a real old system: tier 3 is yours. See
[VALIDATION.md](VALIDATION.md) for what was tested, [Limits](docs/USER-GUIDE.md#limits),
and the [roadmap](docs/ROADMAP.md).

## How the build is controlled

cargo-libc always runs Cargo with an explicit `--target x86_64-unknown-linux-gnu`
and the target's flags in `CARGO_ENCODED_RUSTFLAGS`. Those flags apply to target
code only, so build scripts and procedural macros are built and linked natively.
The target linker is set with `-C linker=libc-linker` in those flags rather than
`CARGO_TARGET_*_LINKER`: on same-triple builds, Cargo can apply that setting to
host artifacts too.

The `libc-linker` helper runs the Clang driver with rustc's arguments.

- **With a provider,** it links with `-nostdlib -no-pie` and supplies the inputs
  explicitly:
  - the provider's `crt1.o` and `crti.o`, and GCC's `crtbegin.o`;
  - rustc's objects and libraries;
  - the provider's stubs (`-L`) and `libgcc_s` stub;
  - the nonshared archives, `crtend.o` and `crtn.o`;
  - `--dynamic-linker=/lib64/ld-linux-x86-64.so.2`.
- **With `--sysroot`,** it adds:

  ```text
  --target=x86_64-unknown-linux-gnu
  --sysroot=<target filesystem>
  --gcc-install-dir=<target GCC runtime directory>
  --rtlib=libgcc
  -fuse-ld=<host linker executable>
  ```

The target flags also do four things:
- select the GCC-style linker interface;
- disable Rust's automatic LLD;
- choose the baseline `x86-64` CPU;
- carry a configuration cfg that changes Cargo's fingerprints when the selection
  changes.

That cfg is not an integrity check, so **changing a provider or sysroot in place
needs a clean build or a new `--target-dir`.**

cargo-libc refuses to run with these variables already set: `RUSTFLAGS`,
`CARGO_ENCODED_RUSTFLAGS`, `RUSTDOCFLAGS` for `test`/`doc`, and the target's `CC`/`CXX`.
It also refuses Cargo arguments that change the target, configuration or message
format. Ambient C include and library search variables are removed from Clang's
environment. That is hygiene, not a sandbox: build scripts and explicit link
arguments can still add files.

## Source map

| File | Role |
| --- | --- |
| `src/cli.rs` | Command parsing; automatic providers through `libc-abi-tools` |
| `src/toolchain.rs` | Input probes, target flags, Cargo execution, the linker and C compiler drivers, artifact checks, `env` |
| `src/cc.rs`, `src/bin/libc-cc.rs`, `src/bin/libc-cxx.rs` | C compiler wrappers for target C dependencies |
| `src/simulate.rs`, `src/bin/libc-runner.rs` | Tier 2 target runner: glibc checks, then the local filter or a container |
| `src/report.rs` | glibc assessment, tier 1 scan, human and JSON reports |
| `src/elf.rs`, `src/cargo_output.rs`, `src/version.rs` | Bounded ELF reader, Cargo JSON messages, versions |
| `src/provider.rs` | Provider descriptor (v1) |
| `examples/` | CI fixtures: `hello` (host build script, proc macro, threads), `c-deps`, `too-new` |

## Credits and licensing

cargo-libc began as the MIT-licensed `cargo-glibc` prototype. Providers come from
`libc-abi-tools` in [linux-abi-tools](https://github.com/rustcommons/linux-abi-tools).
That approach is inspired by the work of **Zig's contributors** on glibc support.

The tool's glibc data comes from glibc itself, through two crates:
- ABI lists from glibc's signed release tags, via
  [glibc-abilists](https://github.com/rustcommons/glibc-abilists);
- startup sources and headers, via
  [glibc-provider-sources](https://github.com/rustcommons/glibc-provider).

Both are LGPL-2.1-or-later and compiled into the `cargo-libc` executable. This
package's source is MIT licensed and contains no glibc files. [NOTICE.md](NOTICE.md)
has the full attribution and what the built tool contains.

Design references:
- [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags);
- [same-triple host behavior](https://doc.rust-lang.org/cargo/reference/unstable.html#target-applies-to-host);
- [JSON messages](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages);
- [rustc codegen options](https://doc.rust-lang.org/rustc/codegen-options/index.html);
- [Clang options](https://clang.llvm.org/docs/ClangCommandLineReference.html);
- [Zig's glibc design](https://codeberg.org/ziglang/zig/src/branch/master/lib/libc/glibc/README.md);
- [ELF symbol versioning](https://refspecs.linuxfoundation.org/LSB_3.1.1/LSB-Core-generic/LSB-Core-generic/symversion.html).
