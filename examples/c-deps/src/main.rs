use std::ffi::{c_char, c_int, c_long, CStr};

extern "C" {
    fn c_parse(text: *const c_char) -> c_long;
    fn c_header_minor() -> c_int;
    fn cmake_running_glibc() -> *const c_char;
    fn cmake_header_minor() -> c_int;
    fn cmake_has_memfd_create() -> c_int;
}

fn main() {
    // SAFETY: the C functions take a NUL-terminated string or nothing, and
    // gnu_get_libc_version returns a static string.
    unsafe {
        println!("cc: glibc 2.{} headers", c_header_minor());
        println!("cc: strtol parsed {}", c_parse(c"0x2a".as_ptr()));
        println!("cmake: glibc 2.{} headers", cmake_header_minor());
        println!("cmake: memfd_create found = {}", cmake_has_memfd_create());
        let running = CStr::from_ptr(cmake_running_glibc()).to_string_lossy();
        println!("cmake: running on glibc {running}");
    }
    println!("host proc macro: glibc 2.{} headers", host_macro::host_header_minor!());
}
