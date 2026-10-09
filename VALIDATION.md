# Validation record — version 0.1.0

Validated on 2026-10-09. These results apply to the source in this package.

## Environment and the limit of the result

| Component | Tested value |
| --- | --- |
| Build/run host | Ubuntu 24.04.3, x86-64 |
| Host glibc | 2.39 |
| rustc | 1.99.0 (`b940084d7`, 2026-09-28) |
| Cargo | 1.99.0 (`5f94df478`, 2026-08-27) |
| Clang | Ubuntu Clang 18.1.3 |
| Native linker | GNU `ld.bfd`, binutils 2.42 |
| Validation sysroot | `/` on that Ubuntu host |
| Validation GCC runtime | `/usr/lib/gcc/x86_64-linux-gnu/13` |

**No SLES 12 SP5 sysroot or glibc 2.22 runtime was available for this
validation. No Rocky Linux 9 host was tested.** The tests establish that the
implemented host build path, Cargo artifact selection, and rejection checks
work in the environment above. They do not establish that a SLES-targeted
application has been built or run successfully.

The declared Rust 1.90 minimum was not separately tested. No prebuilt tool
binary is included: build the source on your own build host.

## Source build and tests

```sh
cargo test --locked --offline
cargo build --locked --offline --release
cargo fmt --all -- --check
```

All succeeded. **38 unit tests passed**, with no failed or ignored tests:

| Area | Tests | Selected coverage |
| --- | ---: | --- |
| ELF parser | 14 | Sectionless/stripped metadata, bounds, overflow, truncated inputs, malformed chains/strings, version-level weak flags |
| Cargo JSON parser | 12 | Fresh artifacts, top-level identity, nested fields, Unicode escapes, malformed input, bounded nesting/size |
| Direct glibc assessment and output | 4 | Numeric requirements, weak versions, nonnumeric ABI tags, non-glibc tags, JSON escaping |
| Command parsing | 3 | Cargo subcommand invocation, path preservation, missing/ambiguous arguments |
| Toolchain configuration | 3 | Header parsing, conflicting Cargo options, paths containing spaces |
| Version ordering | 2 | Numeric ordering, malformed versions |

The main package's Cargo.lock contains only `cargo-glibc`. The example's
lockfile contains only the example and its local procedural-macro crate.

## Local installation

Installing from source with `cargo install --path . --locked --offline`
and a private test `--root` succeeded. It installed both executables.
Actual `cargo glibc --version` and `cargo glibc --help` invocations succeeded
after adding that private bin directory to PATH.

## Target-input checks

`doctor` succeeded with the validation sysroot and a 2.39 ceiling. It resolved
all listed CRT, glibc development, and GCC runtime inputs inside that root.

Using the same sysroot with a 2.22 ceiling was rejected with exit 1:

```text
sysroot headers are glibc 2.39, newer than requested ceiling 2.22;
supply older target development files
```

## Complete build and execution

The included `examples/hello` package contains a build script, a local
procedural macro, and a target executable that starts and joins a thread.
The release `cargo-glibc` binary built it using the validation toolchain,
`--locked --offline`, an explicit target output directory, and verbose Cargo
output.

The first build exited 0. Inspection of actual rustc command lines confirmed:

- The host build script had neither the target flag nor the managed linker.
- The host procedural macro had neither the target flag nor the managed linker.
- The target executable had both `--target x86_64-unknown-linux-gnu` and the
  managed `glibc-linker` path.

The resulting executable ran on the validation host and exited 0:

```text
host procedural macro ran; host build script ran
target executable ran; thread result = 42
```

Its interpreter was `/lib64/ld-linux-x86-64.so.2`. It had no RPATH or RUNPATH.
Its highest recorded mandatory numeric glibc requirement was **2.34**, so the
direct check passed the actual 2.39 build ceiling.

## Cache and artifact identity

Repeating the same build exited 0. Cargo reported both example packages as
fresh and emitted no rustc compile command. Matching the selected executable
against Cargo's JSON and inspecting the resulting file still succeeded.

Selecting `/usr/bin/true` as the artifact while building that example was
rejected with exit 1, because Cargo had not reported that file as an executable
from the invocation. An arbitrary existing file therefore cannot stand in for
the selected build output.

## Requirement rejection

Inspecting the built example with `--max-glibc 2.22 --json` exited 2. Its JSON
reported a highest required glibc version of 2.34 and a failed direct check.

Independently inspecting `/usr/bin/true` with the same 2.22 ceiling also exited
2 because of a recorded `GLIBC_2.34` requirement. The parser's six version
requirements for that system file were compared with `readelf`, including
their order and version-level flags.

The parser also read the host libc and dynamic loader. Reading those files is
parser validation, not a declaration that a runtime libc should be assessed as
an application executable.

## Outstanding qualification work

The next environment-specific gate is to use a coherent SLES 12 SP5 development
sysroot on the intended Rocky host, inspect the output and deployed libraries,
and run the actual application on SLES. Native C/C++ dependencies, alternate
linkers, other architectures, generated glibc stubs, and dependency-closure
resolution remain outside the tested feature set.
