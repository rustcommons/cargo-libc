#![forbid(unsafe_code)]

fn main() {
    std::process::exit(cargo_libc::cc::run(cargo_libc::cc::Language::Cxx));
}
