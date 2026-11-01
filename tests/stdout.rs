//! Process-level stdout checks; no target compiler or provider preparation needed.
#![cfg(unix)]

use std::{
    fs,
    os::{fd::OwnedFd, unix::net::UnixStream},
    path::PathBuf,
    process::{Command, Output, Stdio},
};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-libc"))
}

fn closed_stdout(cmd: &mut Command) -> Output {
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader); // Closed before spawn: no scheduler-dependent race with head.
    cmd.stdout(Stdio::from(OwnedFd::from(writer)))
        .output()
        .unwrap()
}

fn assert_status(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
}

struct Elf(PathBuf);

impl Elf {
    fn new() -> Self {
        // An x86_64 ELF requiring GLIBC_2.34, with DT_RELR (needs 2.36).
        let mut bytes = vec![0_u8; 512];
        bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        let put = |bytes: &mut [u8], at: usize, data: &[u8]| {
            bytes[at..at + data.len()].copy_from_slice(data);
        };
        put(&mut bytes, 16, &3_u16.to_le_bytes());
        put(&mut bytes, 18, &62_u16.to_le_bytes());
        put(&mut bytes, 20, &1_u32.to_le_bytes());
        put(&mut bytes, 32, &64_u64.to_le_bytes());
        put(&mut bytes, 52, &64_u16.to_le_bytes());
        put(&mut bytes, 54, &56_u16.to_le_bytes());
        put(&mut bytes, 56, &2_u16.to_le_bytes());
        for (at, kind, offset, size) in [(64, 1_u32, 0_u64, 512_u64), (120, 2, 176, 96)] {
            put(&mut bytes, at, &kind.to_le_bytes());
            put(&mut bytes, at + 8, &offset.to_le_bytes());
            put(&mut bytes, at + 16, &offset.to_le_bytes());
            put(&mut bytes, at + 32, &size.to_le_bytes());
            put(&mut bytes, at + 40, &size.to_le_bytes());
        }
        let strings = b"\0libc.so.6\0GLIBC_2.34\0";
        put(&mut bytes, 288, strings);
        for (i, (tag, value)) in [
            (5_u64, 288_u64),
            (10, strings.len() as u64),
            (0x6fff_fffe, 320),
            (0x6fff_ffff, 1),
            (36, 0),
        ]
        .into_iter()
        .enumerate()
        {
            put(&mut bytes, 176 + i * 16, &tag.to_le_bytes());
            put(&mut bytes, 184 + i * 16, &value.to_le_bytes());
        }
        put(&mut bytes, 320, &1_u16.to_le_bytes());
        put(&mut bytes, 322, &1_u16.to_le_bytes());
        put(&mut bytes, 324, &1_u32.to_le_bytes());
        put(&mut bytes, 328, &16_u32.to_le_bytes());
        put(&mut bytes, 342, &2_u16.to_le_bytes());
        put(&mut bytes, 344, &11_u32.to_le_bytes());
        // Null section plus a string table: enough for the tier 1 parser.
        put(&mut bytes, 40, &384_u64.to_le_bytes());
        put(&mut bytes, 58, &64_u16.to_le_bytes());
        put(&mut bytes, 60, &2_u16.to_le_bytes());
        put(&mut bytes, 62, &1_u16.to_le_bytes());
        put(&mut bytes, 452, &3_u32.to_le_bytes());
        put(&mut bytes, 472, &288_u64.to_le_bytes());
        put(&mut bytes, 480, &(strings.len() as u64).to_le_bytes());
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cargo-libc-stdout-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::write(&path, bytes).unwrap();
        Self(path)
    }
}

