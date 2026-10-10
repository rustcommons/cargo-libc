use crate::{
    report::{self, ScanOptions},
    toolchain::{self, Toolchain},
    version::Version,
};
use std::{ffi::OsString, path::PathBuf};

const HELP: &str = r#"cargo-libc — explicit sysroot builds and native ELF inspection

USAGE
  cargo libc doctor --sysroot DIR --gcc-runtime DIR --glibc VERSION [OPTIONS]
  cargo libc build  --sysroot DIR --gcc-runtime DIR --glibc VERSION
                    --artifact FILE [OPTIONS] -- [CARGO BUILD ARGUMENTS]
  cargo libc inspect FILE [--max-glibc VERSION | --distro ID] [--kernel VERSION]
                     [--scan] [--strict] [--verbose] [--json]

BUILD / DOCTOR OPTIONS
  --provider FILE    Experimental glibc 2.17/2.18/2.22 provider descriptor instead of sysroot/GCC options
  --sysroot DIR      Target development filesystem, including glibc and GCC runtime
  --gcc-runtime DIR  Absolute GCC runtime directory inside that sysroot
  --glibc VERSION    Maximum permitted glibc version, at least 2.17 (e.g. 2.22)
  --target TARGET    x86_64-unknown-linux-gnu[.GLIBC]; suffix supplies --glibc
  --cc FILE          Host Clang driver (default: /usr/bin/clang)
  --ld FILE          Host native linker (default: /usr/bin/ld.bfd)
  --artifact FILE    Exact executable from this Cargo build to inspect (build only)
  --dry-run          Probe target inputs and print the command without building

INSPECT OPTIONS
  --max-glibc VERSION  Fail when a mandatory GLIBC_* requirement is newer
  --distro ID          Target preset (e.g. sles-12-sp5): sets the glibc and
                       kernel floors and runs the tier 1 scan
  --kernel VERSION     Kernel floor (e.g. 3.10); runs the tier 1 scan
  --scan               Run the tier 1 scan with only the --max-glibc floor
  --strict             Scan warnings (heuristic or optional use) also fail
  --verbose            Show the scan's informational findings
  --json               Machine-readable report

GENERAL
  -h, --help         Show this help
  -V, --version      Show the tool version

This prototype supports x86-64 Linux dynamically linked Rust executables.
It requires an existing target sysroot; --glibc does not create or downgrade one.
It uses a Rust linker helper, a host Clang driver, and the selected native linker.
It does not use Zig, Python, or third-party Rust crates.

The ELF check covers direct version requirements in the selected file.
The tier 1 scan (linux-abi-scan) adds kernel checks: imported syscall
wrappers and DT_RELR exactly, constant syscall()/madvise() call sites
heuristically. Neither resolves dependent libraries or replaces running the
program under a kernel simulation or on the target OS.
Native C/C++ dependencies need their own target configuration in this version.

Use separate values for options, e.g. --glibc 2.22 (not --glibc=2.22).
Direct invocation as `cargo-libc ...` is also supported.
"#;

#[derive(Debug)]
struct RawConfig {
    provider: Option<PathBuf>,
    sysroot: PathBuf,
    gcc_runtime: PathBuf,
    cc: PathBuf,
    ld: PathBuf,
    ceiling: Version,
}

