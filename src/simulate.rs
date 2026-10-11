//! Tier 2: `cargo libc test` and `cargo libc run` execute each target binary
//! through the `libc-runner` helper, which Cargo calls as its target runner.
//!
//! The helper first applies the direct glibc check to the binary (a newer
//! host glibc would otherwise hide a too-new requirement under the local
//! runner), then runs it under the kernel simulation from `kernel-abi-tools`:
//! the local seccomp filter (host userland, no root) or a container with the
//! target's userland. The selection travels from `cargo libc` to the helper
//! in `CARGO_LIBC_RUN_*` environment variables.
//!
//! Without a runner, and with the local runner, the program uses the host's
//! glibc. The helper then also compares the program's GLIBC version
//! requirements with what the host's libraries define (the same test the
//! loader makes) and refuses to start a program that cannot load there.

use crate::{elf::ElfInfo, report, version::Version};
use kernel_abi_tools::{
    container::{self, Engine},
    local, table_floor, KernelVersion, Runner, Target,
};
use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const GLIBC_ENV: &str = "CARGO_LIBC_RUN_GLIBC";
const DISTRO_ENV: &str = "CARGO_LIBC_RUN_DISTRO";
const KERNEL_ENV: &str = "CARGO_LIBC_RUN_KERNEL";
const RUNNER_ENV: &str = "CARGO_LIBC_RUN_RUNNER";
const IMAGE_ENV: &str = "CARGO_LIBC_RUN_IMAGE";

/// What `test`/`run` simulate. `distro` excludes `kernel`; `image` needs a
/// kernel floor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Simulation {
    pub distro: Option<String>,
    pub kernel: Option<String>,
    pub runner: Option<Runner>,
    pub image: Option<String>,
}

/// How one binary will run.
#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    /// No kernel floor: run directly on the host (the glibc check still applies).
    Direct,
    Local(String),
    Container(String, String),
}

impl Simulation {
    /// The kernel target (with a preset's backports), if a floor was given.
    pub fn target(&self) -> Result<Option<Target>, String> {
        match (&self.distro, &self.kernel) {
            (Some(_), Some(_)) => {
                Err("--distro sets the kernel floor; do not combine it with --kernel".into())
            }
            (Some(id), None) => {
                let d =
                    kernel_abi_tools::distro(id).ok_or_else(|| format!("unknown distro {id:?}"))?;
                Ok(Some(Target::distro(&d)))
            }
            (None, Some(k)) => Ok(Some(Target::kernel(KernelVersion::parse(k)?))),
            (None, None) => Ok(None),
        }
    }

    /// Applies `kernel_abi_tools::Runner::choose`: a requested runner wins; a
    /// known image (`--image`, or the preset's) means the container, otherwise
    /// the local filter. A preset's image is only a default, so `--runner
    /// local` ignores it.
    pub fn plan(&self) -> Result<(Plan, Option<Target>), String> {
        let target = self.target()?;
        let Some(t) = &target else {
            if self.runner.is_some() || self.image.is_some() {
                return Err(
                    "--runner and --image need a kernel floor (--distro or --kernel)".into(),
                );
            }
            return Ok((Plan::Direct, None));
        };
        let preset = self
            .distro
            .as_deref()
            .and_then(kernel_abi_tools::distro)
            .map(|d| d.image.to_string());
        let image = match self.runner {
            Some(Runner::Local) => self.image.clone(),
            _ => self.image.clone().or(preset),
        };
        let plan = match Runner::choose(self.runner, image.as_deref())? {
            Runner::Local => Plan::Local(t.label.clone()),
            Runner::Container => Plan::Container(t.label.clone(), image.unwrap_or_default()),
        };
        Ok((plan, target))
    }

    /// Passes the selection to the helper.
    pub fn set_environment(&self, cmd: &mut Command, ceiling: Version) {
        cmd.env(GLIBC_ENV, ceiling.to_string());
        let runner = self.runner.map(|r| match r {
            Runner::Local => "local",
            Runner::Container => "container",
        });
        for (key, value) in [
            (DISTRO_ENV, self.distro.as_deref()),
            (KERNEL_ENV, self.kernel.as_deref()),
            (RUNNER_ENV, runner),
            (IMAGE_ENV, self.image.as_deref()),
        ] {
            match value {
                Some(v) => cmd.env(key, v),
                None => cmd.env_remove(key),
            };
        }
    }

