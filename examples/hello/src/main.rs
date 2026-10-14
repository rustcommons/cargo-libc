fn main() {
    println!(
        "{}; {}",
        hello_macro::message!(),
        env!("HELLO_BUILD_SCRIPT")
    );
    let result = std::thread::spawn(|| 6 * 7)
        .join()
        .expect("worker panicked");
    assert_eq!(result, 42);
    println!("target executable ran; thread result = {result}");
    println!("runtime glibc {}", runtime_glibc());
}

/// The version of the glibc the program is running on, which under a container
/// runner is the image's, not the build host's.
fn runtime_glibc() -> String {
    extern "C" {
        fn gnu_get_libc_version() -> *const std::ffi::c_char;
    }
    // SAFETY: glibc returns a static, NUL-terminated string.
    unsafe { std::ffi::CStr::from_ptr(gnu_get_libc_version()) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn runs_on_a_glibc() {
        let version = super::runtime_glibc();
        assert!(version.starts_with("2."), "{version}");
        println!("test ran on runtime glibc {version}");
    }
}
