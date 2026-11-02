//! Private target clock-link profile integration. The producer owns ELF changes
//! and the content-addressed overlay cache; this module never modifies a DSO.
use crate::provider::Provider;
use std::{
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) const POLICY: &str = "cargo-libc-clock-libc-v1";
pub(crate) const LIBC_ENV: &str = "CARGO_LIBC_CLOCK_LIBC";
pub(crate) const RT_ENV: &str = "CARGO_LIBC_CLOCK_RT";
pub(crate) const OVERLAY_ENV: &str = "CARGO_LIBC_CLOCK_OVERLAY";
pub(crate) const IDENTITY_ENV: &str = "CARGO_LIBC_CLOCK_IDENTITY";

#[derive(Clone, Debug)]
pub(crate) struct Libraries {
    pub(crate) libc: PathBuf,
    pub(crate) rt: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct Profile {
    pub(crate) libraries: Libraries,
    pub(crate) overlay: Option<PathBuf>,
    pub(crate) identity: String,
}

fn contained_library(path: &Path, root: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "target clock library must have an absolute path: {}",
            path.display()
        ));
    }
    let actual = fs::canonicalize(path).map_err(|e| {
        format!(
            "cannot resolve target clock library {}: {e}",
            path.display()
        )
    })?;
    let root =
        fs::canonicalize(root).map_err(|e| format!("cannot resolve target library root: {e}"))?;
    if !actual.starts_with(&root) || !actual.is_file() {
        return Err(format!("target clock library {} resolves outside selected root {}; provide a self-contained target sysroot/provider (host fallback is forbidden)", path.display(), root.display()));
    }
    crate::toolchain::text_path(&actual)?;
    Ok(actual)
}

