//! A required symbol newer than both generated provider ceilings.
extern "C" {
    fn memfd_create(name: *const std::ffi::c_char, flags: u32) -> i32;
}
fn main() {
    let result = unsafe { memfd_create(b"provider-negative\0".as_ptr().cast(), 0) };
    std::process::exit(if result >= 0 { 0 } else { 1 });
}
