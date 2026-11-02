//! Bounded checks of the completed Clang linker command. No search path is
//! guessed: unresolved linker-built-in fallbacks are refused. Arguments and
//! linker state are inspected, never reordered or substituted here.
use crate::clock::{input_soname, Profile};
use std::{
    ffi::{OsStr, OsString},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

fn os(bytes: &[u8]) -> Result<OsString, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(OsString::from_vec(bytes.to_vec()))
    }
    #[cfg(not(unix))]
    {
        String::from_utf8(bytes.to_vec())
            .map(OsString::from)
            .map_err(|_| "non-Unicode linker trace on an unsupported host".into())
    }
}

/// Clang prints each command argument double-quoted. Reject other trace syntax
/// rather than interpreting shell expansions or silently losing OS bytes.
pub(crate) fn trace_command(stderr: &[u8], linker: &Path) -> Result<Vec<OsString>, String> {
    if stderr.len() > 4 * 1024 * 1024 {
        return Err("Clang linker trace exceeds 4 MiB".into());
    }
    let expected = fs::canonicalize(linker).map_err(|e| e.to_string())?;
    let mut selected = None;
    for line in stderr.split(|&b| b == b'\n') {
        let line = line.trim_ascii();
        if !line.starts_with(b"\"") {
            continue;
        }
        let first = quoted_args_limit(line, true)?;
        if first
            .first()
            .is_some_and(|a| fs::canonicalize(Path::new(a)).ok().as_ref() == Some(&expected))
        {
            let args = quoted_args(line)?;
            if selected.replace(args).is_some() {
                return Err("Clang trace has multiple selected linker commands".into());
            }
        }
    }
    selected.ok_or_else(|| "Clang did not expose one verifiable selected linker command; cannot validate named inputs".into())
}

fn quoted_args(line: &[u8]) -> Result<Vec<OsString>, String> {
    quoted_args_limit(line, false)
}

fn quoted_args_limit(mut line: &[u8], first_only: bool) -> Result<Vec<OsString>, String> {
    let mut args = Vec::new();
    while !line.trim_ascii_start().is_empty() {
        line = line.trim_ascii_start();
        if line.first() != Some(&b'"') {
            return Err("unsupported Clang command trace quoting".into());
        }
        line = &line[1..];
        let mut value = Vec::new();
        let mut closed = false;
        while let Some((&byte, rest)) = line.split_first() {
            line = rest;
            match byte {
                b'"' => {
                    closed = true;
                    break;
                }
                b'\\' => {
                    let (&next, rest) = line.split_first().ok_or("truncated Clang trace escape")?;
                    if !matches!(next, b'\\' | b'"' | b'$' | b'`') {
                        return Err("unsupported Clang trace escape".into());
                    }
                    value.push(next);
                    line = rest;
                }
                _ => value.push(byte),
            }
        }
        if !closed {
            return Err("unterminated Clang trace argument".into());
        }
        args.push(os(&value)?);
        if first_only {
            break;
        }
    }
    Ok(args)
}

fn rooted(path: &OsStr, root: &Path) -> Result<PathBuf, String> {
    let b = path.as_encoded_bytes();
    let relative = b.strip_prefix(b"=").or_else(|| b.strip_prefix(b"$SYSROOT"));
    Ok(match relative {
        Some(p) => {
            let p = PathBuf::from(os(p)?);
            root.join(p.strip_prefix("/").unwrap_or(&p))
        }
        None => PathBuf::from(path),
    })
}

fn takes_value(b: &[u8]) -> bool {
    matches!(
        b,
        b"-o"
            | b"-L"
            | b"--library-path"
            | b"-l"
            | b"--library"
            | b"-a"
            | b"--sysroot"
            | b"-m"
            | b"-z"
            | b"-u"
            | b"--undefined"
            | b"-e"
            | b"--entry"
            | b"-h"
            | b"-soname"
            | b"--soname"
            | b"-dynamic-linker"
            | b"--dynamic-linker"
            | b"-rpath"
            | b"--rpath"
            | b"-rpath-link"
            | b"--rpath-link"
            | b"-Map"
            | b"--Map"
            | b"--version-script"
            | b"--dynamic-list"
            | b"--retain-symbols-file"
            | b"--defsym"
            | b"--section-start"
            | b"--wrap"
            | b"--exclude-libs"
            | b"--exclude-symbols"
            | b"-Ttext"
            | b"-Tdata"
            | b"-Tbss"
            | b"-Ttext-segment"
            | b"-plugin"
            | b"--plugin"
            | b"-plugin-opt"
    )
}

