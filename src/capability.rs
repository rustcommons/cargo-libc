//! Bounded observations, not a host ABI or deployment compatibility certificate.

use std::{
    fs,
    io::{self, Read},
    path::Path,
};

const MAPS_LIMIT: u64 = 1024 * 1024;

fn mapped_libc(maps: &str) -> &'static str {
    let mut glibc = false;
    let mut musl = false;
    for line in maps.lines() {
        let Some(path) = line.split_whitespace().nth(5) else {
            continue;
        };
        let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        glibc |= name == "libc.so.6";
        musl |= name.starts_with("ld-musl-") && name.ends_with(".so.1");
    }
    match (glibc, musl) {
        (true, false) => {
            "mapped libc.so.6 observed (name only; version and compatibility unproven)"
        }
        (false, true) => {
            "mapped musl loader observed (name only; GNU execution capability unknown)"
        }
        _ => "unknown (static tool, absent or conflicting mapped-libc evidence)",
    }
}

fn host_observation() -> String {
    let result = (|| -> io::Result<String> {
        let mut maps = String::new();
        fs::File::open("/proc/self/maps")?
            .take(MAPS_LIMIT + 1)
            .read_to_string(&mut maps)?;
        if maps.len() as u64 > MAPS_LIMIT {
            return Ok("unknown (/proc/self/maps observation budget exceeded)".into());
        }
        Ok(mapped_libc(&maps).into())
    })();
    result.unwrap_or_else(|e| format!("unknown (cannot observe /proc/self/maps: {e})"))
}

pub(crate) fn report(
    line: fn(std::fmt::Arguments<'_>) -> Result<(), String>,
) -> Result<(), String> {
    let abi = if cfg!(target_env = "gnu") {
        "gnu"
    } else if cfg!(target_env = "musl") {
        "musl"
    } else {
        "other/unknown"
    };
    line(format_args!(
        "Tool build provenance: {}-{}; ABI {abi} (not host libc detection)",
        std::env::consts::ARCH,
        std::env::consts::OS
    ))?;
    line(format_args!("Host observation: {}", host_observation()))?;
    line(format_args!("Build preflight checks selected tools and target inputs only; host build scripts keep their host toolchain. Non-glibc build hosts are unqualified, not rejected by distro or tool ABI. GNU target execution and complete runtime dependencies are separate checks."))
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Interpreter {
    Observed,
    Missing,
    Unknown(String),
}

fn interpreter_result(result: io::Result<fs::Metadata>) -> Interpreter {
    match result {
        Ok(meta) if meta.is_file() => Interpreter::Observed,
        Ok(_) => Interpreter::Unknown("path is not a regular file".into()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Interpreter::Missing,
        Err(e) => Interpreter::Unknown(e.to_string()),
    }
}

/// Called only after ELF validation and execution-route selection. Never execute a probe.
pub(crate) fn interpreter(path: &str) -> Interpreter {
    if !Path::new(path).is_absolute() {
        return Interpreter::Unknown("PT_INTERP is not absolute".into());
    }
    interpreter_result(fs::metadata(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libc_observations_do_not_infer_os_from_build_abi() {
        let gnu = "0-1 r-xp 0 00:00 0 /lib/libc.so.6\n";
        let musl = "0-1 r-xp 0 00:00 0 /lib/ld-musl-x86_64.so.1\n";
        assert!(mapped_libc(gnu).contains("name only"));
        assert!(mapped_libc(musl).contains("GNU execution capability unknown"));
        assert!(mapped_libc("").starts_with("unknown"));
        assert!(mapped_libc(&format!("{gnu}{musl}")).starts_with("unknown"));
    }

    #[test]
    fn interpreter_missing_and_unreadable_are_distinct() {
        assert_eq!(
            interpreter_result(Err(io::ErrorKind::NotFound.into())),
            Interpreter::Missing
        );
        assert!(matches!(
            interpreter_result(Err(io::ErrorKind::PermissionDenied.into())),
            Interpreter::Unknown(_)
        ));
        assert_eq!(
            interpreter_result(fs::metadata(std::env::current_exe().unwrap())),
            Interpreter::Observed
        );
        assert!(matches!(
            interpreter("relative-loader"),
            Interpreter::Unknown(_)
        ));
    }
}
