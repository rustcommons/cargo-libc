#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| cargo_libc::fuzzing::version(data));
