use crate::{
    cargo_output,
    report::{self, ScanOptions},
    simulate::{self, Simulation},
    version::Version,
};
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
const SYSROOT_ENV: &str = "CARGO_LIBC_SYSROOT";
const GCC_ENV: &str = "CARGO_LIBC_GCC_RUNTIME";
const CC_ENV: &str = "CARGO_LIBC_CC";
const LD_ENV: &str = "CARGO_LIBC_LD";
const DESCRIPTOR_ENV: &str = "CARGO_LIBC_PROVIDER_DESCRIPTOR";

#[derive(Clone, Debug)]
pub struct Toolchain {
    pub provider: Option<crate::provider::Provider>,
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
    pub fn from_provider(
        path: &Path,
        cc: PathBuf,
        ld: PathBuf,
        ceiling: Version,
    ) -> Result<Self, String> {
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return Err("provider builds require x86-64 Linux".into());
        }

        let provider = crate::provider::Provider::load(path)?;
        provider.validate_selection(ceiling)?;
        provider.validate_files()?;
        let sysroot = canonical(&provider.sysroot, "provider sysroot")?;
        let gcc_runtime = canonical(&provider.gcc_startup, "provider GCC startup")?;
        let cc = canonical(&cc, "Clang driver")?;
        let ld = canonical(&ld, "linker")?;
        for path in [
            &sysroot,
            &gcc_runtime,
            &provider.stubs,
            &provider.compiler_runtime,
            &cc,
            &ld,
        ] {
            text_path(path)?;
        }
        Ok(Self {
            provider: Some(provider),
            sysroot,
            gcc_runtime,
            cc,
            ld,
            ceiling,
        })
    }
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
        ceiling.validate_glibc_target()?;
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
            provider: None,
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
        if let Some(provider) = &self.provider {
            provider.validate_files()?;
            if header_version != self.ceiling {
                return Err(format!(
                    "provider headers must match glibc {} exactly",
                    self.ceiling
                ));
            }
            for name in ["crt1.o", "crti.o", "crtn.o"] {
                let path = inside_file(&self.sysroot.join("usr/lib64").join(name), &self.sysroot)?;
                validate_x86_object(&path)?;
                files.push((name.into(), path));
            }
            for name in ["crtbegin.o", "crtend.o"] {
                let path = inside_file(&self.gcc_runtime.join(name), &self.gcc_runtime)?;
                validate_x86_object(&path)?;
                files.push((name.into(), path));
            }
            for name in crate::provider::SONAMES {
                let path = provider.stubs.join(name);
                validate_x86_object(&path)?;
                files.push((name.into(), path));
            }
            return Ok(Diagnostics {
                header_version,
                compiler_version,
                linker_version,
                files,
            });
        }
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
        Ok(self.rustflag_list(driver)?.join("\u{1f}").into())
    }

    pub fn rustflag_list(&self, driver: &Path) -> Result<Vec<String>, String> {
        // These input paths deliberately also appear in rustc flags: Cargo then sees
        // profile-path changes in its fingerprints rather than only hidden env vars.
        let mut flags = vec![
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
            format!("cargo_libc_profile=\"{}\"", self.profile_fingerprint()?),
        ];
        if self.provider.is_some() {
            flags.extend(["-C".into(), "relocation-model=static".into()]);
        }
        Ok(flags)
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
        if let Some(provider) = &self.provider {
            for byte in provider.descriptor.bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        Ok(format!("{hash:016x}"))
    }

    /// The selection the helpers read (libc-linker, libc-cc).
    pub fn variables(&self) -> Vec<(&'static str, OsString)> {
        let mut variables = vec![
            (SYSROOT_ENV, self.sysroot.clone().into_os_string()),
            (GCC_ENV, self.gcc_runtime.clone().into_os_string()),
            (CC_ENV, self.cc.clone().into_os_string()),
            (LD_ENV, self.ld.clone().into_os_string()),
        ];
        if let Some(provider) = &self.provider {
            variables.push((DESCRIPTOR_ENV, provider.descriptor.clone().into()));
        }
        variables
    }

    pub fn set_environment(&self, cmd: &mut Command) {
        cmd.env_remove(DESCRIPTOR_ENV);
        cmd.envs(self.variables());
    }
}

