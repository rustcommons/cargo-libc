fn main() {
    // A proc macro runs on the build host: its C code must use the host's compiler.
    cc::Build::new().file("host.c").compile("host");
    println!("cargo::rerun-if-changed=host.c");
}
