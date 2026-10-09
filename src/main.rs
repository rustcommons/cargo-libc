#![forbid(unsafe_code)]

fn main() {
    let args = std::env::args_os().skip(1);
    match cargo_libc::cli::run(args) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("cargo-libc: {error}");
            std::process::exit(1);
        }
    }
}
