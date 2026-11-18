//! Fuzz-only entry points, compiled only with `--cfg fuzzing` (set by
//! cargo-fuzz). Each calls one pure parser on untrusted bytes. None resolves
//! SDKs, reads provider files, probes tools or runs parsed input. Shipped
//! builds never contain this module.
use crate::version::Version;
use std::{ffi::OsString, hint::black_box, os::unix::ffi::OsStringExt};

/// glibc versions and `TARGET[.GLIBC]` release suffixes.
pub fn version(data: &[u8]) {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(version) = text.parse::<Version>() {
        // Display and parsing must agree for every accepted version.
        let shown = version.to_string();
        assert_eq!(shown.parse::<Version>().as_ref(), Ok(&version), "{shown}");
        let _ = black_box(version.validate_glibc_target());
    }
}

/// The `cargo libc` command line, including `--target` specs; NUL separates
/// arguments. Parsing only: no command runs.
pub fn cli_args(data: &[u8]) {
    let args: Vec<OsString> = data
        .split(|&b| b == 0)
        .map(|arg| OsString::from_vec(arg.to_vec()))
        .collect();
    crate::cli::fuzz_parse(args);
}

/// `libc-abi-provider-v1` descriptor text; its paths are never opened.
pub fn provider_descriptor(data: &[u8]) {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(provider) = crate::provider::Provider::parse(text) {
            assert!(crate::provider::RELEASES.contains(&provider.release.to_string().as_str()));
            black_box(provider);
        }
    }
}

/// glibc `features.h` version macros from an SDK sysroot.
pub fn sdk_header(data: &[u8]) {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = black_box(crate::toolchain::header_version(text));
    }
}

/// One line of Cargo's `--message-format=json` output.
pub fn cargo_message(data: &[u8]) {
    if let Ok(line) = std::str::from_utf8(data) {
        let _ = black_box(crate::cargo_output::executable_from_message(line));
    }
}

/// ELF dynamic metadata, symbol-version requirements/definitions and the
/// exact dynsym inventory used by `inspect`.
pub fn elf(data: &[u8]) {
    let info = crate::elf::inspect(data);
    let versions = crate::elf::exact::validated_versions(data);
    let symbols = crate::elf::exact::inspect(data);
    // The exact reader validates the same metadata first.
    if info.is_err() {
        assert!(versions.is_err() && symbols.is_err());
    }
    let _ = black_box((info, versions, symbols));
}

/// Clang `-###` command-trace quoting and GNU linker-script tokens.
pub fn linker_syntax(data: &[u8]) {
    let first = crate::clock_named::quoted_args_limit(data, true);
    let all = crate::clock_named::quoted_args_limit(data, false);
    if let (Ok(first), Ok(all)) = (&first, &all) {
        assert_eq!(first.first(), all.first());
    }
    let _ = black_box((first, all, crate::clock_named::script_tokens(data)));
}
