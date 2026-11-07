//! Parser failures must happen before inspecting files or resolving build inputs.

use std::process::Command;

#[test]
fn duplicate_value_options_fail_before_work_with_empty_stdout() {
    for (command, flag, first, second) in [
        ("doctor", "--sysroot", "missing-sdk", "other-sdk"),
        ("doctor", "--gcc-runtime", "missing-gcc", "other-gcc"),
        ("check", "--cc", "missing-clang", "other-clang"),
        ("run", "--ld", "missing-linker", "other-linker"),
        ("build", "--artifact", "missing-app", "other-app"),
        ("inspect", "--max-glibc", "2.17", "2.22"),
    ] {
        for repeated in [first, second] {
            // No usable provider/compiler/ELF is supplied: duplicate detection
            // must finish before any of those operations can be attempted.
            let output = Command::new(env!("CARGO_BIN_EXE_cargo-libc"))
                .args(["libc", command, flag, first, flag, repeated])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{command} {flag}");
            assert!(output.stdout.is_empty(), "{command} {flag}");
            let error = String::from_utf8(output.stderr).unwrap();
            assert!(error.contains(&format!("duplicate {flag}")), "{error}");
        }
    }
}
