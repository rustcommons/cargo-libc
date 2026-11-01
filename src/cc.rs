//! Compiler wrappers for C dependencies (`libc-cc`, and `libc-cxx` to refuse C++).
//!
//! `cargo libc` points the `cc` crate (and CMake through it) at these wrappers.
//! Cargo builds host and target units for the same triple, so the triple cannot
//! tell them apart; the wrapper uses the build script's `CARGO_CFG_CARGO_LIBC_PROFILE`,
//! which only target units have (it comes from the target rustflags). Target
//! compiles get the selected sysroot or provider; host compiles run the host
//! compiler unchanged.
use crate::toolchain;
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

/// Set in the build script environment of target units only.
pub(crate) const TARGET_MARKER: &str = "CARGO_CFG_CARGO_LIBC_PROFILE";
pub(crate) const HOST_CC_ENV: &str = "CARGO_LIBC_HOST_CC";
pub(crate) const HOST_CXX_ENV: &str = "CARGO_LIBC_HOST_CXX";

/// The language a wrapper compiles: which executable is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    /// C, as `libc-cc`.
    C,
    /// C++, as `libc-cxx`: host units only; target units are refused.
    Cxx,
}

/// The `cc` crate's compiler variables, most specific first; with the host
/// triple equal to the target triple it reads `HOST_CC`, never `TARGET_CC`.
fn compiler_variables(base: &str) -> [String; 4] {
    let triple = toolchain::TARGET;
    [
        format!("{base}_{triple}"),
        format!("{base}_{}", triple.replace('-', "_")),
        format!("HOST_{base}"),
        base.to_string(),
    ]
}

/// The variables pointing `CC`/`CXX` for the target triple at the wrappers.
/// The host's own choice (`HOST_CC`, `CC`, or the `cc` crate's default) is kept
/// for host units. `shell_names` uses `CC_x86_64_unknown_linux_gnu`, which a
/// shell can export, instead of the spelling the `cc` crate reads first.
pub(crate) fn variables(
    cc: &Path,
    cxx: &Path,
    shell_names: bool,
) -> Result<Vec<(String, OsString)>, String> {
    let mut variables = Vec::new();
    for (base, wrapper, host_env, default) in [
        ("CC", cc, HOST_CC_ENV, "cc"),
        ("CXX", cxx, HOST_CXX_ENV, "c++"),
    ] {
        let [hyphen, underscore, host, plain] = compiler_variables(base);
        for key in [&hyphen, &underscore] {
            if env::var_os(key).is_some_and(|value| !value.is_empty()) {
                return Err(format!("{key} is already set; cargo-libc selects the target C compiler itself, so use a shell without that override"));
            }
        }
        let host_compiler = [&host, &plain]
            .into_iter()
            .find_map(|key| env::var_os(key).filter(|value| !value.is_empty()))
            .unwrap_or_else(|| default.into());
        variables.push((host_env.to_string(), host_compiler));
        let key = if shell_names { underscore } else { hyphen };
        variables.push((key, wrapper.as_os_str().to_owned()));
    }
    Ok(variables)
}

pub(crate) fn set_environment(cmd: &mut Command, cc: &Path, cxx: &Path) -> Result<(), String> {
    cmd.envs(variables(cc, cxx, false)?);
    Ok(())
}

/// What a driver invocation does with these arguments.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Preprocess or query (`-E`, `--version`, `-print-*`, no inputs).
    Query,
    /// Compile only (`-c`, `-S`, `-fsyntax-only`): no target link inputs are added.
    Compile,
    Link {
        shared: bool,
    },
    /// `-r` partial links take no startup files or libraries.
    Relocatable,
}

/// Driver options whose value is the next argument.
const SEPARATE_VALUE: &[&str] = &[
    "-o",
    "-I",
    "-L",
    "-D",
    "-U",
    "-include",
    "-imacros",
    "-isystem",
    "-iquote",
    "-idirafter",
    "-iprefix",
    "-isysroot",
    "-x",
    "-MF",
    "-MT",
    "-MQ",
    "-Xlinker",
    "-Xassembler",
    "-Xpreprocessor",
    "-Xclang",
    "-target",
    "-T",
    "-z",
    "-u",
    "-e",
    "-arch",
];

pub(crate) fn classify(args: &[OsString]) -> Result<Action, String> {
    let mut inputs = 0;
    let mut shared = false;
    let mut relocatable = false;
    let mut compile = false;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].to_string_lossy();
        match arg.as_ref() {
            "-E" | "-M" | "-MM" | "--version" | "-dumpversion" | "-dumpfullversion"
            | "-dumpmachine" | "-###" => return Ok(Action::Query),
            "-c" | "-S" | "-fsyntax-only" => compile = true,
            "-static" | "-static-pie" => {
                return Err(
                    "static linking is not supported: providers have no static glibc".into(),
                )
            }
            "-shared" => shared = true,
            "-r" => relocatable = true,
            _ if arg.starts_with("-print-") || arg.starts_with("--print-") => {
                return Ok(Action::Query)
            }
            _ if SEPARATE_VALUE.contains(&arg.as_ref()) => i += 1,
            // `-` is standard input; `@file` may hold inputs (CMake response files).
            _ if arg == "-" || !arg.starts_with('-') => inputs += 1,
            _ => {}
        }
        i += 1;
    }
    Ok(if inputs == 0 {
        Action::Query
    } else if compile {
        Action::Compile
    } else if relocatable {
        Action::Relocatable
    } else {
        Action::Link { shared }
    })
}

