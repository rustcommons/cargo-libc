#![forbid(unsafe_code)]

use std::io::Write;

fn main() {
    let args = std::env::args_os().skip(1);
    match cargo_libc::cli::run(args) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            // Reporting must not replace the deliberate failure status with a panic.
            let _ = writeln!(std::io::stderr().lock(), "cargo-libc: {error}");
            std::process::exit(1);
        }
    }
}
