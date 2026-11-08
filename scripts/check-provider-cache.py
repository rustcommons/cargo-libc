#!/usr/bin/env python3
"""Installed Cargo/provider-generation regression; x86_64 Linux, GCC and Clang.

Usage: check-provider-cache.py TOOLS_DIR CLANG EVIDENCE
The evidence directory must not exist. No tool rebuild or cache removal occurs.
"""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time
import tomllib

tools = Path(sys.argv[1]).resolve()
clang = str(Path(sys.argv[2]).resolve())
root = Path(sys.argv[3]).resolve()
root.mkdir(parents=True, exist_ok=False)
repo = Path(__file__).resolve().parents[1]
env = os.environ.copy()
for key in list(env):
    if key.startswith("CARGO_LIBC_") or key in {
        "RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET",
        "CARGO_TARGET_DIR", "CC", "CXX", "CMAKE_TOOLCHAIN_FILE",
    } or key.startswith(("CC_", "CXX_", "CARGO_TARGET_")):
        env.pop(key, None)
env.update(PATH=str(tools) + os.pathsep + env["PATH"],
           CARGO_LIBC_PROVIDER_DIR=str(root / "cache"),
           CARGO_BUILD_JOBS="2", RUST_TEST_THREADS="2", CARGO_TERM_COLOR="never")
tool = str(tools / "cargo-libc")
commands = []


def run(label, args, cwd=repo):
    started = time.monotonic()
    result = subprocess.run(list(map(str, args)), cwd=cwd, env=env,
                            capture_output=True, timeout=300)
    (root / (label + ".stdout")).write_bytes(result.stdout)
    (root / (label + ".stderr")).write_bytes(result.stderr)
    commands.append(dict(label=label, args=list(map(str, args)),
                         exit=result.returncode,
                         seconds=round(time.monotonic() - started, 2)))
    (root / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
    assert result.returncode == 0, (label, result.stderr.decode(errors="replace"))
    return result.stdout


project = root / "probe"
(project / "src").mkdir(parents=True)
(project / "Cargo.toml").write_text(
    '[package]\nname="provider-cache-probe"\nversion="0.0.0"\nedition="2021"\n')
(project / "src/main.rs").write_text(
    'fn main() { assert_eq!(std::thread::spawn(|| 42).join().unwrap(), 42); '
    'println!("provider-cache-probe:42"); }\n')
run("probe-lock", ["cargo", "generate-lockfile", "--offline"], project)
selection = ["--glibc", "2.17", "--cc", clang]
manifest = ["--manifest-path", project / "Cargo.toml", "--locked", "--offline"]
target = root / "managed-target"
artifact = target / "x86_64-unknown-linux-gnu/debug/provider-cache-probe"


def build(label):
    before = artifact.stat().st_mtime_ns if artifact.exists() else None
    run(label, [tool, "build", *selection, "--artifact", artifact, "--",
                *manifest, "--target-dir", target, "--verbose"])
    diagnostics = (root / (label + ".stderr")).read_bytes()
    compiled = b"Compiling provider-cache-probe " in diagnostics
    fresh = b"Fresh provider-cache-probe " in diagnostics
    assert compiled != fresh, (label, diagnostics.decode(errors="replace"))
    assert fresh == (before == artifact.stat().st_mtime_ns), label
    return fresh


assert build("managed-first") is False
old = Path(run("old-path", [tool, "provider", "path", "--glibc", "2.17"]).decode().strip())
assert old.parent.name.startswith(".gen-source-")
old_config = run("old-config", [tool, "env", *selection, "--format", "cargo"])
config = root / "old-provider.toml"
config.write_bytes(old_config)
tomllib.loads(old_config.decode())
assert build("managed-cached") is True
plain = ["cargo", "build", "--config", config, "--target", "x86_64-unknown-linux-gnu",
         *manifest, "--target-dir", root / "plain-target"]
run("plain-before", plain)
held = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in [
    old, old.with_name("manifest.json"), old.parent / "stubs/libc.so.6",
    old.parent / "sysroot/usr/include/stdio.h", old.parent / "sysroot/usr/lib64/crt1.o"]}
new = Path(run("force", [tool, "provider", "prepare", "--glibc", "2.17", "--force"]).decode().strip())
assert new != old and new.parent.name.startswith(".gen-source-")
for p, digest in held.items():
    assert hashlib.sha256(p.read_bytes()).hexdigest() == digest, p
assert run("new-path", [tool, "provider", "path", "--glibc", "2.17"]).decode().strip() == str(new)
new_config = run("new-config", [tool, "env", *selection, "--format", "cargo"])
tomllib.loads(new_config.decode())
assert old_config != new_config
assert build("managed-new-generation") is False
assert build("managed-new-cached") is True
run("plain-after", plain)
executable = root / "plain-target/x86_64-unknown-linux-gnu/debug/provider-cache-probe"
assert run("plain-run-after", [executable]) == b"provider-cache-probe:42\n"
assert run("explicit-old-run", [tool, "run", "--provider", old, *selection, "--",
                               *manifest, "--target-dir", root / "explicit-target"]) == b"provider-cache-probe:42\n"
output = run("explicit-old-native-c", [tool, "run", "--provider", old, *selection, "--",
             "--manifest-path", repo / "examples/c-deps/Cargo.toml", "--locked", "--offline",
             "--target-dir", root / "native-target"])
assert b"cc: glibc 2.17 headers" in output and b"cmake: glibc 2.17 headers" in output
run("verify-current", [tool, "provider", "verify", "--glibc", "2.17"])
summary = dict(status="PASS", old_descriptor=str(old), new_descriptor=str(new),
               held_files_preserved=len(held), managed_freshness=[False, True, False, True],
               saved_plain_config="old generation remains runnable",
               explicit_old="Rust and C/CMake passed", commands=len(commands),
               scope="Source mode on modern x86_64 Linux; no removal/GC/container/old-OS claim")
(root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