    fn from_environment() -> Result<(Self, Version), String> {
        let text = |key: &str| -> Result<Option<String>, String> {
            match env::var_os(key) {
                None => Ok(None),
                Some(v) if v.is_empty() => Ok(None),
                Some(v) => v
                    .into_string()
                    .map(Some)
                    .map_err(|_| format!("{key} must be UTF-8")),
            }
        };
        let ceiling = text(GLIBC_ENV)?
            .ok_or_else(|| {
                format!("{GLIBC_ENV} is missing; this helper is launched by cargo libc test/run")
            })?
            .parse::<Version>()?
            .validate_glibc_target()?;
        let sim = Simulation {
            distro: text(DISTRO_ENV)?,
            kernel: text(KERNEL_ENV)?,
            runner: text(RUNNER_ENV)?.map(|r| Runner::parse(&r)).transpose()?,
            image: text(IMAGE_ENV)?,
        };
        Ok((sim, ceiling))
    }
}

/// Describes the plan for `cargo libc test/run` output.
pub fn describe(plan: &Plan) -> String {
    match plan {
        Plan::Direct => "no kernel simulation (host kernel); glibc check only".into(),
        Plan::Local(label) => {
            format!("{label} with the local seccomp filter; userland (libc) is the host's")
        }
        Plan::Container(label, image) => format!("{label} in container image {image}"),
    }
}

/// Whether the host's glibc can load a program that runs on the host userland.
#[derive(Debug, PartialEq, Eq)]
pub enum HostGlibc {
    Satisfied,
    /// `library GLIBC_x.y` requirements the host's libraries do not define.
    Missing(Vec<String>),
    /// The host libraries could not be read; the run proceeds with a warning.
    Unknown(String),
}

/// Compare every non-weak GLIBC_* requirement with the versions `defined`
/// reports for that library (the loader rejects any missing one).
pub fn host_glibc(
    info: &ElfInfo,
    mut defined: impl FnMut(&str) -> Result<Vec<String>, String>,
) -> HostGlibc {
    let mut required: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for need in info
        .versions
        .iter()
        .filter(|n| !n.weak && n.name.starts_with("GLIBC_"))
    {
        required.entry(&need.library).or_default().push(&need.name);
    }
    let mut missing = Vec::new();
    for (library, names) in required {
        let available = match defined(library) {
            Ok(available) => available,
            Err(e) => return HostGlibc::Unknown(e),
        };
        for name in names {
            if !available.iter().any(|a| a == name) {
                missing.push(format!("{library} {name}"));
            }
        }
    }
    if missing.is_empty() {
        HostGlibc::Satisfied
    } else {
        HostGlibc::Missing(missing)
    }
}

/// The directory of the libc.so.6 this helper loaded: the host's glibc.
fn host_libc_dir() -> Result<PathBuf, String> {
    let maps = fs::read_to_string("/proc/self/maps")
        .map_err(|e| format!("cannot read /proc/self/maps: {e}"))?;
    maps.lines()
        .filter_map(|line| line.split_whitespace().nth(5))
        .map(Path::new)
        .find(|path| path.file_name().is_some_and(|name| name == "libc.so.6"))
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "this helper did not load a glibc libc.so.6".into())
}

/// The directories the loader searches before the system glibc: DT_RPATH
/// (only without DT_RUNPATH), LD_LIBRARY_PATH, then DT_RUNPATH, with $ORIGIN
/// expanded. Other tokens ($LIB, $PLATFORM) make the result uncertain.
fn search_dirs(program: &Path, info: &ElfInfo) -> Result<Vec<PathBuf>, String> {
    let origin = program
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));
    let expand = |list: &str| -> Result<Vec<PathBuf>, String> {
        list.split(':')
            .map(|entry| {
                let entry = entry
                    .replace("${ORIGIN}", &origin.to_string_lossy())
                    .replace("$ORIGIN", &origin.to_string_lossy());
                if entry.contains('$') {
                    return Err(format!("cannot expand library search path entry {entry:?}"));
                }
                Ok(PathBuf::from(if entry.is_empty() { "." } else { &entry }))
            })
            .collect()
    };
    let mut dirs = Vec::new();
    if info.runpath.is_empty() {
        for list in &info.rpath {
            dirs.extend(expand(list)?);
        }
    }
    if let Some(list) = env::var_os("LD_LIBRARY_PATH").filter(|v| !v.is_empty()) {
        dirs.extend(expand(&list.to_string_lossy())?);
    }
    for list in &info.runpath {
        dirs.extend(expand(list)?);
    }
    Ok(dirs)
}