struct Search<'a> {
    profile: &'a Profile,
    root: &'a Path,
    dirs: Vec<PathBuf>,
    visits: usize,
}
impl Search<'_> {
    fn library(&mut self, name: &OsStr, static_only: bool, depth: usize) -> Result<(), String> {
        let b = name.as_encoded_bytes();
        let names = if let Some(exact) = b.strip_prefix(b":") {
            if exact.contains(&b'/') {
                return Err(
                    "named library path contains '/'; pass a controlled explicit ELF path".into(),
                );
            }
            vec![os(exact)?]
        } else {
            if b.is_empty() || b.contains(&b'/') {
                return Err("ambiguous named library selector".into());
            }
            let mut archive = OsString::from("lib");
            archive.push(name);
            archive.push(".a");
            if static_only {
                vec![archive]
            } else {
                let mut shared = OsString::from("lib");
                shared.push(name);
                shared.push(".so");
                vec![shared, archive]
            }
        };
        for dir in &self.dirs {
            for name in &names {
                let file = dir.join(name);
                if file.is_file() {
                    return self.file(&file, static_only, depth);
                }
            }
        }
        Err(format!("cannot prove selected named library {name:?} in the completed Clang search paths; implicit linker-default fallback is unsupported, so pass a target ELF/archive path or an explicit -L directory"))
    }

    fn file(&mut self, file: &Path, static_only: bool, depth: usize) -> Result<(), String> {
        if depth != 0 {
            self.visits += 1;
        }
        if depth > 4 || self.visits > 4096 {
            return Err("named-library validation exceeds bounded script/input limits".into());
        }
        let canonical = fs::canonicalize(file)
            .map_err(|e| format!("cannot identify named input {}: {e}", file.display()))?;
        let mut header = Vec::new();
        fs::File::open(&canonical)
            .and_then(|f| f.take(8).read_to_end(&mut header))
            .map_err(|e| e.to_string())?;
        let binary = header.starts_with(b"\x7fELF")
            || header.starts_with(b"!<arch>\n")
            || header.starts_with(b"!<thin>\n")
            || header.starts_with(b"BC\xc0\xde")
            || header.starts_with(b"\xde\xc0\x17\x0b");
        if binary {
            let soname = input_soname(&canonical, false)?;
            if soname.as_deref() == Some("libc.so.6") && canonical != self.profile.libraries.libc {
                return Err(format!("named/script input {} is not the selected target libc; select a matching provider/sysroot", file.display()));
            }
            if soname.as_deref() == Some("librt.so.1") {
                let allowed = match &self.profile.overlay {
                    Some(dir) => ["librt.so", "librt.so.1"].iter().any(|name| {
                        fs::canonicalize(dir.join(name)).ok().as_ref() == Some(&canonical)
                    }),
                    None => canonical == self.profile.libraries.rt,
                };
                if !allowed {
                    return Err(format!("named/script input {} selects RT outside the chosen clock overlay; use the selected target RT by name or a controlled direct path", file.display()));
                }
            }
            return Ok(());
        }
        // Ordinary full sysroots use these GNU wrapper scripts. No arbitrary
        // application script gains acceptance merely by using INPUT/GROUP.
        let known = [
            "libc.so",
            "libm.so",
            "libgcc_s.so",
            "librt.so",
            "libpthread.so",
            "libdl.so",
            "libresolv.so",
            "libutil.so",
        ]
        .iter()
        .any(|n| file.file_name() == Some(OsStr::new(n)));
        if !known {
            return Err(format!("opaque named linker input {} is a script/nonbinary, not a verifiable ELF/archive; pass direct target inputs", file.display()));
        }
        let mut bytes = Vec::new();
        fs::File::open(&canonical)
            .and_then(|f| f.take(65537).read_to_end(&mut bytes))
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("standard library script exceeds 64 KiB".into());
        }
        let tokens = script_tokens(&bytes)?;
        let mut at = 0;
        while at < tokens.len() {
            let command = tokens[at].as_slice();
            at += 1;
            if command == b";" {
                continue;
            }
            if !matches!(command, b"GROUP" | b"INPUT" | b"OUTPUT_FORMAT") {
                return Err("unsupported standard-library linker script command".into());
            }
            expect(&tokens, &mut at, b"(")?;
            if command == b"OUTPUT_FORMAT" {
                expect(&tokens, &mut at, b"elf64-x86-64")?;
                expect(&tokens, &mut at, b")")?;
            } else {
                self.script_inputs(&tokens, &mut at, &canonical, static_only, depth + 1)?;
            }
        }
        Ok(())
    }

    fn script_inputs(
        &mut self,
        tokens: &[Vec<u8>],
        at: &mut usize,
        script: &Path,
        static_only: bool,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 4 {
            return Err("nested standard-library script limit exceeded".into());
        }
        while let Some(token) = tokens.get(*at) {
            *at += 1;
            if token == b")" {
                return Ok(());
            }
            if token == b"," {
                continue;
            }
            if token == b"AS_NEEDED" {
                expect(tokens, at, b"(")?;
                self.script_inputs(tokens, at, script, static_only, depth + 1)?;
                continue;
            }
            if token == b"(" || token == b";" {
                return Err("unsupported standard-library script syntax".into());
            }
            if let Some(name) = token.strip_prefix(b"-l") {
                if name != b"gcc" {
                    return Err("only the standard -lgcc script leaf is supported; pass other selected inputs explicitly".into());
                }
                self.library(&os(name)?, static_only, depth)?;
                continue;
            }
            if token.starts_with(b"-") {
                return Err("unsupported option in standard-library script".into());
            }
            let path = os(token)?;
            let file = if Path::new(&path).is_absolute() && script.starts_with(self.root) {
                self.root.join(
                    Path::new(&path)
                        .strip_prefix("/")
                        .map_err(|e| e.to_string())?,
                )
            } else if token.starts_with(b"=") || token.starts_with(b"$SYSROOT") {
                rooted(&path, self.root)?
            } else if Path::new(&path).is_absolute() {
                PathBuf::from(&path)
            } else {
                let mut paths = vec![
                    script.parent().unwrap_or(Path::new(".")).join(&path),
                    PathBuf::from(&path),
                ];
                paths.extend(self.dirs.iter().map(|d| d.join(&path)));
                paths
                    .into_iter()
                    .find(|p| p.is_file())
                    .ok_or("unresolved standard-library script leaf; no implicit fallback")?
            };
            self.file(&file, static_only, depth)?;
        }
        Err("unterminated standard-library script group".into())
    }
}