impl Drop for Elf {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

#[test]
fn help_version_and_provider_help_allow_closed_stdout() {
    for args in [
        vec!["--help"],
        vec!["--version"],
        vec!["provider", "--help"],
    ] {
        let normal = command().args(&args).output().unwrap();
        assert_status(&normal, 0);
        assert!(!normal.stdout.is_empty());
        assert!(normal.stderr.is_empty());
        let closed = closed_stdout(command().args(&args));
        assert_status(&closed, 0);
        assert!(closed.stderr.is_empty());
    }
}

#[test]
fn inspection_results_survive_closed_stdout() {
    let elf = Elf::new();
    for json in [false, true] {
        for (ceiling, code) in [("2.34", 0), ("2.17", 2)] {
            let mut args = vec!["inspect", elf.0.to_str().unwrap(), "--max-glibc", ceiling];
            if json {
                args.push("--json");
            }
            let normal = command().args(&args).output().unwrap();
            assert_status(&normal, code);
            assert!(String::from_utf8_lossy(&normal.stdout).contains("GLIBC_2.34"));
            assert!(normal.stderr.is_empty());
            let closed = closed_stdout(command().args(&args));
            assert_status(&closed, code);
            assert!(closed.stderr.is_empty());
        }
    }
    let args = [
        "inspect",
        elf.0.to_str().unwrap(),
        "--max-glibc",
        "2.34",
        "--scan",
    ];
    assert_status(&command().args(args).output().unwrap(), 2);
    assert_status(&closed_stdout(command().args(args)), 2);
}

#[test]
fn command_errors_keep_stderr_and_failure_status() {
    let output = closed_stdout(command().arg("not-a-command"));
    assert_status(&output, 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown command"));
}

#[cfg(target_os = "linux")]
#[test]
fn non_pipe_stdout_errors_fail() {
    let full = fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let output = command().arg("--help").stdout(full).output().unwrap();
    assert_status(&output, 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot write stdout"));
}

fn closed_stderr(cmd: &mut Command) -> Output {
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader);
    cmd.stderr(Stdio::from(OwnedFd::from(writer)))
        .output()
        .unwrap()
}

fn owned_failure(binary: &str) -> Command {
    let mut cmd = Command::new(binary);
    // Exercise missing wrapper configuration rather than inherited build settings.
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("CARGO_LIBC_") {
            cmd.env_remove(key);
        }
    }
    cmd.env_remove("CARGO_CFG_CARGO_LIBC_PROFILE");
    if binary == env!("CARGO_BIN_EXE_cargo-libc") {
        cmd.arg("not-a-command");
    }
    cmd
}

#[test]
fn all_five_final_errors_preserve_status_with_unwritable_stderr() {
    for binary in [
        env!("CARGO_BIN_EXE_cargo-libc"),
        env!("CARGO_BIN_EXE_libc-linker"),
        env!("CARGO_BIN_EXE_libc-runner"),
        env!("CARGO_BIN_EXE_libc-cc"),
        env!("CARGO_BIN_EXE_libc-cxx"),
    ] {
        let normal = owned_failure(binary).output().unwrap();
        assert_status(&normal, 1);
        assert!(!normal.stderr.is_empty(), "{binary}");
        assert_status(&closed_stderr(&mut owned_failure(binary)), 1);
        #[cfg(target_os = "linux")]
        {
            let full = fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap();
            assert_status(&owned_failure(binary).stderr(full).output().unwrap(), 1);
        }
    }
}

#[test]
fn runner_refusal_keeps_status_two_with_unwritable_stderr() {
    let elf = Elf::new();
    let command = || {
        let mut cmd = owned_failure(env!("CARGO_BIN_EXE_libc-runner"));
        cmd.arg(&elf.0).env("CARGO_LIBC_RUN_GLIBC", "2.17");
        cmd
    };
    assert_status(&command().output().unwrap(), 2);
    assert_status(&closed_stderr(&mut command()), 2);
    #[cfg(target_os = "linux")]
    {
        let full = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        assert_status(&command().stderr(full).output().unwrap(), 2);
    }
}

#[test]
fn compiler_child_status_survives_unwritable_stderr() {
    for (binary, key) in [
        (env!("CARGO_BIN_EXE_libc-cc"), "CARGO_LIBC_HOST_CC"),
        (env!("CARGO_BIN_EXE_libc-cxx"), "CARGO_LIBC_HOST_CXX"),
    ] {
        let command = || {
            let mut cmd = owned_failure(binary);
            cmd.env(key, "/bin/sh").args(["-c", "exit 7"]);
            cmd
        };
        assert_status(&closed_stderr(&mut command()), 7);
        #[cfg(target_os = "linux")]
        {
            let full = fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap();
            assert_status(&command().stderr(full).output().unwrap(), 7);
        }
    }
}
