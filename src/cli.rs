use crate::{
    report::{self, ScanOptions},
    simulate::{self, Simulation},
    toolchain::{self, Toolchain},
    version::Version,
};
use libc_abi_tools::provider::cache::Mode;
use std::{ffi::OsString, path::PathBuf};

const HELP: &str = r#"cargo-libc — builds for an older glibc, and native ELF inspection

USAGE
  cargo libc build  --target x86_64-unknown-linux-gnu.2.28 --artifact FILE
                    [OPTIONS] -- [CARGO BUILD ARGUMENTS]
  cargo libc doctor --sysroot DIR --gcc-runtime DIR --glibc VERSION [OPTIONS]
  cargo libc doctor (--distro ID | --kernel VERSION) [--runner R] [--image IMAGE]
  cargo libc build  --sysroot DIR --gcc-runtime DIR --glibc VERSION
                    --artifact FILE [OPTIONS] -- [CARGO BUILD ARGUMENTS]
  cargo libc check  --sysroot DIR --gcc-runtime DIR --glibc VERSION
                    [OPTIONS] [-- CARGO CHECK ARGUMENTS]
  cargo libc test   --sysroot DIR --gcc-runtime DIR --glibc VERSION
                    [OPTIONS] [-- CARGO TEST ARGUMENTS]
  cargo libc run    (same as test) [-- CARGO RUN ARGUMENTS]
  cargo libc inspect FILE [--max-glibc VERSION | --distro ID] [--kernel VERSION]
                     [--scan] [--strict] [--verbose] [--json]
  cargo libc provider prepare|list|path|verify|export|import|remove ...

PROVIDERS
Without --provider or --sysroot, build/check/test/run/doctor use a glibc link
provider for the selected release (2.17 through 2.44), prepared on first use and
cached (see `cargo libc provider`). The default mode, source, compiles glibc's
startup objects from the recorded sources with the host's GCC (or
$CARGO_LIBC_PROVIDER_GCC), offline, and verifies them against the qualified build;
it refuses rather than use objects that differ. container runs the pinned
container build (podman or docker).

BUILD / DOCTOR OPTIONS
  --provider-mode M  source (default) or container, for the automatic provider
                     (also CARGO_LIBC_PROVIDER_MODE); prebuilt awaits a legal review
  --provider FILE    glibc provider descriptor (see docs/PROVIDER-CONTRACT.md) instead of the automatic provider
  --sysroot DIR      Target development filesystem, including glibc and GCC runtime
  --gcc-runtime DIR  Absolute GCC runtime directory inside that sysroot
  --glibc VERSION    Maximum permitted glibc version, at least 2.17 (e.g. 2.22)
  --target TARGET    x86_64-unknown-linux-gnu[.GLIBC]; suffix supplies --glibc
  --cc FILE          Host Clang driver (default: /usr/bin/clang)
  --ld FILE          Host native linker (default: /usr/bin/ld.bfd)
  --artifact FILE    Exact executable from this Cargo build to inspect (build only)
  --distro ID        Target preset (e.g. sles-12-sp5): supplies --glibc and the
                     kernel floor (build: tier 1 scan; test/run: simulation)
  --kernel VERSION   Kernel floor without a preset
  --strict           Scan warnings also fail (build only)
  --runner R         local or container (test/run/doctor; default: container
                     when an image is known, else local)
  --image IMAGE      Container image (test/run/doctor; default: the preset's)
  --dry-run          Probe target inputs and print the command without building

test/run use the libc-runner helper as Cargo's target runner. It refuses a
binary that needs a newer glibc than --glibc, then runs it under the kernel
simulation (tier 2): the local seccomp filter keeps the host userland and needs
no root; the container uses the image's userland (podman or docker, chosen by
CONTAINER_ENGINE). Neither replaces testing on a real target host (tier 3).

check type-checks with the same target selection and flags as build, so host
build scripts and procedural macros stay native. It does not link, so it
proves nothing about the final executable's glibc or kernel requirements.

doctor with --distro or --kernel also checks both runners: it installs the
filter and runs /bin/sh under it, and starts `true` in the image under the
profile (which may pull the image). It exits with 2 when the runner test/run
would choose is not usable. Without --sysroot/--provider it checks runners only.

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
With --sysroot it uses that existing target sysroot; --glibc does not create or
downgrade one. It uses a Rust linker helper, a host Clang driver, and the selected
native linker. It does not use Zig, Python, or third-party Rust crates.

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
    /// Prepare (or reuse) a provider in this mode: no provider or sysroot given.
    auto: Option<Mode>,
    sysroot: PathBuf,
    gcc_runtime: PathBuf,
    cc: PathBuf,
    ld: PathBuf,
    ceiling: Version,
}

