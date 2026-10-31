//! cargo-libc: build Rust programs for an older glibc.
//!
//! cargo-libc is a Cargo subcommand, `cargo libc`. It builds x86_64 Linux programs on
//! a current machine for a glibc release you choose, 2.17 through 2.44. It links
//! against exactly what that release defines, checks the executables' requirements,
//! and runs tests under a simulation of the target's kernel.
//!
//! ```sh
//! cargo libc build --target x86_64-unknown-linux-gnu.2.17 \
//!   --artifact target/x86_64-unknown-linux-gnu/release/myapp -- --release
//! ```
//!
//! Install it with `cargo install cargo-libc --locked`, and read how to use it in the
//! [user guide](https://github.com/rustcommons/cargo-libc/blob/main/docs/USER-GUIDE.md)
//! and the [README](https://github.com/rustcommons/cargo-libc#readme).
//!
//! # The library
//!
//! This library is the implementation of cargo-libc's five executables, and is not
//! meant to be used directly. Its public items are their entry points:
//!
//! | Executable | Entry point | Role |
//! | --- | --- | --- |
//! | `cargo-libc` | [`cli::run`] | The `cargo libc` command |
//! | `libc-linker` | [`toolchain::run_linker`] | The linker Cargo calls for target units |
//! | `libc-runner` | [`simulate::run_program`] | Cargo's target runner for `test` and `run` |
//! | `libc-cc` | [`cc::run`] with [`cc::Language::C`] | The C compiler for the `cc` crate and CMake |
//! | `libc-cxx` | [`cc::run`] with [`cc::Language::Cxx`] | The C++ compiler: host units only |
//!
//! To use provider preparation or ELF scanning from your own code, depend on
//! [`libc-abi-tools`](https://docs.rs/libc-abi-tools) and
//! [`linux-abi-scan`](https://docs.rs/linux-abi-scan).
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cargo_output;
pub mod cc;
pub mod cli;
mod elf;
mod output;
mod provider;
mod report;
pub mod simulate;
pub mod toolchain;
mod version;