/// Clang discovery is only a candidate locator. The producer must validate the
/// pair's architecture, SONAMEs and versioned clock exports before deriving it.
pub(crate) fn discover(
    provider: Option<&Provider>,
    sysroot: &Path,
    mut probe_command: impl FnMut() -> Command,
) -> Result<Libraries, String> {
    let locate = |name: &str, probe_command: &mut dyn FnMut() -> Command| {
        if let Some(provider) = provider {
            return contained_library(&provider.stubs.join(name), &provider.stubs);
        }
        let output = probe_command()
            .arg(format!("-print-file-name={name}"))
            .output()
            .map_err(|e| format!("cannot probe target clock library {name}: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "Clang failed while finding target {name}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let value = std::str::from_utf8(&output.stdout)
            .map_err(|_| format!("Clang returned a non-UTF-8 path for {name}"))?
            .trim();
        if value.is_empty() || value == name || value.contains(['\n', '\r']) {
            return Err(format!("Clang did not locate target {name}; provide a complete target sysroot, not host search paths"));
        }
        contained_library(Path::new(value), sysroot)
    };
    Ok(Libraries {
        libc: locate("libc.so.6", &mut probe_command)?,
        rt: locate("librt.so.1", &mut probe_command)?,
    })
}

impl Profile {
    pub(crate) fn prepare(libraries: Libraries) -> Result<Self, String> {
        let cache = libc_abi_tools::provider::cache::default_root()?.join("link-profiles");
        let prepared = producer_overlay(&libraries, &cache)?;
        if let Some(path) = &prepared.search_dir {
            crate::toolchain::text_path(path)?;
        }
        let identity = format!(
            "{}:{}:{}",
            prepared.profile_id, prepared.libc_sha256, prepared.librt_sha256
        );
        Ok(Self {
            libraries,
            overlay: prepared.search_dir,
            identity,
        })
    }

    pub(crate) fn prefix(&self, command: &mut Command) {
        if let Some(overlay) = &self.overlay {
            let mut arg = OsString::from("-L");
            arg.push(overlay);
            command.arg(arg);
        }
    }

    /// Only substitutes an identity-checked RT file, at the same argument slot.
    /// Opaque response files and scripts are refused until their bounded input
    /// validation is implemented; they must not bypass the selected profile.
    pub(crate) fn link_args(&self, args: Vec<OsString>) -> Result<Vec<OsString>, String> {
        let mut result = Vec::with_capacity(args.len());
        let mut value_for_driver_option = false;
        let mut value_for_linker_option = false;
        let mut forwarded_operand = false;
        for arg in args {
            if value_for_driver_option {
                result.push(arg);
                value_for_driver_option = false;
                continue;
            }
            let text = arg
                .to_str()
                .ok_or("target linker arguments must be UTF-8 for clock-profile validation")?;
            if forwarded_operand {
                result.push(
                    self.link_operand(text, &mut value_for_linker_option)?
                        .into(),
                );
                forwarded_operand = false;
            } else if text == "-Xlinker" {
                result.push(arg);
                forwarded_operand = true;
            // These operands are not linker inputs (notably an output lib.so).
            } else if [
                "-o",
                "-L",
                "-I",
                "-isystem",
                "-include",
                "-isysroot",
                "-x",
                "-MF",
                "-MT",
                "-MQ",
                "-D",
                "-U",
            ]
            .contains(&text)
            {
                result.push(arg);
                value_for_driver_option = true;
            } else if let Some(forwarded) = text.strip_prefix("-Wl,") {
                let values = forwarded
                    .split(',')
                    .map(|value| -> Result<String, String> {
                        let replacement = self.link_operand(value, &mut value_for_linker_option)?;
                        if replacement.contains(',') {
                            return Err("clock overlay path cannot contain a comma when forwarded with -Wl; select a cache path without commas".into());
                        }
                        Ok(replacement)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                result.push(format!("-Wl,{}", values.join(",")).into());
            } else {
                result.push(
                    self.link_operand(text, &mut value_for_linker_option)?
                        .into(),
                );
            }
        }
        if forwarded_operand || value_for_linker_option || value_for_driver_option {
            return Err(
                "missing linker/driver option operand during clock-profile validation".into(),
            );
        }
        Ok(result)
    }

    fn link_operand(&self, value: &str, option_value: &mut bool) -> Result<String, String> {
        if *option_value {
            *option_value = false;
            return Ok(value.into());
        }
        if [
            "-o",
            "-L",
            "--library-path",
            "-soname",
            "--soname",
            "-h",
            "--dynamic-linker",
            "-rpath",
            "--rpath",
            "-rpath-link",
            "-z",
            "-u",
            "--undefined",
            "-e",
            "--entry",
            "--defsym",
        ]
        .contains(&value)
        {
            *option_value = true;
            return Ok(value.into());
        }
        if value.starts_with('@')
            || value == "-T"
            || value.starts_with("-T")
            || value == "--script"
            || value.starts_with("--script=")
        {
            return Err(format!("cannot verify clock policy through opaque linker input {value:?}; use direct target ELF/library arguments until bounded response/script handling is available"));
        }
        if value
            .strip_prefix("-l:")
            .is_some_and(|name| name.contains('/'))
        {
            return Err(format!("ambiguous library-name path {value:?}; pass the selected target RT ELF as a direct path for identity-checked substitution"));
        }
        if value.starts_with('-') {
            return Ok(value.into());
        }
        let path = Path::new(value);
        if !path.is_file() {
            return Ok(value.into());
        }
        let canonical = fs::canonicalize(path)
            .map_err(|e| format!("cannot identify linker input {value:?}: {e}"))?;
        if canonical == self.libraries.rt {
            return self.overlay.as_ref().map_or_else(
                || Ok(value.into()),
                |overlay| {
                    let replacement = overlay.join("librt.so.1");
                    crate::toolchain::text_path(&replacement).map(str::to_owned)
                },
            );
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let mut header = Vec::new();
        fs::File::open(&canonical)
            .and_then(|file| file.take(18).read_to_end(&mut header))
            .map_err(|e| format!("cannot inspect linker input {value:?}: {e}"))?;
        let dynamic_elf = header.len() == 18
            && &header[..6] == b"\x7fELF\x02\x01"
            && u16::from_le_bytes([header[16], header[17]]) == 3;
        if name.ends_with(".ld") || name.ends_with(".lds") {
            return Err(format!("cannot verify clock policy through linker script {value:?}; use direct target ELF/library arguments"));
        }
        if dynamic_elf || name.ends_with(".so") || name.contains(".so.") {
            let metadata = libc_abi_tools::elf::Metadata::load(&canonical)
                .map_err(|e| format!("cannot verify explicit shared-library input {value:?}: {e}; linker scripts and ambiguous RT inputs are unsupported"))?;
            if metadata.soname == "librt.so.1" {
                return Err(format!("explicit RT input {value:?} is not the selected target librt {}; select a matching provider/sysroot instead of bypassing its clock profile", self.libraries.rt.display()));
            }
        }
        Ok(value.into())
    }

    pub(crate) fn variables(&self) -> Vec<(&'static str, OsString)> {
        vec![
            (LIBC_ENV, self.libraries.libc.clone().into_os_string()),
            (RT_ENV, self.libraries.rt.clone().into_os_string()),
            (IDENTITY_ENV, self.identity.clone().into()),
            // Empty explicitly clears an inherited overlay for a compliant pair.
            (
                OVERLAY_ENV,
                self.overlay
                    .as_ref()
                    .map_or_else(OsString::new, |p| p.as_os_str().to_owned()),
            ),
        ]
    }

    pub(crate) fn from_environment(
        provider: Option<&Provider>,
        sysroot: &Path,
    ) -> Result<Self, String> {
        let required = |key| {
            std::env::var_os(key)
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .ok_or_else(|| {
                    format!(
                        "{key} is missing; regenerate the target environment with cargo libc env"
                    )
                })
        };
        let root = provider.map_or(sysroot, |p| p.stubs.as_path());
        let libraries = Libraries {
            libc: contained_library(&required(LIBC_ENV)?, root)?,
            rt: contained_library(&required(RT_ENV)?, root)?,
        };
        if let Some(provider) = provider {
            if libraries.libc != contained_library(&provider.stubs.join("libc.so.6"), root)?
                || libraries.rt != contained_library(&provider.stubs.join("librt.so.1"), root)?
            {
                return Err("clock library inputs do not match the selected provider; regenerate the target environment with cargo libc env".into());
            }
        }
        let profile = Self::prepare(libraries)?;
        let exported = std::env::var_os(OVERLAY_ENV)
            .ok_or("clock overlay selection is missing; regenerate the target environment with cargo libc env")?;
        if exported
            != profile
                .overlay
                .as_ref()
                .map_or_else(OsString::new, |p| p.as_os_str().to_owned())
        {
            return Err("clock overlay no longer matches selected target libraries; regenerate the target environment with cargo libc env".into());
        }
        if std::env::var_os(IDENTITY_ENV).as_deref()
            != Some(std::ffi::OsStr::new(&profile.identity))
        {
            return Err("clock input identity changed; regenerate the target environment with cargo libc env".into());
        }
        Ok(profile)
    }
}

struct Prepared {
    search_dir: Option<PathBuf>,
    libc_sha256: String,
    librt_sha256: String,
    profile_id: &'static str,
}

/// Temporary fail-closed adapter until the exact producer source is available.
/// Replace with provider::link_profile::prepare(libc, rt, cache_root,
/// LinkProfile::LibcClocksV1), mapping Overlay's search_dir, both raw hashes and
/// profile.id(). ELF transformation/cache code must remain in the producer.
fn producer_overlay(libraries: &Libraries, _cache_root: &Path) -> Result<Prepared, String> {
    Err(format!("clock profile producer API integration is pending for {} and {}; this scaffolding is not a release candidate", libraries.libc.display(), libraries.rt.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_prefix_preserves_each_caller_argument_and_state() {
        let profile = Profile {
            libraries: Libraries {
                libc: "/target/libc.so.6".into(),
                rt: "/target/librt.so.1".into(),
            },
            overlay: Some("/cache with spaces/clock".into()),
            identity: "libc-clocks-v1:libc-hash:rt-hash".into(),
        };
        let caller = [
            "--push-state",
            "--no-as-needed",
            "--start-group",
            "-lrt",
            "-lc",
            "--end-group",
            "--pop-state",
        ];
        let mut command = Command::new("clang");
        profile.prefix(&mut command);
        command.args(caller);
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args[0], "-L/cache with spaces/clock");
        assert_eq!(&args[1..], caller.map(std::ffi::OsStr::new));
    }

    #[test]
    fn pending_adapter_cannot_silently_use_old_clock_defaults() {
        assert!(Profile::prepare(Libraries {
            libc: "/target/libc.so.6".into(),
            rt: "/target/librt.so.1".into()
        })
        .unwrap_err()
        .contains("not a release candidate"));
    }

    struct Files(PathBuf);

    impl Files {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("clock-input-test-{}-{stamp}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Files {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn explicit_rt_replacement_stays_in_its_original_argument_slot() {
        let files = Files::new();
        let rt = files.0.join("librt.so.1");
        fs::write(&rt, []).unwrap();
        let profile = Profile {
            libraries: Libraries {
                libc: files.0.join("libc.so.6"),
                rt: fs::canonicalize(&rt).unwrap(),
            },
            overlay: Some(files.0.join("derived")),
            identity: "test-only".into(),
        };
        let rt = rt.to_str().unwrap();
        let replacement = files
            .0
            .join("derived/librt.so.1")
            .to_str()
            .unwrap()
            .to_owned();
        let args = [
            "-Wl,--push-state,--no-as-needed",
            rt,
            "-lc",
            "-Wl,--pop-state",
            "-Xlinker",
            rt,
            "-o",
            "existing-output.so",
        ];
        let result = profile
            .link_args(args.map(OsString::from).to_vec())
            .unwrap();
        let mut expected = args.map(OsString::from);
        expected[1] = replacement.clone().into();
        expected[5] = replacement.clone().into();
        assert_eq!(result, expected);
        assert_eq!(
            profile
                .link_args(vec![
                    format!("-Wl,--start-group,{rt},-lc,--end-group").into()
                ])
                .unwrap(),
            vec![OsString::from(format!(
                "-Wl,--start-group,{replacement},-lc,--end-group"
            ))]
        );
        for input in [
            "@unknown.rsp",
            "-Wl,@unknown.rsp",
            "-Wl,--script=unknown.ld",
        ] {
            assert!(profile
                .link_args(vec![input.into()])
                .unwrap_err()
                .contains("cannot verify clock policy"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn discovery_rejects_a_target_symlink_that_escapes_to_host_files() {
        use std::os::unix::fs::symlink;
        let files = Files::new();
        let root = files.0.join("target");
        fs::create_dir(&root).unwrap();
        let host = files.0.join("host-libc.so.6");
        fs::write(&host, []).unwrap();
        let link = root.join("libc.so.6");
        symlink(&host, &link).unwrap();
        assert!(contained_library(&link, &root)
            .unwrap_err()
            .contains("host fallback is forbidden"));
        fs::remove_file(&link).unwrap();
        let actual = root.join("actual-libc.so.6");
        fs::write(&actual, []).unwrap();
        symlink(&actual, &link).unwrap();
        assert_eq!(
            contained_library(&link, &root).unwrap(),
            fs::canonicalize(actual).unwrap()
        );
        assert!(contained_library(Path::new("libc.so.6"), &root).is_err());
    }
}
