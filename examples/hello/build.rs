fn main() {
    // This executable runs on the build host, so it must keep the host linker.
    println!("cargo::rustc-env=HELLO_BUILD_SCRIPT=host build script ran");
    println!("cargo::rerun-if-changed=build.rs");
}