/// Version definitions of the `library` file the loader would use: the first
/// one in the search directories, otherwise the one beside the host's libc.so.6.
fn host_definitions(dirs: &[PathBuf], host: &Path, library: &str) -> Result<Vec<String>, String> {
    if library.contains('/') {
        return Err(format!("unexpected library name {library:?}"));
    }
    let path = dirs
        .iter()
        .map(|dir| dir.join(library))
        .find(|path| path.is_file())
        .unwrap_or_else(|| host.join(library));
    let bytes = fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(crate::elf::inspect(&bytes)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .definitions)
}

/// The highest GLIBC version among `library GLIBC_x.y` entries.
fn highest(entries: &[String]) -> Option<Version> {
    entries
        .iter()
        .filter_map(|e| e.rsplit_once(" GLIBC_")?.1.parse().ok())
        .max()
}

/// Applies the host check for runs on the host userland: refuses (exit code
/// 2) when the libraries the loader would use cannot satisfy the program, and
/// warns and runs when it cannot tell.
fn check_host(program: &str, info: &ElfInfo) -> Option<i32> {
    let located =
        host_libc_dir().and_then(|host| Ok((search_dirs(Path::new(program), info)?, host)));
    let result = match &located {
        Ok((dirs, host)) => host_glibc(info, |library| host_definitions(dirs, host, library)),
        Err(e) => HostGlibc::Unknown(e.clone()),
    };
    match result {
        HostGlibc::Satisfied => None,
        HostGlibc::Unknown(e) => {
            eprintln!("libc-runner: warning: host glibc not checked ({e}); running {program}");
            None
        }
        HostGlibc::Missing(missing) => {
            let need = highest(&missing).map_or_else(String::new, |v| format!(" {v} or newer"));
            eprintln!(
                "libc-runner: not running {program}: the host's libraries lack {}, so the \
                 loader would refuse it. Run it in a container with glibc{need} (--runner \
                 container --image IMAGE, or --distro), or build it with an older provider.",
                missing.join(", ")
            );
            Some(2)
        }
    }
}

/// The `libc-runner` entry point: `libc-runner PROGRAM [ARGS...]`.
pub fn run_program(mut args: impl Iterator<Item = OsString>) -> Result<i32, String> {
    let program = args
        .next()
        .ok_or("usage: libc-runner PROGRAM [ARGS...] (Cargo's target runner)")?;
    let program = program
        .into_string()
        .map_err(|_| "program path must be UTF-8".to_string())?;
    let args = args
        .map(|a| {
            a.into_string()
                .map_err(|_| "arguments must be UTF-8".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (sim, ceiling) = Simulation::from_environment()?;

    let info = report::read_elf(Path::new(&program))?;
    let assessment = report::assess(&info, Some(ceiling));
    if !assessment.errors.is_empty() {
        for error in &assessment.errors {
            eprintln!("libc-runner: {program}: {error}");
        }
        eprintln!("libc-runner: not running {program}: it needs a newer glibc than {ceiling}");
        return Ok(2);
    }

    let (plan, target) = sim.plan()?;
    // The container supplies its own glibc; the other plans use the host's.
    if !matches!(plan, Plan::Container(..)) {
        if let Some(code) = check_host(&program, &info) {
            return Ok(code);
        }
    }
    if let Some(t) = &target {
        if t.kernel < table_floor() {
            eprintln!(
                "libc-runner: warning: kernel data starts at {}; {} is treated as {}",
                table_floor(),
                t.kernel,
                table_floor()
            );
        }
    }
    match (&plan, &target) {
        (Plan::Local(_), Some(t)) => local::run(t, &program, &args),
        (Plan::Container(_, image), Some(t)) => {
            container::run(&Engine::from_env(), t, image, &program, &args)
        }
        _ => {
            let status = Command::new(&program)
                .args(&args)
                .status()
                .map_err(|e| format!("cannot start {program}: {e}"))?;
            Ok(status.code().unwrap_or(128))
        }
    }
}

/// The OCI runtime named in the engine options (`--runtime X` or
/// `--runtime=X`), if any.
pub fn runtime_from_engine_args(args: &[String]) -> Option<String> {
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--runtime" {
            return it.next().cloned();
        }
        if let Some(value) = arg.strip_prefix("--runtime=") {
            return Some(value.to_string());
        }
    }
    None
}

fn first_line(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{program} {} failed", args.join(" ")));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string())
}

/// The local runner: install the filter for `target` in a child and run
/// `/bin/sh -c 'exit 0'` under it.
fn check_local(target: &Target) -> Result<String, String> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err("needs an x86_64 Linux host".into());
    }
    match local::run(target, "/bin/sh", &["-c".into(), "exit 0".into()])? {
        0 => Ok(format!(
            "seccomp filter for {} installed without root; /bin/sh ran under it",
            target.label
        )),
        code => Err(format!("/bin/sh exited with {code} under the filter")),
    }
}