/// Cargo subcommands whose own `--` passes the rest to the compiler or linter.
fn has_tool_arguments(subcommand: &str) -> bool {
    matches!(subcommand, "clippy" | "rustc")
}

pub fn validate_cargo_args(subcommand: &str, args: &[OsString]) -> Result<(), String> {
    for (i, arg) in args.iter().enumerate() {
        let text = arg.to_string_lossy();
        if text == "--" && has_tool_arguments(subcommand) {
            return validate_tool_args(&args[i + 1..]);
        }
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

/// Arguments after `cargo clippy --` or `cargo rustc --` reach rustc directly:
/// they must not replace the target, linker or relocation model.
fn validate_tool_args(args: &[OsString]) -> Result<(), String> {
    for arg in args {
        let text = arg.to_string_lossy();
        if text.starts_with("--target")
            || text.contains("linker")
            || text.contains("relocation-model")
            || text.contains("link-arg")
        {
            return Err(format!(
                "compiler argument {text:?} conflicts with the managed target configuration"
            ));
        }
    }
    Ok(())
}

pub fn sibling(name: &str) -> Result<PathBuf, String> {
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    let path = executable
        .parent()
        .ok_or_else(|| format!("cannot locate sibling helper {name}"))?
        .join(name);
    if !path.is_file() {
        return Err(format!(
            "missing Rust helper {}; install/build all cargo-libc binaries",
            path.display()
        ));
    }
    Ok(path)
}

/// A TOML basic string, for Cargo's `--config KEY=VALUE`.
fn toml_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Probes the inputs and prepares `cargo SUBCOMMAND --target ...` with the
/// managed target flags; returns the command and the linker driver path.
fn cargo_command(
    config: &Toolchain,
    subcommand: &str,
    cargo_args: &[OsString],
) -> Result<(Command, PathBuf), String> {
    validate_cargo_args(subcommand, cargo_args)?;
    // `test` compiles doctests and `doc` documents with rustdoc, which needs the
    // same flags (the cfg, and the linker for doctests).
    let doctests = matches!(subcommand, "test" | "doc");
    let mut managed = vec!["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"];
    if doctests {
        managed.extend(["RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS"]);
    }
    for key in managed {
        if env::var_os(key).is_some_and(|value| !value.is_empty()) {
            return Err(format!("{key} is already set; use a shell without that override so the target flags are explicit"));
        }
    }
    let diagnostic = config.diagnose()?;
    config.print_diagnostics(&diagnostic);
    let driver = sibling("libc-linker")?;
    let (cc, cxx) = crate::cc::wrappers()?;
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg(subcommand).arg("--target").arg(TARGET);
    config.set_environment(&mut cmd);
    // C dependencies (the `cc` crate, CMake through it) compile for the target
    // with libc-cc; build scripts and proc macros keep the host compiler.
    crate::cc::set_environment(&mut cmd, &cc, &cxx)?;
    // Target linker config also affects host build units on same-triple builds.
    // Explicit --target plus encoded rustflags keeps this linker target-only.
    let flags = config.rustflags(&driver)?;
    println!(
        "Target C compiler (cc crate, CMake): {} (C only; host units keep the host compiler)",
        cc.display()
    );
    if doctests {
        cmd.env("CARGO_ENCODED_RUSTDOCFLAGS", &flags);
    }
    cmd.env("CARGO_ENCODED_RUSTFLAGS", flags);
    cmd.env_remove("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS");
    Ok((cmd, driver))
}

fn print_command(cmd: &Command) {
    println!(
        "Cargo command: {:?} {:?}",
        cmd.get_program(),
        cmd.get_args().collect::<Vec<_>>()
    );
}

pub fn build(
    config: &Toolchain,
    cargo_args: &[OsString],
    artifact: &Path,
    scan: Option<&ScanOptions>,
    dry_run: bool,
) -> Result<i32, String> {
    let (mut cmd, driver) = cargo_command(config, "build", cargo_args)?;
    cmd.arg("--message-format=json-render-diagnostics")
        .args(cargo_args);
    print_command(&cmd);
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
    let passed = verify_executable(config, artifact, scan)?;
    Ok(if passed { 0 } else { 2 })
}

/// The direct ELF check (and the tier 1 scan when requested) of one executable
/// from this build; prints the report and returns whether it passed.
fn verify_executable(
    config: &Toolchain,
    artifact: &Path,
    scan: Option<&ScanOptions>,
) -> Result<bool, String> {
    let bytes = report::read_bytes(artifact)?;
    let info = crate::elf::inspect(&bytes)?;
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
    // A preset already set the ceiling; otherwise the scan uses --glibc.
    let scan = scan
        .map(|options| {
            let ceiling = options.distro.is_none().then_some(config.ceiling);
            report::scan(&bytes, ceiling, options).map(|(_, scan)| scan)
        })
        .transpose()?;
    println!(
        "{}",
        report::render(
            artifact,
            &info,
            &assessment,
            Some(config.ceiling),
            scan.as_ref(),
            false
        )
    );
    Ok(assessment.errors.is_empty() && scan.as_ref().is_none_or(report::Scan::passed))
}

/// `cargo libc check|clippy|doc|rustc`: Cargo with the same managed target
/// flags as `build`. Cargo's diagnostics and exit status pass through
/// unchanged; there is no artifact to verify (`rustc` may link one: inspect it).
pub fn cargo_passthrough(
    config: &Toolchain,
    subcommand: &str,
    cargo_args: &[OsString],
    dry_run: bool,
) -> Result<i32, String> {
    let (mut cmd, driver) = cargo_command(config, subcommand, cargo_args)?;
    cmd.args(cargo_args);
    print_command(&cmd);
    let links = subcommand == "rustc";
    println!(
        "Rust linker driver: {}{}",
        driver.display(),
        if links {
            ""
        } else {
            " (configured; this command does not link)"
        }
    );
    if dry_run {
        return Ok(0);
    }
    let status = cmd.status().map_err(|e| format!("cannot run Cargo: {e}"))?;
    if status.success() {
        if subcommand == "check" {
            println!(
                "Type check passed for glibc {} target flags. Nothing was linked: build and inspect the executable to check its requirements.",
                config.ceiling
            );
        } else if links {
            println!(
                "Built with glibc {} target flags. The output was not checked: run cargo libc inspect on it.",
                config.ceiling
            );
        }
    }
    Ok(status.code().unwrap_or(1))
}

/// `cargo libc install`: `cargo install` with the managed target flags, then
/// the same direct ELF check (and scan) as `build` for every executable built.
/// The check runs after Cargo installed them; a failure exits with 2.
pub fn install(
    config: &Toolchain,
    cargo_args: &[OsString],
    scan: Option<&ScanOptions>,
    dry_run: bool,
) -> Result<i32, String> {
    let (mut cmd, driver) = cargo_command(config, "install", cargo_args)?;
    cmd.arg("--message-format=json-render-diagnostics");
    // cargo install deletes its temporary target directory; keep the
    // executables until they are checked.
    let explicit = env::var_os("CARGO_TARGET_DIR").is_some_and(|v| !v.is_empty())
        || cargo_args.iter().any(|arg| {
            let text = arg.to_string_lossy();
            text == "--target-dir" || text.starts_with("--target-dir=")
        });
    let temporary = (!explicit).then(|| {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        env::temp_dir().join(format!("cargo-libc-install-{}-{nanos}", std::process::id()))
    });
    if let Some(dir) = &temporary {
        cmd.arg("--target-dir").arg(dir);
    }
    cmd.args(cargo_args);
    print_command(&cmd);
    println!("Rust linker driver: {}", driver.display());
    println!(
        "Every executable this install builds is checked against glibc {}.",
        config.ceiling
    );
    if dry_run {
        return Ok(0);
    }
    let result = (|| {
        let (status, artifacts) = run_cargo(&mut cmd)?;
        if !status.success() {
            return Ok(status.code().unwrap_or(1));
        }
        if artifacts.is_empty() {
            println!("Cargo built no executable (already installed?); nothing was checked.");
            return Ok(0);
        }
        let mut passed = true;
        for artifact in &artifacts {
            passed &= verify_executable(config, artifact, scan)?;
        }
        if !passed {
            println!("An installed executable failed the check above; remove it or rebuild for a newer target.");
        }
        Ok(if passed { 0 } else { 2 })
    })();
    if let Some(dir) = &temporary {
        let _ = fs::remove_dir_all(dir);
    }
    result
}

/// Output format of `cargo libc env`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvFormat {
    /// POSIX shell `export` lines.
    Shell,
    /// Cargo configuration (TOML) for `--config FILE` or `.cargo/config.toml`.
    Cargo,
}

fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// `cargo libc env`: the configuration `build`/`test` apply, for plain Cargo
/// with `--target x86_64-unknown-linux-gnu`.
pub fn plain_cargo(
    config: &Toolchain,
    simulation: &Simulation,
    format: EnvFormat,
) -> Result<String, String> {
    simulation.plan()?;
    config.diagnose()?;
    let driver = sibling("libc-linker")?;
    let runner = sibling("libc-runner")?;
    let (cc, cxx) = crate::cc::wrappers()?;
    let flags = config.rustflag_list(&driver)?;
    let runner = text_path(&runner)?.to_string();
    let mut variables: Vec<(String, String)> = Vec::new();
    let utf8 = |key: &str, value: OsString| {
        value
            .into_string()
            .map_err(|_| format!("{key} must be UTF-8 for plain Cargo configuration"))
    };
    for (key, value) in config.variables() {
        variables.push((key.to_string(), utf8(key, value)?));
    }
    for (key, value) in crate::cc::variables(&cc, &cxx, format == EnvFormat::Shell)? {
        let value = utf8(&key, value)?;
        variables.push((key, value));
    }
    for (key, value) in simulation.variables(config.ceiling) {
        if let Some(value) = value {
            variables.push((key.to_string(), value));
        }
    }
    let triple_key = TARGET.to_uppercase().replace('-', "_");
    let mut out = format!(
        "# cargo-libc {}: glibc {} for {TARGET}, generated for this machine.\n\
         # Use with: cargo build --target {TARGET} (always pass --target, so build\n\
         # scripts and proc macros stay native). RUSTFLAGS or CARGO_ENCODED_RUSTFLAGS in\n\
         # the environment replace these flags. Nothing checks the output: run\n\
         # cargo libc inspect FILE --max-glibc {}. Regenerate after updating\n\
         # cargo-libc or the provider.\n",
        env!("CARGO_PKG_VERSION"),
        config.ceiling,
        config.ceiling
    );
    match format {
        EnvFormat::Shell => {
            if let Some(flag) = flags
                .iter()
                .chain([&runner])
                .find(|f| f.chars().any(char::is_whitespace))
            {
                return Err(format!("{flag:?} contains whitespace, which Cargo's flag variables cannot hold; use --format cargo"));
            }
            let joined = flags.join(" ");
            out.push_str(&format!(
                "export CARGO_TARGET_{triple_key}_RUSTFLAGS={}\n",
                shell_quote(&joined)
            ));
            out.push_str(&format!("export RUSTDOCFLAGS={}\n", shell_quote(&joined)));
            out.push_str(&format!(
                "export CARGO_TARGET_{triple_key}_RUNNER={}\n",
                shell_quote(&runner)
            ));
            for (key, value) in variables {
                out.push_str(&format!("export {key}={}\n", shell_quote(&value)));
            }
        }
        EnvFormat::Cargo => {
            let list = |items: &[String]| {
                items
                    .iter()
                    .map(|f| toml_string(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            out.push_str(&format!(
                "\n[target.{TARGET}]\nrustflags = [{}]\nrunner = [{}]\n\n[build]\nrustdocflags = [{}]\n\n[env]\n",
                list(&flags),
                toml_string(&runner),
                list(&flags)
            ));
            for (key, value) in variables {
                out.push_str(&format!("{key} = {}\n", toml_string(&value)));
            }
        }
    }
    Ok(out)
}

/// `cargo libc test` / `cargo libc run`: Cargo builds with the managed target
/// flags and runs every target binary through `libc-runner` (tier 2).
pub fn test_or_run(
    config: &Toolchain,
    subcommand: &str,
    cargo_args: &[OsString],
    simulation: &Simulation,
    dry_run: bool,
) -> Result<i32, String> {
    let (plan, _) = simulation.plan()?;
    let (mut cmd, driver) = cargo_command(config, subcommand, cargo_args)?;
    let runner = sibling("libc-runner")?;
    // An array keeps a runner path with spaces as one program name.
    cmd.arg("--config").arg(format!(
        "target.{TARGET}.runner=[{}]",
        toml_string(text_path(&runner)?)
    ));
    cmd.args(cargo_args);
    simulation.set_environment(&mut cmd, config.ceiling);
    print_command(&cmd);
    println!("Rust linker driver: {}", driver.display());
    println!("Target runner: {}", runner.display());
    println!("Kernel simulation: {}", simulate::describe(&plan));
    println!(
        "Each binary is checked against glibc {} before it runs.",
        config.ceiling
    );
    if dry_run {
        return Ok(0);
    }
    let status = cmd.status().map_err(|e| format!("cannot run Cargo: {e}"))?;
    Ok(status.code().unwrap_or(1))
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

/// The selection `cargo libc` hands its helpers through the environment.
struct Selection {
    cc: OsString,
    sysroot: OsString,
    gcc: OsString,
    ld: OsString,
    provider: Option<crate::provider::Provider>,
}

impl Selection {
    fn from_environment(helper: &str) -> Result<Self, String> {
        let required = |key: &str| {
            env::var_os(key).ok_or_else(|| {
                format!("{key} is missing; {helper} is launched by cargo libc build")
            })
        };
        let provider = env::var_os("CARGO_LIBC_PROVIDER_DESCRIPTOR")
            .map(|descriptor| {
                let provider = crate::provider::Provider::parse(
                    descriptor
                        .to_str()
                        .ok_or("provider descriptor must be UTF-8")?,
                )?;
                provider.validate_files()?;
                Ok::<_, String>(provider)
            })
            .transpose()?;
        Ok(Self {
            cc: required(CC_ENV)?,
            sysroot: required(SYSROOT_ENV)?,
            gcc: required(GCC_ENV)?,
            ld: required(LD_ENV)?,
            provider,
        })
    }

    /// Fixed profile selections; they come last, after the caller's arguments.
    fn finish(&self, cmd: &mut Command, link: bool) {
        cmd.arg(format!("--target={TARGET}"));
        cmd.arg(prefixed_path("--sysroot=", Path::new(&self.sysroot)));
        cmd.arg(prefixed_path("--gcc-install-dir=", Path::new(&self.gcc)));
        if link {
            if self.provider.is_none() {
                cmd.arg("--rtlib=libgcc");
            }
            cmd.arg(prefixed_path("-fuse-ld=", Path::new(&self.ld)));
        }
        remove_search_overrides(cmd);
    }
}

/// A provider link: only the provider's startup objects, stubs and archives,
/// around `args`. `shared` links a shared library instead of an executable.
fn provider_link(
    cmd: &mut Command,
    provider: &crate::provider::Provider,
    args: impl IntoIterator<Item = OsString>,
    default_libraries: bool,
    shared: bool,
) {
    let (begin, end) = if shared {
        ("crtbeginS.o", "crtendS.o")
    } else {
        ("crtbegin.o", "crtend.o")
    };
    cmd.arg("-nostdlib");
    if !shared {
        cmd.arg("-no-pie");
    }
    if let Some(path) = env::var_os("CARGO_LIBC_PROVIDER_LINK_MAP") {
        cmd.arg(prefixed_path("-Wl,-Map=", Path::new(&path)));
    }
    cmd.arg(prefixed_path("-L", &provider.stubs));
    cmd.arg(prefixed_path("-L", &provider.compiler_runtime));
    if !shared {
        cmd.arg(provider.sysroot.join("usr/lib64/crt1.o"));
    }
    cmd.arg(provider.sysroot.join("usr/lib64/crti.o"));
    cmd.arg(provider.gcc_startup.join(begin));
    cmd.args(args);
    // GCC's default libraries for a dynamic link; rustc names its own.
    let as_needed_gcc_s = ["-Wl,--push-state,--as-needed", "-lgcc_s", "-Wl,--pop-state"];
    if default_libraries {
        cmd.arg("-lgcc").args(as_needed_gcc_s).arg("-lc");
    }
    for archive in provider.nonshared_archives() {
        cmd.arg(provider.sysroot.join(archive));
    }
    if default_libraries {
        cmd.arg("-lgcc").args(as_needed_gcc_s);
    }
    cmd.arg(provider.gcc_startup.join(end));
    cmd.arg(provider.sysroot.join("usr/lib64/crtn.o"));
    if !shared {
        cmd.arg(format!("-Wl,--dynamic-linker={LOADER}"));
    }
}

pub fn run_linker(args: impl Iterator<Item = OsString>) -> Result<i32, String> {
    let selection = Selection::from_environment("this helper")?;
    let mut cmd = Command::new(&selection.cc);
    match &selection.provider {
        Some(provider) => provider_link(&mut cmd, provider, args, false, false),
        None => {
            cmd.args(args);
        }
    }
    selection.finish(&mut cmd, true);
    let status = cmd
        .status()
        .map_err(|e| format!("cannot launch Clang driver: {e}"))?;
    Ok(status.code().unwrap_or(1))
}

/// The target C compiler command for `libc-cc`: Clang with the selected
/// sysroot or provider. Links get the provider's startup objects and stubs.
pub fn target_compiler(action: crate::cc::Action, args: Vec<OsString>) -> Result<Command, String> {
    use crate::cc::Action;
    let selection = Selection::from_environment("this compiler wrapper")?;
    let mut cmd = Command::new(&selection.cc);
    let link = matches!(action, Action::Link { .. } | Action::Relocatable);
    match (&selection.provider, action) {
        (Some(provider), Action::Link { shared }) => {
            provider_link(&mut cmd, provider, args, true, shared)
        }
        _ => {
            cmd.args(args);
        }
    }
    selection.finish(&mut cmd, link);
    Ok(cmd)
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
            assert!(validate_cargo_args("build", &[value.into()]).is_err());
        }
        assert!(validate_cargo_args(
            "build",
            &[
                "--release".into(),
                "--target-dir".into(),
                "target with spaces".into()
            ]
        )
        .is_ok());
    }

    #[test]
    fn runner_path_is_one_toml_string() {
        assert_eq!(
            toml_string("/tool path/libc-runner"),
            "\"/tool path/libc-runner\""
        );
        assert_eq!(toml_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000A\"");
    }

    #[test]
    fn path_with_spaces_is_one_driver_argument() {
        let cfg = Toolchain {
            provider: None,
            sysroot: "/sdk root".into(),
            gcc_runtime: "/sdk root/gcc".into(),
            cc: "/usr/bin/clang".into(),
            ld: "/tools/ld.bfd".into(),
            ceiling: "2.22".parse().unwrap(),
        };
        assert!(cfg
            .driver_args()
            .contains(&OsString::from("--sysroot=/sdk root")));
        let flags = cfg.rustflags(Path::new("/tool path/libc-linker")).unwrap();
        assert!(flags
            .to_str()
            .unwrap()
            .split('\u{1f}')
            .any(|s| s == "link-arg=--sysroot=/sdk root"));
    }

    #[test]
    fn shell_values_are_single_quoted() {
        assert_eq!(shell_quote("/a b/c"), "'/a b/c'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }
}
