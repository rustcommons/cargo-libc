#![forbid(unsafe_code)]

fn main() {
    match cargo_libc::simulate::run_program(std::env::args_os().skip(1)) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("libc-runner: {error}");
            std::process::exit(1);
        }
    }
}
