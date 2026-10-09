use crate::{
    report,
    toolchain::{self, Toolchain},
    version::Version,
};
use std::{ffi::OsString, path::PathBuf};

const HELP: &str = r#"cargo-glibc — explicit sysroot builds and native ELF inspection

USAGE
  cargo glibc doctor --sysroot DIR --gcc-runtime DIR --glibc VERSION [OPTIONS]
  cargo glibc build  --sysroot DIR --gcc-runtime DIR --glibc VERSION
                    --artifact FILE [OPTIONS] -- [CARGO BUILD ARGUMENTS]
  cargo glibc inspect FILE [--max-glibc VERSION] [--json]

BUILD / DOCTOR OPTIONS
  --sysroot DIR      Target development filesystem, including glibc and GCC runtime
  --gcc-runtime DIR  Absolute GCC runtime directory inside that sysroot
  --glibc VERSION    Maximum permitted glibc version, for example 2.22
  --cc FILE          Host Clang driver (default: /usr/bin/clang)
  --ld FILE          Host native linker (default: /usr/bin/ld.bfd)
  --artifact FILE    Exact executable from this Cargo build to inspect (build only)
  --dry-run          Probe target inputs and print the command without building

GENERAL
  -h, --help         Show this help
  -V, --version      Show the tool version

This prototype supports x86-64 Linux dynamically linked Rust executables.
It requires an existing target sysroot; --glibc does not create or downgrade one.
It uses a Rust linker helper, a host Clang driver, and the selected native linker.
It does not use Zig, Python, or third-party Rust crates.

The ELF check covers direct version requirements in the selected file.
It does not resolve dependent libraries or replace testing on the target OS.
Native C/C++ dependencies need their own target configuration in this version.

Use separate values for options, e.g. --glibc 2.22 (not --glibc=2.22).
Direct invocation as `cargo-glibc ...` is also supported.
"#;

#[derive(Debug)]
struct RawConfig {
    sysroot: PathBuf,
    gcc_runtime: PathBuf,
    cc: PathBuf,
    ld: PathBuf,
    ceiling: Version,
}

impl RawConfig {
    fn resolve(self) -> Result<Toolchain, String> {
        Toolchain::new(
            self.sysroot,
            self.gcc_runtime,
            self.cc,
            self.ld,
            self.ceiling,
        )
    }
}

#[derive(Debug)]
enum ParsedCommand {
    Help,
    Version,
    Doctor(RawConfig),
    Build {
        config: RawConfig,
        artifact: PathBuf,
        dry_run: bool,
        cargo_args: Vec<OsString>,
    },
    Inspect {
        file: PathBuf,
        ceiling: Option<Version>,
        json: bool,
    },
}

fn next_value(args: &[OsString], i: &mut usize, flag: &str) -> Result<OsString, String> {
    *i += 1;
    let value = args
        .get(*i)
        .ok_or_else(|| format!("{flag} needs a value"))?;
    if value.is_empty() || value == "--" {
        return Err(format!("{flag} needs a nonempty value"));
    }
    Ok(value.clone())
}

fn parse_version(value: OsString, flag: &str) -> Result<Version, String> {
    value
        .to_str()
        .ok_or_else(|| format!("{flag} must be UTF-8"))?
        .parse()
}

fn parse(mut args: Vec<OsString>) -> Result<ParsedCommand, String> {
    // Cargo includes the external subcommand's name in argv.
    if args.first().is_some_and(|arg| arg == "glibc") {
        args.remove(0);
    }
    let Some(command) = args.first() else {
        return Ok(ParsedCommand::Help);
    };
    match command.to_str() {
        Some("-h" | "--help" | "help") => return Ok(ParsedCommand::Help),
        Some("-V" | "--version") => return Ok(ParsedCommand::Version),
        Some("inspect") => return parse_inspect(&args[1..]),
        Some("doctor" | "build") => {}
        _ => {
            return Err(format!(
                "unknown command {command:?}; run cargo glibc --help"
            ))
        }
    }
    let build = command == "build";
    let mut sysroot = None;
    let mut gcc_runtime = None;
    let mut ceiling = None;
    let mut cc = PathBuf::from("/usr/bin/clang");
    let mut ld = PathBuf::from("/usr/bin/ld.bfd");
    let mut artifact = None;
    let mut dry_run = false;
    let mut cargo_args = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].to_str() {
            Some("-h" | "--help") => return Ok(ParsedCommand::Help),
            Some("--sysroot") => {
                sysroot = Some(PathBuf::from(next_value(&args, &mut i, "--sysroot")?))
            }
            Some("--gcc-runtime") => {
                gcc_runtime = Some(PathBuf::from(next_value(&args, &mut i, "--gcc-runtime")?))
            }
            Some("--cc") => cc = PathBuf::from(next_value(&args, &mut i, "--cc")?),
            Some("--ld") => ld = PathBuf::from(next_value(&args, &mut i, "--ld")?),
            Some("--glibc") => {
                ceiling = Some(parse_version(
                    next_value(&args, &mut i, "--glibc")?,
                    "--glibc",
                )?)
            }
            Some("--artifact") if build => {
                artifact = Some(PathBuf::from(next_value(&args, &mut i, "--artifact")?))
            }
            Some("--dry-run") if build => dry_run = true,
            Some("--") if build => {
                cargo_args.extend_from_slice(&args[i + 1..]);
                break;
            }
            _ => {
                return Err(format!(
                    "unexpected option {:?}; Cargo arguments belong after --",
                    args[i]
                ))
            }
        }
        i += 1;
    }
    let config = RawConfig {
        sysroot: sysroot.ok_or("--sysroot is required")?,
        gcc_runtime: gcc_runtime.ok_or("--gcc-runtime is required")?,
        cc,
        ld,
        ceiling: ceiling.ok_or("--glibc is required")?,
    };
    if build {
        toolchain::validate_cargo_args(&cargo_args)?;
        Ok(ParsedCommand::Build {
            config,
            artifact: artifact.ok_or(
                "--artifact is required; name the exact executable produced by this build",
            )?,
            dry_run,
            cargo_args,
        })
    } else {
        Ok(ParsedCommand::Doctor(config))
    }
}

