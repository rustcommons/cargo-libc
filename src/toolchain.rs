use crate::{cargo_output, report, version::Version};
use std::{
    env,
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const LOADER: &str = "/lib64/ld-linux-x86-64.so.2";
const SYSROOT_ENV: &str = "CARGO_GLIBC_SYSROOT";
const GCC_ENV: &str = "CARGO_GLIBC_GCC_RUNTIME";
const CC_ENV: &str = "CARGO_GLIBC_CC";
const LD_ENV: &str = "CARGO_GLIBC_LD";

#[derive(Clone, Debug)]
pub struct Toolchain {
    pub sysroot: PathBuf,
    pub gcc_runtime: PathBuf,
    pub cc: PathBuf,
    pub ld: PathBuf,
    pub ceiling: Version,
}

#[derive(Debug)]
pub struct Diagnostics {
    pub header_version: Version,
    pub compiler_version: String,
    pub linker_version: String,
    pub files: Vec<(String, PathBuf)>,
}

fn canonical(path: &Path, label: &str) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|e| format!("cannot resolve {label} {}: {e}", path.display()))
}

fn inside_file(path: &Path, root: &Path) -> Result<PathBuf, String> {
    let actual = canonical(path, "target file")?;
    if !actual.starts_with(root) {
        return Err(format!(
            "target file {} resolves outside sysroot {}",
            path.display(),
            root.display()
        ));
    }
    if !actual.is_file() {
        return Err(format!(
            "target input is not a regular file: {}",
            actual.display()
        ));
    }
    Ok(actual)
}

pub fn text_path(path: &Path) -> Result<&str, String> {
    let text = path
        .to_str()
        .ok_or("toolchain paths must be UTF-8 for Cargo's encoded flags")?;
    if text.contains('\u{1f}') {
        return Err("toolchain paths cannot contain the Cargo flag separator U+001F".into());
    }
    Ok(text)
}

