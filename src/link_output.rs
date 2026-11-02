//! Final-link output classification, independent of Cargo's subcommand or file
//! suffix. Intermediate libraries can feed executables and DSOs in one build.
use std::ffi::OsString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Executable,
    Shared,
    Relocatable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) kind: Kind,
    pub(crate) pie: bool,
    /// Executable selection after a shared control can leave compiler-selected
    /// DSO CRT inputs. Keep the actual kind, but refuse that startup ambiguity.
    pub(crate) overrides_shared: bool,
}

impl Output {
    pub(crate) fn validate_startup_policy(self, provider: bool) -> Result<(), String> {
        if self.overrides_shared {
            return Err("completed linker command selects an executable after shared-library controls; compiler CRT selection is ambiguous, so use consistent direct output controls".into());
        }
        if provider && self.pie {
            return Err("completed linker command selects PIE, but providers support non-PIE executables and shared libraries; remove forwarded PIE controls or use a complete sysroot".into());
        }
        Ok(())
    }
}

/// Consume ONLY the completed linker argv from Clang's -### trace. Driver
/// controls and -Wl/-Xlinker syntax must never be interpreted in this stream.
/// Linker operands retain their positions even when they spell a mode switch.
pub(crate) fn classify(args: &[OsString]) -> Result<Output, String> {
    let mut output = Output {
        kind: Kind::Executable,
        pie: false,
        overrides_shared: false,
    };
    let mut value = false;
    let mut operands_only = false;
    let mut partial = false;
    let mut dynamic_mode = false;
    let mut saw_shared = false;
    for arg in args {
        if value {
            value = false;
            continue;
        }
        if operands_only {
            continue;
        }
        let b = arg.as_encoded_bytes();
        match b {
            b"--" => operands_only = true,
            b"-shared" | b"--shared" | b"-Bshareable" => {
                dynamic_mode = true;
                saw_shared = true;
                output = Output {
                    kind: Kind::Shared,
                    pie: false,
                    overrides_shared: false,
                };
            }
            b"-pie" | b"--pie" | b"--pic-executable" => {
                dynamic_mode = true;
                output = Output {
                    kind: Kind::Executable,
                    pie: true,
                    overrides_shared: saw_shared,
                };
            }
            b"-no-pie" | b"--no-pie" => {
                output = Output {
                    kind: Kind::Executable,
                    pie: false,
                    overrides_shared: saw_shared,
                };
            }
            b"-r" | b"--relocatable" | b"-Ur" => {
                partial = true;
                output = Output {
                    kind: Kind::Relocatable,
                    pie: false,
                    overrides_shared: false,
                };
            }
            b"-Xlinker" => return Err("driver syntax in completed linker arguments".into()),
            _ if b.starts_with(b"-Wl,") || b.starts_with(b"@") => {
                return Err(
                    "unexpanded driver/response syntax in completed linker arguments".into(),
                );
            }
            _ if crate::clock_named::takes_value(b) => value = true,
            _ => {}
        }
    }
    if value {
        return Err("missing operand in completed linker arguments".into());
    }
    if partial && dynamic_mode {
        return Err("ambiguous relocatable/shared/PIE controls in completed linker command".into());
    }
    Ok(output)
}