const UNTESTED: &str =
    "engine found; no container started (select an image with --image or --distro)";

/// The container runner: engine and runtime versions, then `true` in the
/// image under the profile for `target`.
fn check_container(target: &Target, image: Option<&str>) -> Result<String, String> {
    let engine = Engine::from_env();
    let version = first_line(&engine.program, &["--version"])?;
    let runtime = match runtime_from_engine_args(&engine.engine_args) {
        Some(r) => format!("runtime {r} ({})", first_line(&r, &["--version"])?),
        None => "the engine's default runtime".into(),
    };
    println!("  container: {version}; {runtime}");
    let Some(image) = image else {
        return Ok(UNTESTED.into());
    };
    println!(
        "  container: starting `true` in {image} under the {} profile (may pull the image)",
        target.label
    );
    match container::run(&engine, target, image, "true", &[])? {
        0 => Ok(format!(
            "`true` ran in {image} under the {} profile",
            target.label
        )),
        code => Err(format!(
            "`true` exited with {code} in {image}. If the runtime failed to start, its \
             libseccomp may not know every syscall in the profile, or the runtime itself \
             (e.g. runc's statx) is caught by the older kernel's profile; crun 1.28 works: \
             KERNEL_ABI_ENGINE_ARGS=\"--runtime /path/to/crun\""
        )),
    }
}