impl Toolchain {
    pub fn new(
        sysroot: PathBuf,
        gcc_runtime: PathBuf,
        cc: PathBuf,
        ld: PathBuf,
        ceiling: Version,
    ) -> Result<Self, String> {
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return Err("the build coordinator currently supports x86-64 Linux hosts only".into());
        }
        if ceiling.major != 2 {
            return Err("this prototype supports glibc 2.x targets only".into());
        }
        let sysroot = canonical(&sysroot, "sysroot")?;
        let gcc_runtime = canonical(&gcc_runtime, "GCC runtime directory")?;
        if !sysroot.is_dir() || !gcc_runtime.is_dir() {
            return Err("sysroot and GCC runtime must be directories".into());
        }
        if !gcc_runtime.starts_with(&sysroot) {
            return Err("the GCC runtime directory must be inside the supplied sysroot".into());
        }
        let cc = canonical(&cc, "Clang driver")?;
        let ld = canonical(&ld, "linker")?;
        if !cc.is_file() || !ld.is_file() {
            return Err("compiler and linker paths must identify files".into());
        }
        for path in [&sysroot, &gcc_runtime, &cc, &ld] {
            text_path(path)?;
        }
        Ok(Self {
            sysroot,
            gcc_runtime,
            cc,
            ld,
            ceiling,
        })
    }

    pub fn driver_args(&self) -> Vec<OsString> {
        vec![
            format!("--target={TARGET}").into(),
            prefixed_path("--sysroot=", &self.sysroot),
            prefixed_path("--gcc-install-dir=", &self.gcc_runtime),
            "--rtlib=libgcc".into(),
            prefixed_path("-fuse-ld=", &self.ld),
        ]
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.cc);
        cmd.args(self.driver_args());
        remove_search_overrides(&mut cmd);
        cmd
    }

    pub fn diagnose(&self) -> Result<Diagnostics, String> {
        let header = inside_file(&self.sysroot.join("usr/include/features.h"), &self.sysroot)?;
        let text =
            fs::read_to_string(&header).map_err(|e| format!("cannot read features.h: {e}"))?;
        let header_version = header_version(&text)?;
        if header_version > self.ceiling {
            return Err(format!("sysroot headers are glibc {header_version}, newer than requested ceiling {}; supply older target development files", self.ceiling));
        }
        let compiler_version = tool_version(&self.cc)?;
        if !compiler_version.to_ascii_lowercase().contains("clang") {
            return Err(format!("--cc must be a Clang driver supporting --gcc-install-dir; got {compiler_version:?}"));
        }
        let linker_version = tool_version(&self.ld)?;
        let mut files = Vec::new();
        for name in [
            "Scrt1.o",
            "crt1.o",
            "crti.o",
            "crtn.o",
            "crtbeginS.o",
            "crtendS.o",
            "libc.so",
            "libc_nonshared.a",
            "libgcc_s.so",
        ] {
            let output = self
                .command()
                .arg(format!("-print-file-name={name}"))
                .output()
                .map_err(|e| format!("cannot probe target input {name}: {e}"))?;
            if !output.status.success() {
                return Err(format!(
                    "Clang failed while finding {name}: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
            let value = std::str::from_utf8(&output.stdout)
                .map_err(|_| "Clang returned a non-UTF-8 input path")?
                .trim();
            if value == name || value.is_empty() || !Path::new(value).is_absolute() {
                return Err(format!(
                    "Clang did not locate target input {name}; returned {value:?}"
                ));
            }
            let path = inside_file(Path::new(value), &self.sysroot)?;
            if name.ends_with(".o") {
                validate_x86_object(&path)?;
            }
            files.push((name.into(), path));
        }
        // Explicit runtime selection must really select the supplied GCC directory.
        for name in ["crtbeginS.o", "crtendS.o"] {
            let expected = inside_file(&self.gcc_runtime.join(name), &self.sysroot)?;
            let selected = files.iter().find(|(n, _)| n == name).unwrap();
            if selected.1 != expected {
                return Err(format!("Clang selected an unexpected {name}"));
            }
        }
        inside_file(
            &self.sysroot.join(LOADER.trim_start_matches('/')),
            &self.sysroot,
        )?;
        Ok(Diagnostics {
            header_version,
            compiler_version,
            linker_version,
            files,
        })
    }

    pub fn print_diagnostics(&self, info: &Diagnostics) {
        println!(
            "Target: {TARGET}; requested glibc ceiling: {}",
            self.ceiling
        );
        println!(
            "Sysroot: {}; headers: glibc {}",
            self.sysroot.display(),
            info.header_version
        );
        println!("Compiler: {}", info.compiler_version);
        println!("Linker: {}", info.linker_version);
        for (name, path) in &info.files {
            println!("  {name}: {}", path.display());
        }
        println!("Target input probes passed. Full dependency resolution and target runtime behavior remain separate checks.");
    }

    pub fn rustflags(&self, driver: &Path) -> Result<OsString, String> {
        // These input paths deliberately also appear in rustc flags: Cargo then sees
        // profile-path changes in its fingerprints rather than only hidden env vars.
        let flags = vec![
            "-C".to_string(),
            format!("linker={}", text_path(driver)?),
            "-C".into(),
            "linker-flavor=gcc".into(),
            "-C".to_string(),
            "linker-features=-lld".into(),
            "-C".into(),
            "target-cpu=x86-64".into(),
            "-C".into(),
            format!("link-arg=--sysroot={}", text_path(&self.sysroot)?),
            "-C".into(),
            format!(
                "link-arg=--gcc-install-dir={}",
                text_path(&self.gcc_runtime)?
            ),
            "-C".into(),
            format!("link-arg=-fuse-ld={}", text_path(&self.ld)?),
            "--cfg".into(),
            format!("cargo_glibc_profile=\"{}\"", self.profile_fingerprint()?),
        ];
        Ok(flags.join("\u{1f}").into())
    }

    fn profile_fingerprint(&self) -> Result<String, String> {
        // FNV-1a is only a stable cache discriminator, not an integrity check.
        // SDK/tool contents changed in place still require a clean rebuild.
        let mut hash = 0xcbf29ce484222325u64;
        let ceiling = self.ceiling.to_string();
        for item in [
            text_path(&self.sysroot)?,
            text_path(&self.gcc_runtime)?,
            text_path(&self.cc)?,
            text_path(&self.ld)?,
            &ceiling,
        ] {
            for byte in item.bytes().chain(std::iter::once(0)) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        Ok(format!("{hash:016x}"))
    }

    pub fn set_environment(&self, cmd: &mut Command) {
        cmd.env(SYSROOT_ENV, &self.sysroot)
            .env(GCC_ENV, &self.gcc_runtime)
            .env(CC_ENV, &self.cc)
            .env(LD_ENV, &self.ld);
    }
}

pub fn validate_cargo_args(args: &[OsString]) -> Result<(), String> {
    for arg in args {
        let text = arg.to_string_lossy();
        if text == "--target"
            || text.starts_with("--target=")
            || text == "--config"
            || text.starts_with("--config=")
            || text == "--message-format"
            || text.starts_with("--message-format=")
            || text == "--"
            || text.starts_with('+')
        {
            return Err(format!("Cargo argument {text:?} conflicts with this prototype's controlled target configuration"));
        }
    }
    Ok(())
}

pub fn build(
    config: &Toolchain,
    cargo_args: &[OsString],
    artifact: &Path,
    dry_run: bool,
) -> Result<i32, String> {
    validate_cargo_args(cargo_args)?;
    for key in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"] {
        if env::var_os(key).is_some_and(|value| !value.is_empty()) {
            return Err(format!("{key} is already set; use a shell without that override so the target flags are explicit"));
        }
    }
    let diagnostic = config.diagnose()?;
    config.print_diagnostics(&diagnostic);
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    let driver = executable
        .parent()
        .ok_or("cannot locate sibling linker driver")?
        .join("glibc-linker");
    if !driver.is_file() {
        return Err(format!(
            "missing Rust linker driver {}; install/build both binaries",
            driver.display()
        ));
    }
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("build")
        .arg("--target")
        .arg(TARGET)
        .arg("--message-format=json-render-diagnostics")
        .args(cargo_args);
    config.set_environment(&mut cmd);
    // Target linker config also affects host build units on same-triple builds.
    // Explicit --target plus encoded rustflags keeps this linker target-only.
    cmd.env("CARGO_ENCODED_RUSTFLAGS", config.rustflags(&driver)?);
    cmd.env_remove("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS");
    println!(
        "Cargo command: {:?} {:?}",
        cmd.get_program(),
        cmd.get_args().collect::<Vec<_>>()
    );
    println!("Rust linker driver: {}", driver.display());
    println!(
        "Artifact selected for direct ELF verification: {}",
        artifact.display()
    );
    if dry_run {
        return Ok(0);
    }
    let (status, artifacts) = run_cargo(&mut cmd)?;
    if !status.success() {
        return Ok(status.code().unwrap_or(1));
    }
    let selected = canonical(artifact, "selected executable")?;
    if !artifacts
        .iter()
        .any(|path| fs::canonicalize(path).is_ok_and(|actual| actual == selected))
    {
        return Err(format!("{} was not reported as an executable by this Cargo invocation; select an executable it actually built or found fresh", artifact.display()));
    }
    let info = report::read_elf(artifact)?;
    let mut assessment = report::assess(&info, Some(config.ceiling));
    if info.interpreter.as_deref() != Some(LOADER) {
        assessment.errors.push(format!(
            "expected dynamically linked executable with interpreter {LOADER:?}; got {:?}",
            info.interpreter
        ));
    }
    if config.sysroot != Path::new("/") {
        for paths in info.rpath.iter().chain(&info.runpath) {
            for entry in paths.split(':') {
                if Path::new(entry).starts_with(&config.sysroot) {
                    assessment.errors.push(format!(
                        "build-time sysroot leaked into runtime library search path {entry:?}"
                    ));
                }
            }
        }
    }
    println!(
        "{}",
        report::render(artifact, &info, &assessment, Some(config.ceiling), false)
    );
    Ok(if assessment.errors.is_empty() { 0 } else { 2 })
}

fn run_cargo(cmd: &mut Command) -> Result<(ExitStatus, Vec<PathBuf>), String> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run Cargo: {e}"))?;
    let stdout = child.stdout.take().expect("piped Cargo stdout");
    let parsed = (|| {
        const LINE_LIMIT: u64 = 16 * 1024 * 1024;
        let mut reader = BufReader::new(stdout);
        let mut artifacts = Vec::new();
        loop {
            let mut line = Vec::new();
            let count = (&mut reader)
                .take(LINE_LIMIT + 1)
                .read_until(b'\n', &mut line)
                .map_err(|e| format!("cannot read Cargo messages: {e}"))?;
            if count == 0 {
                break;
            }
            if count as u64 > LINE_LIMIT {
                return Err("Cargo output line exceeds the 16 MiB limit".to_string());
            }
            let line =
                std::str::from_utf8(&line).map_err(|_| "Cargo output is not UTF-8".to_string())?;
            if let Some(path) = cargo_output::executable_from_message(line)? {
                if artifacts.len() >= 100_000 {
                    return Err("Cargo executable count exceeds the prototype limit".into());
                }
                artifacts.push(path);
            } else if !line.trim_start().starts_with('{') {
                print!("{line}");
            }
        }
        Ok(artifacts)
    })();
    match parsed {
        Ok(artifacts) => Ok((
            child
                .wait()
                .map_err(|e| format!("cannot wait for Cargo: {e}"))?,
            artifacts,
        )),
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(error)
        }
    }
}

