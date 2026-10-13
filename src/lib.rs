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
//! Install the command and read how to use it in the
//! [user guide](https://github.com/rustcommons/cargo-libc/blob/main/docs/USER-GUIDE.md).
//!
//! This library holds the implementation of the `cargo-libc`, `libc-linker`,
//! `libc-runner`, `libc-cc` and `libc-cxx` executables; its modules are public for
//! them and for tests.
#![forbid(unsafe_code)]

pub mod cargo_output;
pub mod cc;
pub mod cli;
pub mod elf;
pub mod provider;
pub mod report;
pub mod simulate;
pub mod toolchain;
pub mod version;