/// Make the compiler select CRT/default-library inputs for the verified kind.
/// A provider supplies those itself; relocatable outputs never take them.
pub(crate) fn align_driver_startup(command: &mut std::process::Command, output: Output) {
    match output.kind {
        Kind::Relocatable => {
            command.arg("-r");
        }
        Kind::Shared => {
            command.arg("-shared");
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }
    #[test]
    fn linker_operands_never_become_modes() {
        for values in [
            vec!["-pie", "a.o", "-rpath", "-shared"],
            vec!["-pie", "-rpath", "-r", "a.o"],
            vec!["-pie", "-o", "-shared", "a.o"],
            vec!["-pie", "-R", "-shared", "a.o"],
            vec!["-pie", "--just-symbols", "-r", "a.o"],
        ] {
            assert_eq!(
                classify(&args(&values)).unwrap(),
                Output {
                    kind: Kind::Executable,
                    pie: true,
                    overrides_shared: false
                }
            );
        }
    }
    #[test]
    fn actual_linker_mode_order_is_preserved() {
        assert_eq!(
            classify(&args(&["-shared", "a.o", "-pie"])).unwrap(),
            Output {
                kind: Kind::Executable,
                pie: true,
                overrides_shared: true
            }
        );
        assert_eq!(
            classify(&args(&["-pie", "a.o", "-shared"])).unwrap(),
            Output {
                kind: Kind::Shared,
                pie: false,
                overrides_shared: false
            }
        );
        assert_eq!(
            classify(&args(&["-shared", "a.o"])).unwrap().kind,
            Kind::Shared
        );
        assert_eq!(
            classify(&args(&["-r", "a.o"])).unwrap().kind,
            Kind::Relocatable
        );
    }
    #[test]
    fn missing_ambiguous_or_unexpanded_inputs_refuse() {
        for values in [
            vec!["a.o", "-rpath"],
            vec!["-shared", "-r", "a.o"],
            vec!["-pie", "-r", "a.o"],
            vec!["-Wl,-shared"],
            vec!["-Xlinker", "-shared"],
            vec!["@args"],
        ] {
            assert!(classify(&args(&values)).is_err(), "{values:?}");
        }
    }
    #[test]
    fn unsupported_startup_overrides_refuse_without_losing_actual_kind() {
        let overridden = classify(&args(&["-shared", "a.o", "-pie"])).unwrap();
        assert_eq!(overridden.kind, Kind::Executable);
        assert!(overridden.validate_startup_policy(false).is_err());
        let pie = classify(&args(&["-pie", "a.o"])).unwrap();
        assert!(pie.validate_startup_policy(false).is_ok());
        assert!(pie.validate_startup_policy(true).is_err());
        assert!(classify(&args(&["-shared", "a.o"]))
            .unwrap()
            .validate_startup_policy(true)
            .is_ok());
        assert!(classify(&args(&["-r", "a.o"]))
            .unwrap()
            .validate_startup_policy(true)
            .is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn unrelated_non_utf8_inputs_keep_their_bytes() {
        use std::os::unix::ffi::OsStringExt;
        let args = vec![
            OsString::from_vec(b"source\xff.o".to_vec()),
            "-shared".into(),
        ];
        let before = args.clone();
        assert_eq!(classify(&args).unwrap().kind, Kind::Shared);
        assert_eq!(args, before);
    }
    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "explicit native Clang/GCC/readelf controls; set fresh evidence directory"]
    fn native_completed_stream_matches_elf_and_runpath() {
        use std::{
            env, fs,
            path::{Path, PathBuf},
            process::Command,
        };
        let root = PathBuf::from(
            env::var_os("CARGO_LIBC_LINK_OUTPUT_EVIDENCE")
                .expect("set CARGO_LIBC_LINK_OUTPUT_EVIDENCE to a fresh directory"),
        );
        fs::create_dir(&root).expect("evidence directory must be fresh");
        let root = root.canonicalize().unwrap();
        let clang =
            env::var_os("CARGO_LIBC_TEST_CLANG").unwrap_or_else(|| "/usr/bin/clang-16".into());
        let gcc = env::var_os("CARGO_LIBC_TEST_GCC").unwrap_or_else(|| "/usr/bin/gcc".into());
        let linker = PathBuf::from(
            env::var_os("CARGO_LIBC_TEST_LD").unwrap_or_else(|| "/usr/bin/ld.bfd".into()),
        );
        fs::write(root.join("input.c"), "int main(void) { return 0; }\n").unwrap();
        type NativeCase = (
            &'static str,
            &'static [&'static str],
            Kind,
            bool,
            Option<&'static str>,
        );
        let cases: &[NativeCase] = &[
            ("nonpie", &["-no-pie"], Kind::Executable, false, None),
            ("pie", &["-pie"], Kind::Executable, true, None),
            ("shared", &["-shared"], Kind::Shared, false, None),
            (
                "wl-rpath-shared",
                &["-pie", "-Wl,-rpath", "-fPIC", "-Wl,-shared"],
                Kind::Executable,
                true,
                Some("-shared"),
            ),
            (
                "xlinker-rpath-r",
                &["-pie", "-Xlinker", "-rpath", "-fPIC", "-Xlinker", "-r"],
                Kind::Executable,
                true,
                Some("-r"),
            ),
            (
                "xlinker-rpath-shared",
                &["-pie", "-Xlinker", "-rpath", "-fPIC", "-Xlinker", "-shared"],
                Kind::Executable,
                true,
                Some("-shared"),
            ),
            (
                "shared-rpath-pie",
                &["-shared", "-Wl,-rpath", "-fPIC", "-Wl,-pie"],
                Kind::Shared,
                false,
                Some("-pie"),
            ),
            (
                "direct-shared-pie",
                &["-shared", "-pie"],
                Kind::Shared,
                false,
                None,
            ),
            (
                "shared-forwarded-pie",
                &["-shared", "-Wl,-pie"],
                Kind::Executable,
                true,
                None,
            ),
            (
                "pie-forwarded-shared",
                &["-pie", "-Wl,-shared"],
                Kind::Shared,
                false,
                None,
            ),
            ("relocatable", &["-r"], Kind::Relocatable, false, None),
            (
                "forwarded-relocatable",
                &["-no-pie", "-nostdlib", "-Wl,-r"],
                Kind::Relocatable,
                false,
                None,
            ),
            (
                "pie-rpath-pie",
                &["-pie", "-Wl,-rpath", "-fPIC", "-Wl,-pie"],
                Kind::Executable,
                true,
                Some("-pie"),
            ),
        ];
        fn elf(dir: &Path, binary: &Path, kind: Kind, pie: bool, runpath: Option<&str>) {
            let output = Command::new("readelf")
                .env("LC_ALL", "C")
                .args(["-W", "-h", "-l", "-d"])
                .arg(binary)
                .output()
                .unwrap();
            let mut evidence_name = dir.as_os_str().to_owned();
            evidence_name.push(".readelf.txt");
            fs::write(PathBuf::from(evidence_name), &output.stdout).unwrap();
            assert!(output.status.success(), "readelf {binary:?}");
            let bytes = fs::read(binary).unwrap();
            let actual_type = u16::from_le_bytes([bytes[16], bytes[17]]);
            let expected_type = match kind {
                Kind::Relocatable => 1,
                Kind::Shared => 3,
                Kind::Executable if pie => 3,
                Kind::Executable => 2,
            };
            assert_eq!(actual_type, expected_type, "ELF type {binary:?}");
            let text = String::from_utf8(output.stdout).unwrap();
            let actual_pie = text.lines().any(|line| {
                line.contains("(FLAGS_1)") && line.split_whitespace().any(|v| v == "PIE")
            });
            assert_eq!(actual_pie, pie, "PIE flag {binary:?}");
            let paths: Vec<_> = text.lines().filter(|l| l.contains("(RUNPATH)")).collect();
            if let Some(path) = runpath {
                assert_eq!(paths.len(), 1, "{binary:?}");
                assert!(
                    paths[0].contains(&format!("[{path}]")),
                    "{binary:?}: {paths:?}"
                );
            } else {
                assert!(paths.is_empty(), "{binary:?}: {paths:?}");
            }
        }
        for (family, compiler) in [("clang", clang), ("gcc-reference", gcc)] {
            let version = Command::new(&compiler).arg("--version").output().unwrap();
            fs::write(root.join(format!("{family}-version.txt")), version.stdout).unwrap();
            let obj = root.join(format!("{family}.o"));
            let status = Command::new(&compiler)
                .args(["-fPIC", "-c"])
                .arg(root.join("input.c"))
                .arg("-o")
                .arg(&obj)
                .status()
                .unwrap();
            assert!(status.success());
            for &(name, flags, kind, pie, path) in cases {
                let stem = root.join(format!("{family}-{name}"));
                let binary = stem.with_extension("elf");
                let mut command = Command::new(&compiler);
                let linker_arg = if family == "clang" {
                    format!("-fuse-ld={}", linker.display())
                } else {
                    // GCC reference accepts a linker name, not Clang's absolute
                    // -fuse-ld path spelling. It is not a product driver backend.
                    let name = linker.file_name().unwrap().to_str().unwrap();
                    format!(
                        "-fuse-ld={}",
                        name.strip_prefix("ld.")
                            .expect("reference linker name ld.bfd/ld.lld")
                    )
                };
                command
                    .arg(&obj)
                    .args(flags)
                    .arg(linker_arg)
                    .arg("-o")
                    .arg(&binary);
                let trace = Command::new(&compiler)
                    .args(command.get_args())
                    .arg("-###")
                    .output()
                    .unwrap();
                fs::write(stem.with_extension("trace.txt"), &trace.stderr).unwrap();
                assert!(trace.status.success(), "{family} {name}");
                let mut classified = None;
                if family == "clang" {
                    let effective =
                        crate::clock_named::completed_command(&command, &linker).unwrap();
                    fs::write(
                        stem.with_extension("effective.txt"),
                        format!("{effective:#?}\n"),
                    )
                    .unwrap();
                    let actual = classify(&effective[1..]).unwrap();
                    fs::write(
                        stem.with_extension("classification.txt"),
                        format!("{actual:?}\n"),
                    )
                    .unwrap();
                    assert_eq!(actual.kind, kind, "{name}");
                    assert_eq!(actual.pie, pie, "{name}");
                    if let Some(value) = path {
                        assert!(
                            effective
                                .windows(2)
                                .any(|w| w[0] == "-rpath" && w[1] == value),
                            "{name}: {effective:?}"
                        );
                    }
                    classified = Some(actual);
                } else {
                    // GCC is reference evidence only: it does not implement
                    // the supported Clang -### protocol. Record collect2 too.
                    let text = String::from_utf8_lossy(&trace.stderr);
                    let effective = text
                        .lines()
                        .find(|l| l.contains("/collect2 "))
                        .expect("GCC collect2 command");
                    fs::write(stem.with_extension("effective.txt"), effective).unwrap();
                    if let Some(value) = path {
                        assert!(effective.contains(&format!("-rpath {value}")));
                    }
                }
                let output = command.output().unwrap();
                fs::write(stem.with_extension("stdout"), &output.stdout).unwrap();
                fs::write(stem.with_extension("stderr"), &output.stderr).unwrap();
                assert!(
                    output.status.success(),
                    "{family} {name}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let (expected_kind, expected_pie) =
                    if family == "gcc-reference" && name == "direct-shared-pie" {
                        (Kind::Executable, true)
                    } else {
                        (kind, pie)
                    };
                elf(&stem, &binary, expected_kind, expected_pie, path);
                if let Some(actual) = classified {
                    if actual.overrides_shared {
                        assert_eq!(name, "shared-forwarded-pie");
                        assert!(actual.validate_startup_policy(false).is_err()); // real production guard
                        continue;
                    }
                    if actual.kind != Kind::Executable {
                        let aligned_binary = stem.with_extension("aligned.elf");
                        align_driver_startup(&mut command, actual);
                        command.arg("-o").arg(&aligned_binary);
                        let effective =
                            crate::clock_named::completed_command(&command, &linker).unwrap();
                        fs::write(
                            stem.with_extension("aligned-effective.txt"),
                            format!("{effective:#?}\n"),
                        )
                        .unwrap();
                        assert_eq!(classify(&effective[1..]).unwrap(), actual);
                        assert!(!effective.iter().any(|v| matches!(
                            Path::new(v).file_name().and_then(|v| v.to_str()),
                            Some("crt1.o" | "Scrt1.o" | "rcrt1.o")
                        )));
                        if actual.kind == Kind::Shared {
                            assert!(!effective
                                .iter()
                                .any(|v| v == "-dynamic-linker" || v == "--dynamic-linker"));
                        }
                        if actual.kind == Kind::Relocatable {
                            assert!(!effective.iter().any(|v| Path::new(v)
                                .file_name()
                                .is_some_and(|s| s.to_string_lossy().starts_with("crt"))));
                        }
                        let output = command.output().unwrap();
                        fs::write(stem.with_extension("aligned-stderr"), &output.stderr).unwrap();
                        assert!(output.status.success(), "aligned {name}");
                        elf(
                            &stem.with_extension("aligned"),
                            &aligned_binary,
                            kind,
                            pie,
                            path,
                        );
                    }
                }
            }
        }
        fs::write(root.join("PASS.txt"), "26 Clang/GCC-reference ELF/RUNPATH controls and Clang classification/startup assertions passed\n").unwrap();
    }
}