fn parse_inspect(args: &[OsString]) -> Result<ParsedCommand, String> {
    let mut file = None;
    let mut ceiling = None;
    let mut json = false;
    let mut positional_only = false;
    let mut i = 0;
    while i < args.len() {
        if !positional_only {
            match args[i].to_str() {
                Some("-h" | "--help") => return Ok(ParsedCommand::Help),
                Some("--max-glibc") => {
                    ceiling = Some(parse_version(
                        next_value(args, &mut i, "--max-glibc")?,
                        "--max-glibc",
                    )?);
                    i += 1;
                    continue;
                }
                Some("--json") => {
                    json = true;
                    i += 1;
                    continue;
                }
                Some("--") => {
                    positional_only = true;
                    i += 1;
                    continue;
                }
                Some(s) if s.starts_with('-') => {
                    return Err(format!("unknown inspect option {s:?}"))
                }
                _ => {}
            }
        }
        if file.replace(PathBuf::from(&args[i])).is_some() {
            return Err("inspect accepts exactly one ELF file".into());
        }
        i += 1;
    }
    Ok(ParsedCommand::Inspect {
        file: file.ok_or("inspect requires an ELF file path")?,
        ceiling,
        json,
    })
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<i32, String> {
    match parse(args.into_iter().collect())? {
        ParsedCommand::Help => {
            print!("{HELP}");
            Ok(0)
        }
        ParsedCommand::Version => {
            println!("cargo-glibc {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        ParsedCommand::Doctor(raw) => {
            let config = raw.resolve()?;
            config.print_diagnostics(&config.diagnose()?);
            Ok(0)
        }
        ParsedCommand::Build {
            config,
            artifact,
            dry_run,
            cargo_args,
        } => toolchain::build(&config.resolve()?, &cargo_args, &artifact, dry_run),
        ParsedCommand::Inspect {
            file,
            ceiling,
            json,
        } => report::inspect_file(&file, ceiling, json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn cargo_subcommand_preserves_paths_and_passthrough() {
        let parsed = parse(args(&[
            "glibc",
            "build",
            "--sysroot",
            "/sdk root",
            "--gcc-runtime",
            "/sdk root/gcc",
            "--glibc",
            "2.22",
            "--artifact",
            "target/app name",
            "--",
            "--release",
            "--offline",
        ]))
        .unwrap();
        let ParsedCommand::Build {
            config,
            artifact,
            cargo_args,
            ..
        } = parsed
        else {
            panic!("wrong command")
        };
        assert_eq!(config.sysroot, PathBuf::from("/sdk root"));
        assert_eq!(artifact, PathBuf::from("target/app name"));
        assert_eq!(cargo_args, args(&["--release", "--offline"]));
    }

    #[test]
    fn incomplete_and_ambiguous_commands_are_errors() {
        for argv in [
            vec!["build", "--sysroot"],
            vec!["inspect"],
            vec!["inspect", "a", "b"],
            vec!["install"],
            vec!["doctor", "--dry-run"],
        ] {
            assert!(parse(args(&argv)).is_err(), "{argv:?}");
        }
    }

    #[test]
    fn inspect_supports_literal_dash_filename_and_numeric_ceiling() {
        let parsed = parse(args(&[
            "inspect",
            "--max-glibc",
            "2.22",
            "--json",
            "--",
            "-app",
        ]))
        .unwrap();
        let ParsedCommand::Inspect {
            file,
            ceiling,
            json,
        } = parsed
        else {
            panic!("wrong command")
        };
        assert_eq!(file, PathBuf::from("-app"));
        assert_eq!(ceiling, Some("2.22".parse().unwrap()));
        assert!(json);
    }
}