fn prefixed_path(prefix: &str, path: &Path) -> OsString {
    let mut arg = OsString::from(prefix);
    arg.push(path.as_os_str());
    arg
}

fn tool_version(path: &Path) -> Result<String, String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| format!("cannot run {}: {e}", path.display()))?;
    if !output.status.success() {
        return Err(format!("{} --version failed", path.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .into())
}

fn remove_search_overrides(cmd: &mut Command) {
    for key in [
        "LIBRARY_PATH",
        "CPATH",
        "C_INCLUDE_PATH",
        "CPLUS_INCLUDE_PATH",
        "OBJC_INCLUDE_PATH",
        "GCC_EXEC_PREFIX",
        "COMPILER_PATH",
    ] {
        cmd.env_remove(key);
    }
}

pub fn run_linker(args: impl Iterator<Item = OsString>) -> Result<i32, String> {
    let required = |key: &str| {
        env::var_os(key).ok_or_else(|| {
            format!("{key} is missing; this helper is launched by cargo glibc build")
        })
    };
    let cc = required(CC_ENV)?;
    let root = required(SYSROOT_ENV)?;
    let gcc = required(GCC_ENV)?;
    let ld = required(LD_ENV)?;
    let mut cmd = Command::new(cc);
    cmd.args(args);
    // Fixed profile selections come last, after any arguments from rustc.
    cmd.arg(format!("--target={TARGET}"));
    cmd.arg(prefixed_path("--sysroot=", Path::new(&root)));
    cmd.arg(prefixed_path("--gcc-install-dir=", Path::new(&gcc)));
    cmd.arg("--rtlib=libgcc");
    cmd.arg(prefixed_path("-fuse-ld=", Path::new(&ld)));
    remove_search_overrides(&mut cmd);
    let status = cmd
        .status()
        .map_err(|e| format!("cannot launch Clang driver: {e}"))?;
    Ok(status.code().unwrap_or(1))
}

fn validate_x86_object(path: &Path) -> Result<(), String> {
    use std::io::Read;
    let mut bytes = [0u8; 20];
    fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| format!("cannot read object {}: {e}", path.display()))?;
    if &bytes[0..4] != b"\x7fELF"
        || bytes[4] != 2
        || bytes[5] != 1
        || u16::from_le_bytes([bytes[18], bytes[19]]) != 62
    {
        return Err(format!(
            "{} is not an x86-64 little-endian ELF object",
            path.display()
        ));
    }
    Ok(())
}

