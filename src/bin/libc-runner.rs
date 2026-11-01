#![forbid(unsafe_code)]

use std::io::Write;

fn main() {
    match cargo_libc::simulate::run_program(std::env::args_os().skip(1)) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            // Reporting must not replace the deliberate failure status with a panic.
            let _ = writeln!(std::io::stderr().lock(), "libc-runner: {error}");
            std::process::exit(1);
        }
    }
}
