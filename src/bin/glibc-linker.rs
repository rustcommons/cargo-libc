#![forbid(unsafe_code)]

fn main() {
    match cargo_glibc::toolchain::run_linker(std::env::args_os().skip(1)) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("glibc-linker: {error}");
            std::process::exit(1);
        }
    }
}