impl RawConfig {
    fn resolve(self) -> Result<Toolchain, String> {
        if let Some(path) = self.provider {
            return Toolchain::from_provider(&path, self.cc, self.ld, self.ceiling);
        }
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
        scan: Option<ScanOptions>,
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
        .parse::<Version>()?
        .validate_glibc_target()
}

fn parse(mut args: Vec<OsString>) -> Result<ParsedCommand, String> {
    // Cargo includes the external subcommand's name in argv.
    if args.first().is_some_and(|arg| arg == "libc") {
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
                "unknown command {command:?}; run cargo libc --help"
            ))
        }
    }
    let build = command == "build";
    let mut provider = None;
    let mut sysroot = None;
    let mut gcc_runtime = None;
    let mut ceiling = None;
    let mut target = None;
    let mut cc = PathBuf::from("/usr/bin/clang");
    let mut ld = PathBuf::from("/usr/bin/ld.bfd");
    let mut artifact = None;
    let mut dry_run = false;
    let mut cargo_args = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].to_str() {
            Some("-h" | "--help") => return Ok(ParsedCommand::Help),
            Some("--provider") => {
                if provider.is_some() {
                    return Err("duplicate --provider".into());
                }
                provider = Some(PathBuf::from(next_value(&args, &mut i, "--provider")?));
            }
            Some("--sysroot") => {
                sysroot = Some(PathBuf::from(next_value(&args, &mut i, "--sysroot")?))
            }
            Some("--gcc-runtime") => {
                gcc_runtime = Some(PathBuf::from(next_value(&args, &mut i, "--gcc-runtime")?))
            }
            Some("--cc") => cc = PathBuf::from(next_value(&args, &mut i, "--cc")?),
            Some("--ld") => ld = PathBuf::from(next_value(&args, &mut i, "--ld")?),
            Some("--target") => {
                if target.is_some() {
                    return Err("duplicate --target".into());
                }
                let value = next_value(&args, &mut i, "--target")?;
                let text = value.to_str().ok_or("--target must be UTF-8")?;
                let (triple, suffix) = text
                    .split_once('.')
                    .map_or((text, None), |(t, v)| (t, Some(v)));
                if triple != toolchain::TARGET {
                    return Err(format!(
                        "unsupported target {triple:?}; expected {}",
                        toolchain::TARGET
                    ));
                }
                target = Some(
                    suffix
                        .map(|v| parse_version(v.into(), "--target"))
                        .transpose()?,
                );
            }
            Some("--glibc") => {
                if ceiling.is_some() {
                    return Err("duplicate --glibc".into());
                }
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
    if provider.is_some() && (sysroot.is_some() || gcc_runtime.is_some()) {
        return Err("--provider conflicts with --sysroot and --gcc-runtime".into());
    }
    if let Some(Some(version)) = target {
        if ceiling.is_some_and(|explicit| explicit != version) {
            return Err("--target version conflicts with --glibc".into());
        }
        ceiling = Some(version);
    }
    let config = RawConfig {
        provider: provider.clone(),
        sysroot: if provider.is_some() {
            PathBuf::new()
        } else {
            sysroot.ok_or("--sysroot is required")?
        },
        gcc_runtime: if provider.is_some() {
            PathBuf::new()
        } else {
            gcc_runtime.ok_or("--gcc-runtime is required")?
        },
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
    let mut scan = ScanOptions::default();
    let mut scan_requested = false;
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
                Some(flag @ ("--distro" | "--kernel")) => {
                    let value = next_value(args, &mut i, flag)?
                        .into_string()
                        .map_err(|_| format!("{flag} must be UTF-8"))?;
                    let slot = if flag == "--distro" {
                        &mut scan.distro
                    } else {
                        &mut scan.kernel
                    };
                    if slot.replace(value).is_some() {
                        return Err(format!("duplicate {flag}"));
                    }
                    i += 1;
                    continue;
                }
                Some(flag @ ("--scan" | "--strict" | "--verbose")) => {
                    match flag {
                        "--scan" => scan_requested = true,
                        "--strict" => scan.strict = true,
                        _ => scan.verbose = true,
                    }
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
    if scan.distro.is_some() && ceiling.is_some() {
        return Err("--distro sets the glibc floor; do not combine it with --max-glibc".into());
    }
    let scan = if scan_requested || scan.distro.is_some() || scan.kernel.is_some() {
        Some(scan)
    } else if scan.strict || scan.verbose {
        return Err(
            "--strict and --verbose apply to the scan; add --distro, --kernel, or --scan".into(),
        );
    } else {
        None
    };
    Ok(ParsedCommand::Inspect {
        file: file.ok_or("inspect requires an ELF file path")?,
        ceiling,
        scan,
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
            println!("cargo-libc {}", env!("CARGO_PKG_VERSION"));
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
            scan,
            json,
        } => report::inspect_file(&file, ceiling, scan.as_ref(), json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provider_selection_is_explicit_and_excludes_sysroot_options() {
        let base = [
            "doctor",
            "--provider",
            "/sdk/provider.txt",
            "--glibc",
            "2.17",
        ];
        let ParsedCommand::Doctor(config) = parse(args(&base)).unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(config.provider, Some(PathBuf::from("/sdk/provider.txt")));
        for extra in [
            ["--sysroot", "/sdk"],
            ["--gcc-runtime", "/sdk/gcc"],
            ["--provider", "/other"],
        ] {
            let mut values = base.to_vec();
            values.extend(extra);
            assert!(parse(args(&values)).is_err());
        }
        assert!(parse(args(&["doctor", "--provider", "/sdk/provider.txt"])).is_err());
    }
    #[test]
    fn versioned_target_selects_ceiling_and_preserves_cargo_arguments() {
        for version in ["2.17", "2.18", "2.22"] {
            let target = format!("{}.{}", toolchain::TARGET, version);
            let ParsedCommand::Build {
                config, cargo_args, ..
            } = parse(args(&[
                "build",
                "--provider",
                "/sdk/provider.txt",
                "--target",
                &target,
                "--artifact",
                "target/app",
                "--",
                "--release",
            ]))
            .unwrap()
            else {
                panic!("wrong command")
            };
            assert_eq!(config.ceiling, version.parse().unwrap());
            assert_eq!(cargo_args, args(&["--release"]));
        }
    }

    #[test]
    fn target_selection_rejects_conflicts_and_malformed_requests() {
        for extra in [
            vec!["--target", "x86_64-unknown-linux-gnu.2.18"],
            vec!["--target", "aarch64-unknown-linux-gnu.2.17"],
            vec!["--target", "x86_64-unknown-linux-gnu."],
            vec!["--target", "x86_64-unknown-linux-gnu.2.16"],
            vec!["--target", "x86_64-unknown-linux-gnu.2.17.extra"],
            vec![
                "--target",
                "x86_64-unknown-linux-gnu",
                "--target",
                "x86_64-unknown-linux-gnu",
            ],
            vec!["--glibc", "2.17"],
        ] {
            let mut argv = vec![
                "doctor",
                "--provider",
                "/sdk/provider.txt",
                "--glibc",
                "2.17",
            ];
            argv.extend(extra);
            assert!(parse(args(&argv)).is_err(), "{argv:?}");
        }
        for target in ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu.2.17"] {
            assert!(parse(args(&[
                "doctor",
                "--provider",
                "/sdk/provider.txt",
                "--glibc",
                "2.17",
                "--target",
                target
            ]))
            .is_ok());
        }
        assert!(parse(args(&[
            "doctor",
            "--provider",
            "/sdk/provider.txt",
            "--target",
            toolchain::TARGET
        ]))
        .is_err());
    }

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn cargo_subcommand_preserves_paths_and_passthrough() {
        let parsed = parse(args(&[
            "libc",
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
            scan,
            json,
        } = parsed
        else {
            panic!("wrong command")
        };
        assert_eq!(file, PathBuf::from("-app"));
        assert_eq!(ceiling, Some("2.22".parse().unwrap()));
        assert!(scan.is_none(), "the scan runs only when asked for");
        assert!(json);
    }

    fn inspect_scan(argv: &[&str]) -> Option<ScanOptions> {
        let ParsedCommand::Inspect { scan, .. } = parse(args(argv)).unwrap() else {
            panic!("wrong command")
        };
        scan
    }

    #[test]
    fn inspect_target_options_select_the_scan() {
        let scan =
            inspect_scan(&["inspect", "app", "--distro", "sles-12-sp5", "--strict"]).unwrap();
        assert_eq!(scan.distro.as_deref(), Some("sles-12-sp5"));
        assert!(scan.strict && !scan.verbose);
        let scan =
            inspect_scan(&["inspect", "app", "--max-glibc", "2.17", "--kernel", "3.10"]).unwrap();
        assert_eq!(scan.kernel.as_deref(), Some("3.10"));
        assert!(inspect_scan(&["inspect", "app", "--max-glibc", "2.17", "--scan"]).is_some());
        for argv in [
            vec![
                "inspect",
                "app",
                "--distro",
                "rhel-7",
                "--max-glibc",
                "2.17",
            ],
            vec!["inspect", "app", "--distro", "rhel-7", "--distro", "rhel-7"],
            vec!["inspect", "app", "--strict"],
            vec!["inspect", "app", "--max-glibc", "2.17", "--verbose"],
            vec!["inspect", "app", "--kernel"],
        ] {
            assert!(parse(args(&argv)).is_err(), "{argv:?}");
        }
    }
}
