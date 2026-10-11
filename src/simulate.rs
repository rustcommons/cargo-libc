//! Tier 2: `cargo libc test` and `cargo libc run` execute each target binary
//! through the `libc-runner` helper, which Cargo calls as its target runner.
//!
//! The helper first applies the direct glibc check to the binary (a newer
//! host glibc would otherwise hide a too-new requirement under the local
//! runner), then runs it under the kernel simulation from `kernel-abi-tools`:
//! the local seccomp filter (host userland, no root) or a container with the
//! target's userland. The selection travels from `cargo libc` to the helper
//! in `CARGO_LIBC_RUN_*` environment variables.

use crate::{report, version::Version};
use kernel_abi_tools::{
    container::{self, Engine},
    local, table_floor, KernelVersion, Runner, Target,
};
use std::{env, ffi::OsString, path::Path, process::Command};

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
