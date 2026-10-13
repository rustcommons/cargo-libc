# cargo-libc user guide

cargo-libc builds Rust programs on a current x86_64 Linux machine that run on older
ones. You choose the oldest glibc your users have, and cargo-libc links against
exactly what that glibc release defines. It then checks the result, and can run
your tests under a simulation of the target's kernel.

- [Install](#install)
- [Quick start](#quick-start)
- [Choosing a target](#choosing-a-target)
- [Providers](#providers)
- [Commands](#commands)
- [C dependencies](#c-dependencies)
- [Testing tiers](#testing-tiers)
- [Plain Cargo and CI](#plain-cargo-and-ci)
- [Troubleshooting](#troubleshooting)
- [Limits](#limits)
- [Reference](#reference)

## Install

You need:

- **An x86_64 Linux build machine.**
- **Rust 1.90 or later.** CI uses 1.99.
- **Clang that supports `--gcc-install-dir`**, as the link driver. The default is
  `/usr/bin/clang`; choose another with `--cc`. Clang 18, 21 and 24 were tested.
- **GNU binutils**, with `ld.bfd` as the linker. Choose another linker with `--ld`.
- **GCC**, which builds the glibc provider and supplies `crtbegin.o`/`crtend.o`.
  GCC 11 through 16 are qualified.
- **CMake**, only if a dependency builds C with it.
- **podman or docker**, only for container-mode providers or the container
  test runner.

cargo-libc is not on crates.io yet. Install it from a checkout:

```sh
git clone https://github.com/rustcommons/cargo-libc
cargo install --path cargo-libc --locked
cargo libc --help
```

This installs `cargo-libc` and its helpers `libc-linker`, `libc-runner`, `libc-cc`
and `libc-cxx`. Keep them together in one directory, because cargo-libc finds its
helpers next to itself.

## Quick start

Build your project for glibc 2.17, the oldest supported release. It covers RHEL and
CentOS 7, and anything newer:

```sh
cargo libc build --target x86_64-unknown-linux-gnu.2.17 \
  --artifact target/x86_64-unknown-linux-gnu/release/myapp -- --release
```

On first use for a release this prints
`glibc 2.17 provider (source mode, prepared)`: cargo-libc compiled and verified the
glibc 2.17 provider, which takes a few seconds and is cached. Then it builds and
prints a report for the executable named by `--artifact`. A passing report says
**the executable requires no `GLIBC_*` version newer than 2.17**.

Arguments before `--` are cargo-libc's; everything after `--` goes to Cargo. Cargo
always builds for the ordinary `x86_64-unknown-linux-gnu` target, so outputs are
under `target/x86_64-unknown-linux-gnu/`.

Check any binary at any time:

```sh
cargo libc inspect target/x86_64-unknown-linux-gnu/release/myapp --max-glibc 2.17
```

Run the tests on your machine, with the kernel limited to what RHEL 7 has:

```sh
cargo libc test --distro rhel-7 --runner local
```

## Choosing a target

Name the release with a suffix, `--target x86_64-unknown-linux-gnu.2.28`, or with
`--glibc 2.28`. Releases 2.17 through 2.44 are supported. Choose the **oldest glibc
your users run**: a program linked for 2.17 runs on every later glibc, never the
other way round.

Or name a distribution. `--distro ID` sets the glibc release and the kernel floor
together:

| `--distro` | glibc | Kernel |
| --- | --- | --- |
| `rhel-7`, `centos-7` | 2.17 | 3.10 |
| `rhel-8` | 2.28 | 4.18 |
| `rhel-9` | 2.34 | 5.14 |
| `sles-12-sp5` | 2.22 | 4.12 |
| `sles-15-sp1` | 2.26 | 4.12 |

[linux-targets](https://github.com/rustcommons/linux-abi-tools/blob/main/crates/linux-targets/data/distros.tsv)
lists all the presets, including Debian, Ubuntu and Amazon Linux. `--distro` cannot
be combined with `--glibc`, a versioned `--target`, or `--kernel`.

Things to know:

- **2.34 is a step.** From 2.34, glibc's `crt1.o` references
  `__libc_start_main@GLIBC_2.34`, so every executable built for 2.34 or later needs
  glibc 2.34. If any of your users are below 2.34, target their release.
- **Common functions get new versions.** For example, `expf`/`powf` (2.27) and
  `exp`/`log`/`pow` (2.29) have newer defaults, so an older target gets the older
  versions automatically.
- **A program cannot use what its release lacks.** Calling, say, `memfd_create`
  (2.27) while targeting 2.17 fails at link time. That is the point: the error
  appears on your machine, not your users'.
- **Distributions patch glibc.** cargo-libc targets upstream GNU releases.
  [DISTRIBUTION-DIFFERENCES](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/DISTRIBUTION-DIFFERENCES.md)
  measures how CentOS 7's 2.17 differs.

## Providers

The **provider** for a release supplies everything needed to link for it:
- that release's headers and startup objects;
- link-only stubs of glibc's libraries;
- a `libgcc_s` stub.

cargo-libc prepares it the first time you select the release and then reuses it.

| Mode | How | When |
| --- | --- | --- |
| `source` (default) | Compiles glibc's startup sources, which are built into cargo-libc, with your GCC, then verifies every object against the qualified build. Offline, seconds. | Almost always. |
| `container` | Runs the same pinned container build CI qualifies (podman or docker; network; minutes). | When source mode refuses your GCC. |
| `prebuilt` | — | Not available yet (legal review). |

Choose a mode with `--provider-mode container` or `CARGO_LIBC_PROVIDER_MODE`, and
the GCC with `CARGO_LIBC_PROVIDER_GCC`. A refused provider is never replaced by
another mode behind your back.

Manage the cache (`$CARGO_LIBC_PROVIDER_DIR`, default `~/.cargo/libc-providers`) with
`cargo libc provider prepare|list|path|verify|export|import|remove`. The full
guide covers when to choose each mode and how to use providers on offline and
air-gapped machines:
[Using glibc providers](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/USING-PROVIDERS.md).

**Your own sysroot or provider.** `--provider FILE` uses a provider descriptor you
prepared ([PROVIDER-CONTRACT](PROVIDER-CONTRACT.md)). `--sysroot DIR` with
`--gcc-runtime DIR` and `--glibc` uses a complete development sysroot from a target
system instead: its headers, startup files, `libc.so`, `libc_nonshared.a`, loader
and GCC runtime. With a sysroot, the selected release is a ceiling that the build is
checked against, not something cargo-libc enforces at link time. Prefer providers.

## Commands

Every command below takes the target options: a versioned `--target` or `--glibc`,
or `--distro`, plus optionally `--provider-mode`, `--provider`, `--sysroot`, `--cc`
and `--ld`. Arguments after `--` go to the Cargo command unchanged. A few
arguments are refused because cargo-libc controls them: `--target`, `--config`,
`--message-format`, and `+toolchain`.

### build

```sh
cargo libc build --target x86_64-unknown-linux-gnu.2.28 --kernel 4.18 \
  --artifact target/x86_64-unknown-linux-gnu/release/myapp -- --release --bin myapp
```

`--artifact` names the executable to check. It must be one this Cargo invocation
built or found fresh, so a stale file cannot pass. The check covers:
- the executable's `GLIBC_*` requirements;
- its loader;
- that no runtime search path points into the build sysroot.

`--distro` or `--kernel` adds the tier 1 scan. `--strict` makes scan warnings fail.
`--dry-run` prints the Cargo command without building.

### check, clippy, doc, rustc

```sh
cargo libc check  --target x86_64-unknown-linux-gnu.2.17 -- --workspace --all-targets
cargo libc clippy --target x86_64-unknown-linux-gnu.2.17 -- --all-targets -- -D warnings
cargo libc doc    --target x86_64-unknown-linux-gnu.2.17 -- --no-deps
cargo libc rustc  --target x86_64-unknown-linux-gnu.2.17 -- --bin myapp -- -C opt-level=1
```

These commands run the Cargo command with the same target settings as `build`, so
Cargo's cache is shared with later builds. `clippy` and `rustc` pass arguments after
their own second `--` to clippy or rustc. Arguments there that name a target,
linker, link arguments or relocation model are refused. Nothing is checked:
`check`, `clippy` and `doc` do not link, and for `rustc` you run `inspect` on what
it built.

### test, run

```sh
cargo libc test --distro rhel-8 --runner local -- --release
cargo libc run  --target x86_64-unknown-linux-gnu.2.17 --kernel 3.10 -- --bin myapp
```

Cargo runs every target binary through `libc-runner`: unit and integration tests,
doctests, and `cargo run`'s program. For each binary it:

1. refuses it if it needs a newer glibc than the target;
2. on your machine's own glibc (no runner, or `--runner local`), refuses it if your
   machine's libraries lack a version it needs. This happens when the target is
   newer than your machine, for example a 2.41 target on a 2.39 host. Use an older
   target, or `--runner container` with a newer image;
3. runs it under the kernel simulation, when `--distro` or `--kernel` is given.

| Runner | Chosen by | Userland | Needs |
| --- | --- | --- | --- |
| `local` | `--runner local`, or `--kernel` without an image | Your machine's | Linux; no root |
| `container` | `--runner container`, `--image IMAGE`, or a `--distro` preset | The image's | podman or docker, with crun 1.28 or later (`KERNEL_ABI_ENGINE_ARGS="--runtime /path/to/crun"`) |

`cargo libc doctor --distro ID` (or `--kernel V`) checks both runners by running
something under them. Add the target options to also check the build inputs.

### install

```sh
cargo libc install --target x86_64-unknown-linux-gnu.2.17 --kernel 3.10 -- ripgrep --locked
```

`cargo install` with the target settings. Every executable it built gets the
`build` check, and the scan with `--distro`/`--kernel`. Cargo installs before the
check runs, so a failing executable is already installed: the command exits with
2, and you should remove it.

### inspect

```sh
cargo libc inspect ./myapp --max-glibc 2.22
cargo libc inspect ./myapp --distro sles-12-sp5 --strict --json
```

`inspect` reads any x86_64 ELF file directly, without running tools. It reports:
- the interpreter;
- `DT_NEEDED`, RPATH and RUNPATH;
- version requirements per library;
- whether any `GLIBC_*` requirement exceeds `--max-glibc`.

`--distro`/`--kernel`/`--scan` add the tier 1 scan, and `--json` produces
machine-readable output.

`GLIBCXX_*`, `CXXABI_*` and `GCC_*` requirements are listed, but they are not glibc
and are not checked against the ceiling.

### env

```sh
eval "$(cargo libc env --target x86_64-unknown-linux-gnu.2.17)"
cargo libc env --target x86_64-unknown-linux-gnu.2.17 --format cargo > glibc-2.17.toml
```

`env` prints the configuration for using plain Cargo (see
[Plain Cargo and CI](#plain-cargo-and-ci)).

### provider, doctor

`cargo libc provider ...` manages the provider cache ([Providers](#providers)).
`cargo libc doctor` probes the build inputs and prints what it found: the compiler,
the linker, the header version, and the startup files. With `--distro` or
`--kernel` it also checks the test runners.

## C dependencies

C compiled by build scripts through the [`cc`](https://crates.io/crates/cc) crate,
or by CMake through the [`cmake`](https://crates.io/crates/cmake) crate, is built
for the target automatically:

- **It sees the target's headers.** Under `_GNU_SOURCE`, glibc 2.38's headers turn
  `strtol` into `__isoc23_strtol@GLIBC_2.38`. Built for 2.17, the same code keeps
  `strtol@GLIBC_2.2.5`.
- **It links against the target.** CMake's configure checks, such as
  `check_function_exists(memfd_create)`, find only what the target release defines.
- **Host code stays native.** Build scripts and procedural macros run on your
  machine, so their C code keeps your compiler (`HOST_CC`, else `CC`, else `cc`).
- **Your flags still apply.** `CFLAGS` and the flags a build script sets pass
  through; cargo-libc's target settings come after them.

The limits:

- **C only.** C++ for the target is refused, because providers have no target C++
  runtime. A CMake project must declare `project(NAME C)`.
- **No static linking.** `-static` is refused, because there is no static glibc.
- **No host libraries.** pkg-config and libraries installed on your machine are not
  configured for the target. Their headers and glibc needs are your machine's, so
  build such dependencies from source with `cc` or CMake instead.
- **Two variables are reserved.** cargo-libc sets `CC_x86_64-unknown-linux-gnu`
  and `CXX_x86_64-unknown-linux-gnu`, and refuses to run if you set them.

## Testing tiers

cargo-libc offers three levels of evidence. Each is stronger than the one before.

1. **Tier 1, the scan** (`build`/`inspect` with `--distro` or `--kernel`). This is
   **the least reliable tier.** It reads the binary:
   - exactly: `GLIBC_*` requirements, `DT_RELR`, and imported syscall wrappers
     such as `statx` and `getrandom`;
   - heuristically: constant `syscall()` numbers and `madvise` advice.

   It cannot see syscalls inside glibc, values computed at run time, `dlopen`, or
   whether a fallback works. Rust's standard library, for example, tries `statx`
   and falls back on old kernels, so a `statx` finding is normal. Warnings fail
   only with `--strict`.
2. **Tier 2, simulation** (`test`/`run` with `--distro` or `--kernel`). This
   catches what your tests execute: a syscall the target kernel lacks returns
   `ENOSYS`, and unknown `madvise` advice returns `EINVAL`. It simulates only
   syscall availability and `madvise`. The local runner keeps your machine's
   libraries.
3. **Tier 3, a real target.** Run the program on a real system at your minimums,
   such as a RHEL 7 VM. Only this tier qualifies a release, and cargo-libc does not
   do it for you.

## Plain Cargo and CI

`cargo libc env` prints the settings so that plain Cargo builds the same way. This
is useful in CI scripts or for tools that call Cargo themselves:

```sh
eval "$(cargo libc env --target x86_64-unknown-linux-gnu.2.17 --kernel 3.10)"
cargo build --target x86_64-unknown-linux-gnu --release
cargo test  --target x86_64-unknown-linux-gnu        # through libc-runner
cargo libc inspect target/x86_64-unknown-linux-gnu/release/myapp --max-glibc 2.17
```

Or write a Cargo configuration file:

```sh
cargo libc env --target x86_64-unknown-linux-gnu.2.17 --format cargo > glibc-2.17.toml
cargo build --target x86_64-unknown-linux-gnu --config glibc-2.17.toml --release
```

The settings are:
- the target's rustflags: the linker helper, the provider's paths, and a
  configuration cfg;
- rustdoc flags;
- the `libc-runner` test runner;
- the C compiler wrappers.

Things to keep in mind:

- **Always pass `--target x86_64-unknown-linux-gnu`.** Without it, build scripts
  and procedural macros would get the target settings too.
- **`RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` replace these flags.** Cargo does
  not merge them.
- **Plain Cargo checks nothing.** Run `cargo libc inspect` on what it built.
- **The output is for this machine.** It holds absolute paths, so regenerate it
  after updating cargo-libc or a provider. `--format sh` refuses paths that
  contain whitespace.

A GitHub Actions job on `ubuntu-24.04`, which has GCC, Clang and binutils:

```yaml
- run: cargo install --git https://github.com/rustcommons/cargo-libc --locked
- run: cargo libc build --target x86_64-unknown-linux-gnu.2.17 --kernel 3.10
         --artifact target/x86_64-unknown-linux-gnu/release/myapp -- --release --locked
- run: cargo libc test --target x86_64-unknown-linux-gnu.2.17 --kernel 3.10 --runner local -- --locked
```

## Troubleshooting

| Message | Cause and fix |
| --- | --- |
| `RUSTFLAGS is already set` (or `RUSTDOCFLAGS`, `CARGO_ENCODED_*`) | cargo-libc manages these flags. Unset the variable for this command. |
| `CC_x86_64-unknown-linux-gnu is already set` | Unset it. Your `CC`/`HOST_CC` stays in use for host code. |
| `undefined reference to 'memfd_create'` (or another function) | The function is newer than your target. Raise the target, or avoid the function, for example by calling it through `libc::syscall`. |
| `GCC N built objects that differ from the qualified container build ... use --mode container` | Your GCC builds a startup object differently from the qualified build; the details file it names lists the differences. In cargo-libc, use `--provider-mode container`, or set `CARGO_LIBC_PROVIDER_GCC` to a GCC from 11 to 16. |
| `the host's libraries lack libc.so.6 GLIBC_2.41` from `test`/`run` | The target is newer than your machine. Use an older target, or `--runner container` with an image that has that glibc. |
| `--cc must be a Clang driver supporting --gcc-install-dir` | Install a newer Clang and pass `--cc /usr/bin/clang-NN`. |
| `C++ for the target is not supported yet` | A build script compiles C++ for the target. Only C is supported. |
| `was not reported as an executable by this Cargo invocation` | `--artifact` names a file this build did not produce. Check the path, profile and `--target-dir`. |
| Container runner: `statx` blocked while starting | The runtime calls syscalls the old kernel lacks. Use crun 1.28 or later via `KERNEL_ABI_ENGINE_ARGS`. |

`cargo libc doctor` with the same options shows which compiler, linker, headers and
startup files are being used.

## Limits

- **Platform.** It targets x86_64 Linux and glibc 2.17–2.44, and builds dynamically
  linked executables. It has no other architectures and no other C libraries yet.
- **What is checked.** Checks cover the executable itself, not the shared libraries
  it loads, `dlopen`, C++ runtime versions, or CPU instructions.
- **Dependencies.** C dependencies are supported, but C++, pkg-config and host
  system libraries are not.
- **Release status.** This is a pre-release. It does not replace testing on a real
  target (tier 3).

## Reference

**Exit codes.**
- **`build`:** Cargo's exit code if Cargo fails, 2 if the check or scan fails,
  and 1 for configuration errors.
- **`install`:** the same as `build`, for every executable it built.
- **`inspect`:** 0 if it passed, 2 if the check or scan failed, and 1 for input
  errors.
- **`check`, `clippy`, `doc`, `rustc`, `test`, `run`:** Cargo's exit code.
- **`libc-runner`:** makes a binary fail with 2 when it needs a newer glibc than
  allowed.
- **`doctor`:** 2 when the runner `test`/`run` would choose is unusable.

**Environment variables.**

| Variable | Meaning |
| --- | --- |
| `CARGO_LIBC_PROVIDER_DIR` | Provider cache (default `$CARGO_HOME/libc-providers`) |
| `CARGO_LIBC_PROVIDER_MODE` | Provider mode: `source` or `container` |
| `CARGO_LIBC_PROVIDER_GCC` | GCC for source mode and the startup files |
| `CONTAINER_ENGINE` | Container engine for the container runner |
| `KERNEL_ABI_ENGINE_ARGS`, `KERNEL_ABI_RUN_ARGS` | Extra container engine / run arguments |

cargo-libc sets other `CARGO_LIBC_*` variables for its helpers; you do not set them.

**More detail:**
- [how builds are controlled](../README.md#how-the-build-is-controlled);
- [Cargo workflows](CARGO-WORKFLOWS.md);
- [the provider contract](PROVIDER-CONTRACT.md);
- [what was validated](../VALIDATION.md).