fn expect(tokens: &[Vec<u8>], at: &mut usize, value: &[u8]) -> Result<(), String> {
    if tokens.get(*at).map(Vec::as_slice) != Some(value) {
        return Err("unsupported standard-library linker script syntax".into());
    }
    *at += 1;
    Ok(())
}

fn script_tokens(mut bytes: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut tokens = Vec::new();
    while !bytes.is_empty() {
        bytes = bytes.trim_ascii_start();
        if bytes.is_empty() {
            break;
        }
        if bytes.starts_with(b"/*") {
            let end = bytes
                .windows(2)
                .position(|b| b == b"*/")
                .ok_or("unterminated library-script comment")?;
            bytes = &bytes[end + 2..];
            continue;
        }
        if b"(),;".contains(&bytes[0]) {
            tokens.push(vec![bytes[0]]);
            bytes = &bytes[1..];
            continue;
        }
        if bytes[0] == b'"' {
            bytes = &bytes[1..];
            let end = bytes
                .iter()
                .position(|&b| b == b'"')
                .ok_or("unterminated library-script quoted path")?;
            if bytes[..end].contains(&b'\\') {
                return Err("escaped standard-library script paths are unsupported".into());
            }
            tokens.push(bytes[..end].to_vec());
            bytes = &bytes[end + 1..];
            continue;
        }
        let end = bytes
            .iter()
            .position(|b| b.is_ascii_whitespace() || b"(),;".contains(b))
            .unwrap_or(bytes.len());
        tokens.push(bytes[..end].to_vec());
        bytes = &bytes[end..];
    }
    Ok(tokens)
}