/// C++ needs a target C++ runtime, which providers do not have.
fn is_cxx(args: &[OsString]) -> bool {
    let mut language_cxx = false;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].to_string_lossy();
        i += 1;
        let language = if arg == "-x" {
            i += 1;
            Some(
                args.get(i - 1)
                    .map_or(String::new(), |v| v.to_string_lossy().into_owned()),
            )
        } else {
            arg.strip_prefix("-x").map(str::to_string)
        };
        if let Some(language) = language {
            language_cxx = language.starts_with("c++");
            continue;
        }
        if SEPARATE_VALUE.contains(&arg.as_ref()) {
            i += 1;
            continue;
        }
        if arg.starts_with('-') && arg != "-" {
            continue;
        }
        let ext = Path::new(arg.as_ref())
            .extension()
            .map(|e| e.to_string_lossy().into_owned());
        if language_cxx
            || matches!(
                ext.as_deref(),
                Some("cc" | "cp" | "cpp" | "cxx" | "CPP" | "c++" | "C" | "ii" | "cppm" | "ixx")
            )
        {
            return true;
        }
    }
    false
}

/// Builds the compiler command for one wrapper invocation.
pub(crate) fn command(language: Language, args: Vec<OsString>) -> Result<Command, String> {
    if env::var_os(TARGET_MARKER).is_none() {
        let key = match language {
            Language::C => HOST_CC_ENV,
            Language::Cxx => HOST_CXX_ENV,
        };
        let host = env::var_os(key)
            .ok_or_else(|| format!("{key} is missing; this wrapper is launched by cargo libc"))?;
        // A host compiler variable may hold a wrapper and its compiler ("ccache cc").
        let text = host.to_string_lossy().into_owned();
        let mut words = text.split_whitespace();
        let program = words.next().ok_or(format!("{key} is empty"))?;
        let mut cmd = Command::new(program);
        cmd.args(words).args(args);
        return Ok(cmd);
    }
    let action = classify(&args)?;
    // Queries (the `cc` crate's compiler detection) still answer for libc-cxx.
    if action != Action::Query && (language == Language::Cxx || is_cxx(&args)) {
        return Err("C++ for the target is not supported yet: providers have no target C++ runtime. Build only C for the target (CMake: project(NAME C))".into());
    }
    toolchain::target_compiler(action, args)
}

/// The `libc-cc` and `libc-cxx` entry point. Compiles with the arguments the
/// process was started with: target units with Clang and the selected sysroot or
/// provider, host units with the host compiler unchanged. Returns the compiler's
/// exit code, or 1 after printing why the compile could not run (for example C++
/// for the target).
pub fn run(language: Language) -> i32 {
    let name = match language {
        Language::C => "libc-cc",
        Language::Cxx => "libc-cxx",
    };
    let result = command(language, env::args_os().skip(1).collect()).and_then(|mut cmd| {
        cmd.status()
            .map_err(|e| format!("cannot run {:?}: {e}", cmd.get_program()))
    });
    match result {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            // Preserve the command failure even when its diagnostic cannot be written.
            let _ = crate::output::diagnostic(format_args!("{name}: {error}"));
            1
        }
    }
}

/// The wrapper paths next to the running executable.
pub(crate) fn wrappers() -> Result<(PathBuf, PathBuf), String> {
    Ok((
        toolchain::sibling("libc-cc")?,
        toolchain::sibling("libc-cxx")?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn compiles_queries_and_links_are_told_apart() {
        assert_eq!(
            classify(&args(&["-c", "a.c", "-o", "a.o"])).unwrap(),
            Action::Compile
        );
        for query in [
            &["-E", "probe.c"][..],
            &["--version"],
            &["-v"],
            &["-print-file-name=crt1.o"],
            &["-o", "out", "-I", "include"],
        ] {
            assert_eq!(classify(&args(query)).unwrap(), Action::Query, "{query:?}");
        }
        assert_eq!(
            classify(&args(&["a.o", "-o", "cmTC_1"])).unwrap(),
            Action::Link { shared: false }
        );
        assert_eq!(
            classify(&args(&["-shared", "a.o", "-o", "liba.so"])).unwrap(),
            Action::Link { shared: true }
        );
        assert_eq!(
            classify(&args(&["@objects.rsp", "-o", "app"])).unwrap(),
            Action::Link { shared: false }
        );
        assert_eq!(
            classify(&args(&["-r", "a.o", "-o", "b.o"])).unwrap(),
            Action::Relocatable
        );
        assert!(classify(&args(&["-static", "a.o"])).is_err());
    }

    #[test]
    fn cxx_sources_are_recognized() {
        assert!(is_cxx(&args(&["-c", "a.cpp"])));
        assert!(is_cxx(&args(&["-x", "c++", "-c", "a.h"])));
        assert!(is_cxx(&args(&["-xc++", "-"])));
        assert!(!is_cxx(&args(&["-c", "a.c", "-o", "a.o"])));
        assert!(!is_cxx(&args(&["-x", "c++", "-x", "c", "a.c"])));
    }
}