/// `cargo libc doctor` runner checks: reports both runners and returns whether
/// the one `test`/`run` would choose is usable.
pub fn doctor(simulation: &Simulation) -> Result<bool, String> {
    let (plan, target) = simulation.plan()?;
    println!("Kernel simulation for test/run: {}", describe(&plan));
    let Some(target) = target else {
        println!("Runner checks skipped: no kernel floor (--distro or --kernel).");
        return Ok(true);
    };
    let image = match &plan {
        Plan::Container(_, image) => Some(image.clone()),
        _ => simulation.image.clone().or_else(|| {
            simulation
                .distro
                .as_deref()
                .and_then(kernel_abi_tools::distro)
                .map(|d| d.image.to_string())
        }),
    };
    let report = |name: &str, result: &Result<String, String>| match result {
        Ok(text) if text == UNTESTED => println!("  {name}: not tested: {text}"),
        Ok(text) => println!("  {name}: ok: {text}"),
        Err(text) => println!("  {name}: unavailable: {text}"),
    };
    println!("Runner checks:");
    let local = check_local(&target);
    report("local", &local);
    let container = check_container(&target, image.as_deref());
    report("container", &container);
    let selected = match plan {
        Plan::Local(_) => local.is_ok(),
        Plan::Container(..) => container.is_ok(),
        Plan::Direct => true,
    };
    println!(
        "Selected runner: {}",
        if selected { "usable" } else { "NOT usable" }
    );
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim(
        distro: Option<&str>,
        kernel: Option<&str>,
        runner: Option<Runner>,
        image: Option<&str>,
    ) -> Simulation {
        Simulation {
            distro: distro.map(Into::into),
            kernel: kernel.map(Into::into),
            runner,
            image: image.map(Into::into),
        }
    }

    #[test]
    fn plans_follow_the_runner_rules() {
        let (plan, _) = sim(Some("sles-12-sp5"), None, None, None).plan().unwrap();
        assert_eq!(
            plan,
            Plan::Container(
                "SUSE Linux Enterprise Server 12 SP5 (Linux 4.12)".into(),
                "registry.suse.com/suse/sles12sp5:latest".into()
            )
        );
        let (plan, target) = sim(Some("sles-12-sp5"), None, Some(Runner::Local), None)
            .plan()
            .unwrap();
        assert!(
            matches!(plan, Plan::Local(_)),
            "local ignores the preset image"
        );
        assert_eq!(target.unwrap().kernel.to_string(), "4.12");
        let (plan, _) = sim(None, Some("3.10"), None, None).plan().unwrap();
        assert_eq!(plan, Plan::Local("Linux 3.10".into()));
        let (plan, _) = sim(None, Some("3.10"), None, Some("centos:7"))
            .plan()
            .unwrap();
        assert_eq!(
            plan,
            Plan::Container("Linux 3.10".into(), "centos:7".into())
        );
        assert_eq!(sim(None, None, None, None).plan().unwrap().0, Plan::Direct);
        for bad in [
            sim(None, None, Some(Runner::Local), None),
            sim(None, None, None, Some("centos:7")),
            sim(None, Some("3.10"), Some(Runner::Container), None),
            sim(None, Some("3.10"), Some(Runner::Local), Some("centos:7")),
            sim(Some("rhel-7"), Some("3.10"), None, None),
            sim(Some("no-such-distro"), None, None, None),
        ] {
            assert!(bad.plan().is_err(), "{bad:?}");
        }
    }

    fn needs(entries: &[(&str, &str, bool)]) -> ElfInfo {
        ElfInfo {
            machine: 62,
            interpreter: None,
            needed: Vec::new(),
            rpath: Vec::new(),
            runpath: Vec::new(),
            versions: entries
                .iter()
                .map(|(library, name, weak)| crate::elf::VersionNeed {
                    library: (*library).into(),
                    name: (*name).into(),
                    weak: *weak,
                })
                .collect(),
            definitions: Vec::new(),
        }
    }

    #[test]
    fn host_glibc_compares_required_with_defined_versions() {
        let host = |library: &str| -> Result<Vec<String>, String> {
            match library {
                "libc.so.6" => Ok(vec!["GLIBC_2.2.5".into(), "GLIBC_2.17".into()]),
                "libm.so.6" => Ok(vec!["GLIBC_2.2.5".into()]),
                _ => Err(format!("cannot read {library}")),
            }
        };
        let fits = needs(&[
            ("libc.so.6", "GLIBC_2.17", false),
            ("libm.so.6", "GLIBC_2.2.5", false),
        ]);
        assert_eq!(host_glibc(&fits, host), HostGlibc::Satisfied);
        let too_new = needs(&[
            ("libc.so.6", "GLIBC_2.34", false),
            ("libm.so.6", "GLIBC_2.29", false),
            ("libc.so.6", "GLIBC_2.38", true),
            ("libgcc_s.so.1", "GCC_3.0", false),
        ]);
        let HostGlibc::Missing(missing) = host_glibc(&too_new, host) else {
            panic!("expected missing versions")
        };
        assert_eq!(missing, ["libc.so.6 GLIBC_2.34", "libm.so.6 GLIBC_2.29"]);
        assert_eq!(highest(&missing), Some("2.34".parse().unwrap()));
        let unreadable = needs(&[("libdl.so.2", "GLIBC_2.2.5", false)]);
        assert!(matches!(
            host_glibc(&unreadable, host),
            HostGlibc::Unknown(_)
        ));
    }

    #[test]
    fn search_dirs_follow_the_loader_order_and_expand_origin() {
        let mut info = needs(&[]);
        info.rpath.push("/r1:$ORIGIN/r2".into());
        let dirs = search_dirs(Path::new("/nonexistent/bin/app"), &info).unwrap();
        assert!(dirs.starts_with(&[PathBuf::from("/r1")]));
        assert!(dirs[1].ends_with("r2"), "{dirs:?}");
        // DT_RUNPATH disables DT_RPATH and comes after LD_LIBRARY_PATH.
        info.runpath.push("/run".into());
        let dirs = search_dirs(Path::new("/nonexistent/bin/app"), &info).unwrap();
        assert!(!dirs.contains(&PathBuf::from("/r1")));
        assert_eq!(dirs.last(), Some(&PathBuf::from("/run")));
        info.runpath.push("/x/$LIB".into());
        assert!(search_dirs(Path::new("/nonexistent/bin/app"), &info).is_err());
    }

    #[test]
    fn finds_the_runtime_in_engine_options() {
        let words = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        assert_eq!(
            runtime_from_engine_args(&words("--log-level warn --runtime /opt/crun")),
            Some("/opt/crun".into())
        );
        assert_eq!(
            runtime_from_engine_args(&words("--runtime=crun")),
            Some("crun".into())
        );
        assert_eq!(runtime_from_engine_args(&words("--log-level warn")), None);
    }

    #[test]
    fn selection_round_trips_through_the_environment() {
        let mut cmd = Command::new("x");
        let s = sim(Some("rhel-7"), None, Some(Runner::Local), None);
        s.set_environment(&mut cmd, "2.17".parse().unwrap());
        let envs: Vec<_> = cmd
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.map(|v| v.to_string_lossy().into_owned()),
                )
            })
            .collect();
        assert!(envs.contains(&(GLIBC_ENV.into(), Some("2.17".into()))));
        assert!(envs.contains(&(DISTRO_ENV.into(), Some("rhel-7".into()))));
        assert!(envs.contains(&(RUNNER_ENV.into(), Some("local".into()))));
        assert!(
            envs.contains(&(KERNEL_ENV.into(), None)),
            "stale values are removed"
        );
    }
}