pub fn header_version(text: &str) -> Result<Version, String> {
    fn number(text: &str, name: &str) -> Result<u32, String> {
        for line in text.lines() {
            let Some(rest) = line.trim_start().strip_prefix('#') else {
                continue;
            };
            let mut words = rest.split_whitespace();
            if words.next() == Some("define") && words.next() == Some(name) {
                return words
                    .next()
                    .ok_or_else(|| format!("missing {name} value"))?
                    .parse()
                    .map_err(|_| format!("invalid {name} value"));
            }
        }
        Err(format!("features.h does not define {name}"))
    }
    Ok(Version {
        major: number(text, "__GLIBC__")?,
        minor: number(text, "__GLIBC_MINOR__")?,
        patch: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_glibc_headers_without_running_preprocessor() {
        assert_eq!(
            header_version("#define __GLIBC__ 2\n# define __GLIBC_MINOR__ 22\n").unwrap(),
            "2.22".parse().unwrap()
        );
        assert!(header_version("#define __GLIBC__ 2").is_err());
    }

    #[test]
    fn cargo_cannot_override_managed_target() {
        for value in [
            "--target",
            "--target=aarch64-unknown-linux-gnu",
            "--config=target.x.linker=other",
            "--message-format=json",
            "--",
            "+nightly",
        ] {
            assert!(validate_cargo_args(&[value.into()]).is_err());
        }
        assert!(validate_cargo_args(&[
            "--release".into(),
            "--target-dir".into(),
            "target with spaces".into()
        ])
        .is_ok());
    }

    #[test]
    fn path_with_spaces_is_one_driver_argument() {
        let cfg = Toolchain {
            sysroot: "/sdk root".into(),
            gcc_runtime: "/sdk root/gcc".into(),
            cc: "/usr/bin/clang".into(),
            ld: "/tools/ld.bfd".into(),
            ceiling: "2.22".parse().unwrap(),
        };
        assert!(cfg
            .driver_args()
            .contains(&OsString::from("--sysroot=/sdk root")));
        let flags = cfg.rustflags(Path::new("/tool path/glibc-linker")).unwrap();
        assert!(flags
            .to_str()
            .unwrap()
            .split('\u{1f}')
            .any(|s| s == "link-arg=--sysroot=/sdk root"));
    }
}