impl RawConfig {
    fn resolve(self) -> Result<Toolchain, String> {
        if let Some(mode) = self.auto {
            let release = self.ceiling.to_string();
            let gcc = std::env::var_os("CARGO_LIBC_PROVIDER_GCC").map(PathBuf::from);
            let prepared = libc_abi_tools::provider::cli::ensure(&release, mode, gcc)
                .map_err(|e| format!("cannot prepare the glibc {release} provider: {e}"))?;
            let descriptor = prepared.entry.descriptor();
            eprintln!(
                "glibc {release} provider ({mode} mode, {}): {}",
                if prepared.reused {
                    "cached"
                } else {
                    "prepared"
                },
                descriptor.display()
            );
            for note in &prepared.notes {
                eprintln!("  {note}");
            }
            return Toolchain::from_provider(&descriptor, self.cc, self.ld, self.ceiling);
        }
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
    Doctor {
        config: Option<RawConfig>,
        simulation: Simulation,
    },
    Build {
        config: RawConfig,
        artifact: PathBuf,
        scan: Option<ScanOptions>,
        dry_run: bool,
        cargo_args: Vec<OsString>,
    },
    Check {
        config: RawConfig,
        dry_run: bool,
        cargo_args: Vec<OsString>,
    },
    Test {
        config: RawConfig,
        subcommand: &'static str,
        simulation: Simulation,
        dry_run: bool,
        cargo_args: Vec<OsString>,
    },
    Inspect {
        file: PathBuf,
        ceiling: Option<Version>,
        scan: Option<ScanOptions>,
        json: bool,
    },
    /// `cargo libc provider ...`: the arguments after `provider`.
    Provider(Vec<OsString>),
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
        Some("provider") => return Ok(ParsedCommand::Provider(args[1..].to_vec())),
        Some("doctor" | "build" | "check" | "test" | "run") => {}
        _ => {
            return Err(format!(
                "unknown command {command:?}; run cargo libc --help"
            ))
        }
    }
    let subcommand: &'static str = match command.to_str() {
        Some("build") => "build",
        Some("check") => "check",
        Some("test") => "test",
        Some("run") => "run",
        _ => "doctor",
    };
    let build = subcommand == "build";
    let cargo_command = subcommand != "doctor";
    let runs = matches!(subcommand, "test" | "run");
    let mut simulation = Simulation::default();
    let mut strict = false;
    let mut provider = None;
    let mut mode = None;
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
            Some("--provider-mode") => {
                let value = next_value(&args, &mut i, "--provider-mode")?;
                let parsed = value
                    .to_str()
                    .ok_or("--provider-mode must be UTF-8")?
                    .parse::<Mode>()?;
                if mode.replace(parsed).is_some() {
                    return Err("duplicate --provider-mode".into());
                }
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
            Some(flag @ ("--distro" | "--kernel")) => {
                let value = next_value(&args, &mut i, flag)?
                    .into_string()
                    .map_err(|_| format!("{flag} must be UTF-8"))?;
                let slot = if flag == "--distro" {
                    &mut simulation.distro
                } else {
                    &mut simulation.kernel
                };
                if slot.replace(value).is_some() {
                    return Err(format!("duplicate {flag}"));
                }
            }
            Some("--strict") if build => strict = true,
            Some("--runner") if runs || !cargo_command => {
                let value = next_value(&args, &mut i, "--runner")?;
                let runner = kernel_abi_tools::Runner::parse(
                    value.to_str().ok_or("--runner must be UTF-8")?,
                )?;
                if simulation.runner.replace(runner).is_some() {
                    return Err("duplicate --runner".into());
                }
            }
            Some("--image") if runs || !cargo_command => {
                let value = next_value(&args, &mut i, "--image")?
                    .into_string()
                    .map_err(|_| "--image must be UTF-8".to_string())?;
                if simulation.image.replace(value).is_some() {
                    return Err("duplicate --image".into());
                }
            }
            Some("--dry-run") if cargo_command => dry_run = true,
            Some("--") if cargo_command => {
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
    let explicit = provider.is_some() || sysroot.is_some() || gcc_runtime.is_some();
    if mode.is_some() && explicit {
        return Err(
            "--provider-mode selects the automatic provider; it conflicts with --provider, --sysroot and --gcc-runtime"
                .into(),
        );
    }
    if let Some(Some(version)) = target {
        if ceiling.is_some_and(|explicit| explicit != version) {
            return Err("--target version conflicts with --glibc".into());
        }
        ceiling = Some(version);
    }
    if simulation.distro.is_some() && simulation.kernel.is_some() {
        return Err("--distro sets the kernel floor; do not combine it with --kernel".into());
    }
    if let Some(id) = &simulation.distro {
        if ceiling.is_some() {
            return Err(format!(
                "--distro {id} sets the glibc version; do not combine it with --glibc or a versioned --target"
            ));
        }
        let preset = linux_targets::distro(id).ok_or_else(|| format!("unknown distro {id:?}"))?;
        ceiling = Some(parse_version(preset.glibc.to_string().into(), "--distro")?);
    }
    // doctor --distro/--kernel without toolchain inputs checks the runners only.
    if !cargo_command
        && !explicit
        && mode.is_none()
        && (simulation.distro.is_some() || simulation.kernel.is_some())
    {
        simulation.plan()?;
        return Ok(ParsedCommand::Doctor {
            config: None,
            simulation,
        });
    }
    let auto = if explicit {
        None
    } else {
        Some(match mode {
            Some(mode) => mode,
            None => match std::env::var("CARGO_LIBC_PROVIDER_MODE") {
                Ok(value) => value
                    .parse()
                    .map_err(|e| format!("CARGO_LIBC_PROVIDER_MODE: {e}"))?,
                Err(_) => Mode::Source,
            },
        })
    };
    let sysroot_mode = provider.is_none() && auto.is_none();
    let config = RawConfig {
        provider: provider.clone(),
        auto,
        sysroot: if sysroot_mode {
            sysroot.ok_or("--sysroot is required")?
        } else {
            PathBuf::new()
        },
        gcc_runtime: if sysroot_mode {
            gcc_runtime.ok_or("--gcc-runtime is required")?
        } else {
            PathBuf::new()
        },
        cc,
        ld,
        ceiling: ceiling.ok_or("--glibc is required")?,
    };
    if build {
        toolchain::validate_cargo_args(&cargo_args)?;
        let scan =
            (simulation.distro.is_some() || simulation.kernel.is_some()).then(|| ScanOptions {
                distro: simulation.distro.clone(),
                kernel: simulation.kernel.clone(),
                strict,
                verbose: false,
            });
        if strict && scan.is_none() {
            return Err("--strict applies to the scan; add --distro or --kernel".into());
        }
        Ok(ParsedCommand::Build {
            config,
            artifact: artifact.ok_or(
                "--artifact is required; name the exact executable produced by this build",
            )?,
            scan,
            dry_run,
            cargo_args,
        })
    } else if subcommand == "check" {
        if simulation.kernel.is_some() {
            return Err(
                "check does not link or run anything; --kernel applies to build, test, run, and doctor"
                    .into(),
            );
        }
        toolchain::validate_cargo_args(&cargo_args)?;
        Ok(ParsedCommand::Check {
            config,
            dry_run,
            cargo_args,
        })
    } else if runs {
        toolchain::validate_cargo_args(&cargo_args)?;
        simulation.plan()?;
        Ok(ParsedCommand::Test {
            config,
            subcommand,
            simulation,
            dry_run,
            cargo_args,
        })
    } else {
        simulation.plan()?;
        Ok(ParsedCommand::Doctor {
            config: Some(config),
            simulation,
        })
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
        ParsedCommand::Doctor { config, simulation } => {
            if let Some(raw) = config {
                let config = raw.resolve()?;
                config.print_diagnostics(&config.diagnose()?);
            }
            let runner_ok = simulate::doctor(&simulation)?;
            Ok(if runner_ok { 0 } else { 2 })
        }
        ParsedCommand::Build {
            config,
            artifact,
            scan,
            dry_run,
            cargo_args,
        } => toolchain::build(
            &config.resolve()?,
            &cargo_args,
            &artifact,
            scan.as_ref(),
            dry_run,
        ),
        ParsedCommand::Check {
            config,
            dry_run,
            cargo_args,
        } => toolchain::check(&config.resolve()?, &cargo_args, dry_run),
        ParsedCommand::Test {
            config,
            subcommand,
            simulation,
            dry_run,
            cargo_args,
        } => toolchain::test_or_run(
            &config.resolve()?,
            subcommand,
            &cargo_args,
            &simulation,
            dry_run,
        ),
        ParsedCommand::Inspect {
            file,
            ceiling,
            scan,
            json,
        } => report::inspect_file(&file, ceiling, scan.as_ref(), json),
        ParsedCommand::Provider(args) => {
            if args.is_empty() || matches!(args[0].to_str(), Some("-h" | "--help" | "help")) {
                print!("{}", libc_abi_tools::provider::cli::help("cargo libc"));
                return Ok(0);
            }
            libc_abi_tools::provider::cli::run(&args, "cargo libc")?;
            Ok(0)
        }
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
        let ParsedCommand::Doctor {
            config: Some(config),
            ..
        } = parse(args(&base)).unwrap()
        else {
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
        for &version in crate::provider::RELEASES {
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

    #[test]
    fn without_provider_or_sysroot_a_provider_is_prepared_automatically() {
        let build = |extra: &[&str]| {
            let mut argv = vec![
                "build",
                "--target",
                "x86_64-unknown-linux-gnu.2.28",
                "--artifact",
                "app",
            ];
            argv.extend(extra);
            parse(args(&argv))
        };
        let ParsedCommand::Build { config, .. } = build(&[]).unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(config.auto, Some(Mode::Source));
        assert_eq!(config.ceiling, "2.28".parse().unwrap());
        let ParsedCommand::Build { config, .. } = build(&["--provider-mode", "container"]).unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(config.auto, Some(Mode::Container));
        let ParsedCommand::Build { config, .. } = build(&["--provider", "/p.txt"]).unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(config.auto, None);
        for extra in [
            &["--provider-mode", "binary"][..],
            &["--provider-mode", "source", "--provider-mode", "source"],
            &["--provider-mode", "source", "--provider", "/p.txt"],
            &["--provider-mode", "source", "--sysroot", "/sdk"],
            &["--sysroot", "/sdk"],
        ] {
            assert!(build(extra).is_err(), "{extra:?}");
        }
        let ParsedCommand::Provider(rest) = parse(args(&["libc", "provider", "list"])).unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(rest, args(&["list"]));
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

    const SDK: [&str; 4] = ["--sysroot", "/sdk", "--gcc-runtime", "/sdk/gcc"];

    fn with_sdk(command: &str, extra: &[&str]) -> Vec<OsString> {
        let mut argv = vec![command];
        argv.extend(SDK);
        argv.extend(extra);
        args(&argv)
    }

    #[test]
    fn distro_supplies_the_glibc_version_and_excludes_explicit_ones() {
        let ParsedCommand::Build { config, scan, .. } = parse(with_sdk(
            "build",
            &["--distro", "sles-12-sp5", "--strict", "--artifact", "app"],
        ))
        .unwrap() else {
            panic!("wrong command")
        };
        assert_eq!(config.ceiling, "2.22".parse().unwrap());
        let scan = scan.unwrap();
        assert!(scan.strict && scan.distro.as_deref() == Some("sles-12-sp5"));
        let ParsedCommand::Build { scan, .. } =
            parse(with_sdk("build", &["--glibc", "2.17", "--artifact", "app"])).unwrap()
        else {
            panic!("wrong command")
        };
        assert!(scan.is_none(), "no scan without a target");
        for extra in [
            vec!["--distro", "rhel-7", "--glibc", "2.17", "--artifact", "app"],
            vec![
                "--distro",
                "rhel-7",
                "--target",
                "x86_64-unknown-linux-gnu.2.17",
                "--artifact",
                "app",
            ],
            vec![
                "--distro",
                "rhel-7",
                "--kernel",
                "3.10",
                "--artifact",
                "app",
            ],
            vec!["--distro", "no-such", "--artifact", "app"],
            vec!["--glibc", "2.17", "--strict", "--artifact", "app"],
            vec!["--glibc", "2.17", "--runner", "local", "--artifact", "app"],
        ] {
            assert!(parse(with_sdk("build", &extra)).is_err(), "{extra:?}");
        }
    }

    #[test]
    fn test_and_run_carry_the_simulation_and_cargo_arguments() {
        let ParsedCommand::Test {
            config,
            subcommand,
            simulation,
            cargo_args,
            ..
        } = parse(with_sdk(
            "test",
            &[
                "--distro",
                "rhel-7",
                "--runner",
                "local",
                "--",
                "--release",
                "--lib",
            ],
        ))
        .unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(subcommand, "test");
        assert_eq!(config.ceiling, "2.17".parse().unwrap());
        assert_eq!(simulation.runner, Some(kernel_abi_tools::Runner::Local));
        assert_eq!(cargo_args, args(&["--release", "--lib"]));
        let ParsedCommand::Test {
            subcommand,
            simulation,
            ..
        } = parse(with_sdk(
            "run",
            &["--glibc", "2.17", "--kernel", "3.10", "--image", "centos:7"],
        ))
        .unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(subcommand, "run");
        assert_eq!(simulation.image.as_deref(), Some("centos:7"));
        for extra in [
            vec!["--glibc", "2.17", "--runner", "local"],
            vec![
                "--glibc",
                "2.17",
                "--kernel",
                "3.10",
                "--runner",
                "container",
            ],
            vec!["--glibc", "2.17", "--kernel", "3.10", "--runner", "vm"],
            vec!["--glibc", "2.17", "--strict"],
            vec!["--glibc", "2.17", "--artifact", "app"],
            vec!["--glibc", "2.17", "--", "--config", "x=1"],
        ] {
            assert!(parse(with_sdk("test", &extra)).is_err(), "{extra:?}");
        }
    }

    #[test]
    fn check_uses_the_build_selection_without_an_artifact() {
        let ParsedCommand::Check {
            config,
            dry_run,
            cargo_args,
        } = parse(with_sdk(
            "check",
            &[
                "--target",
                "x86_64-unknown-linux-gnu.2.17",
                "--dry-run",
                "--",
                "--workspace",
                "--all-targets",
            ],
        ))
        .unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(config.ceiling, "2.17".parse().unwrap());
        assert!(dry_run);
        assert_eq!(cargo_args, args(&["--workspace", "--all-targets"]));
        let ParsedCommand::Check { config, .. } =
            parse(with_sdk("check", &["--distro", "sles-12-sp5"])).unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(config.ceiling, "2.22".parse().unwrap());
        for extra in [
            vec!["--glibc", "2.17", "--artifact", "app"],
            vec!["--glibc", "2.17", "--kernel", "3.10"],
            vec!["--glibc", "2.17", "--strict"],
            vec!["--glibc", "2.17", "--runner", "local"],
            vec!["--glibc", "2.17", "--image", "centos:7"],
            vec!["--glibc", "2.17", "--target", "aarch64-unknown-linux-gnu"],
            vec![
                "--glibc",
                "2.17",
                "--target",
                "x86_64-unknown-linux-gnu.2.18",
            ],
            vec![
                "--glibc",
                "2.17",
                "--",
                "--target",
                "x86_64-unknown-linux-gnu",
            ],
            vec!["--glibc", "2.17", "--", "--message-format=json"],
            vec!["--glibc", "2.16"],
            vec![],
        ] {
            assert!(parse(with_sdk("check", &extra)).is_err(), "{extra:?}");
        }
    }

    #[test]
    fn doctor_checks_runners_with_or_without_toolchain_inputs() {
        let ParsedCommand::Doctor { config, simulation } =
            parse(args(&["doctor", "--kernel", "3.10", "--runner", "local"])).unwrap()
        else {
            panic!("wrong command")
        };
        assert!(config.is_none(), "runner checks only");
        assert_eq!(simulation.kernel.as_deref(), Some("3.10"));
        let ParsedCommand::Doctor { config, simulation } =
            parse(with_sdk("doctor", &["--distro", "sles-12-sp5"])).unwrap()
        else {
            panic!("wrong command")
        };
        assert_eq!(config.unwrap().ceiling, "2.22".parse().unwrap());
        assert_eq!(simulation.distro.as_deref(), Some("sles-12-sp5"));
        for argv in [
            vec!["doctor", "--runner", "local"],
            vec!["doctor", "--kernel", "3.10", "--runner", "container"],
            vec!["doctor", "--image", "centos:7"],
            vec!["doctor"],
        ] {
            assert!(parse(args(&argv)).is_err(), "{argv:?}");
        }
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
