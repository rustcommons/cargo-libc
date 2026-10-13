fn main() {
    // Target C code: compiled by the `cc` crate and by CMake, for the selected glibc.
    cc::Build::new().file("c/parse.c").compile("parse");
    let cmake = cmake::build("cmake");
    println!("cargo::rustc-link-search=native={}/lib", cmake.display());
    println!("cargo::rustc-link-lib=static=cmakepart");
    if std::env::var_os("CARGO_FEATURE_CXX").is_some() {
        cc::Build::new().cpp(true).file("c/unsupported.cpp").compile("unsupported");
    }
    println!("cargo::rerun-if-changed=c");
    println!("cargo::rerun-if-changed=cmake");
}