/// Validate effective ld arguments, including compiler-injected default libs.
pub(crate) fn validate(profile: &Profile, args: &[OsString], root: &Path) -> Result<(), String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let mut dirs = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = args[i].as_encoded_bytes();
        i += 1;
        if b == b"--sysroot" || b.starts_with(b"--sysroot=") {
            let value = if let Some(value) = b.strip_prefix(b"--sysroot=") {
                os(value)?
            } else {
                let value = args.get(i).ok_or("missing effective sysroot")?.clone();
                i += 1;
                value
            };
            if fs::canonicalize(Path::new(&value)).map_err(|e| e.to_string())? != root {
                return Err("effective linker sysroot differs from selected clock inputs".into());
            }
            continue;
        }
        let path = if matches!(b, b"-L" | b"--library-path") {
            let p = args.get(i).ok_or("missing linker search path")?;
            i += 1;
            Some(p.clone())
        } else if takes_value(b) {
            if i == args.len() {
                return Err("missing linker option value during search-path discovery".into());
            }
            i += 1;
            None
        } else {
            b.strip_prefix(b"-L")
                .or_else(|| b.strip_prefix(b"--library-path="))
                .map(os)
                .transpose()?
        };
        if let Some(path) = path {
            dirs.push(rooted(&path, &root)?);
        }
    }
    let mut search = Search {
        profile,
        root: &root,
        dirs,
        visits: 0,
    };
    let mut static_only = false;
    let mut stack = Vec::new();
    i = 0;
    while i < args.len() {
        let b = args[i].as_encoded_bytes();
        i += 1;
        match b {
            b"--push-state" => stack.push(static_only),
            b"--pop-state" => {
                static_only = stack.pop().ok_or("unbalanced linker input-state pop")?
            }
            b"-Bstatic" | b"-dn" | b"-non_shared" | b"-static" | b"--static" | b"-n" | b"-N"
            | b"--nmagic" | b"--omagic" | b"--Bstatic" => static_only = true,
            b"-Bdynamic" | b"-dy" | b"-call_shared" | b"--Bdynamic" => static_only = false,
            b"-a" => {
                let mode = args
                    .get(i)
                    .ok_or("missing archive search mode")?
                    .as_encoded_bytes();
                i += 1;
                static_only = match mode {
                    b"archive" => true,
                    b"shared" | b"default" => false,
                    _ => return Err("unknown archive search mode".into()),
                };
            }
            b"-aarchive" => static_only = true,
            b"-ashared" | b"-adefault" => static_only = false,
            b"-l" | b"--library" => {
                let name = args.get(i).ok_or("missing named library")?;
                i += 1;
                search.library(name, static_only, 0)?;
            }
            b"-o"
            | b"-L"
            | b"--library-path"
            | b"-m"
            | b"-z"
            | b"-u"
            | b"-e"
            | b"-h"
            | b"-soname"
            | b"--soname"
            | b"-dynamic-linker"
            | b"--dynamic-linker"
            | b"-rpath"
            | b"--rpath"
            | b"-rpath-link"
            | b"--rpath-link"
            | b"-Map"
            | b"--Map"
            | b"--version-script"
            | b"--dynamic-list"
            | b"--retain-symbols-file"
            | b"--defsym"
            | b"--section-start"
            | b"-Ttext"
            | b"-Tdata"
            | b"-Tbss"
            | b"-Ttext-segment"
            | b"-plugin"
            | b"--plugin"
            | b"-plugin-opt" => {
                if i == args.len() {
                    return Err("missing linker option operand".into());
                }
                i += 1;
            }
            _ => {
                if let Some(name) = b
                    .strip_prefix(b"-l")
                    .or_else(|| b.strip_prefix(b"--library="))
                {
                    search.library(&os(name)?, static_only, 0)?;
                } else if b.starts_with(b"@")
                    || b.starts_with(b"--script")
                    || b.starts_with(b"--default-script")
                    || b.starts_with(b"--mri-script")
                    || b == b"-c"
                    || b.starts_with(b"-dT")
                    || (b.starts_with(b"-T")
                        && ![
                            b"-Ttext=".as_slice(),
                            b"-Tdata=",
                            b"-Tbss=",
                            b"-Ttext-segment=",
                        ]
                        .iter()
                        .any(|p| b.starts_with(p)))
                {
                    return Err("opaque script/response in completed linker command".into());
                } else if !b.starts_with(b"-") && Path::new(&args[i - 1]).is_file() {
                    search.file(Path::new(&args[i - 1]), static_only, 0)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    #[cfg(unix)]
    fn clang_trace_keeps_os_bytes_and_requires_one_selected_command() {
        let args = quoted_args(b"\"ld\" \"-L/native with spaces\" \"raw-\xff.o\" \"a\\\"b\\\\c\"")
            .unwrap();
        assert_eq!(args[1], "-L/native with spaces");
        assert_eq!(args[2].as_encoded_bytes(), b"raw-\xff.o");
        assert_eq!(args[3], "a\"b\\c");
        assert!(quoted_args(b"\"ld\" unquoted").is_err());
        let executable = std::env::current_exe().unwrap();
        let trace = format!(
            "clang version test\n \"{}\" \"-L/native\" \"-lfoo\"\n",
            executable.display()
        );
        assert_eq!(
            trace_command(trace.as_bytes(), &executable).unwrap()[2],
            "-lfoo"
        );
        assert!(trace_command((trace.clone() + &trace).as_bytes(), &executable).is_err());
    }
}
