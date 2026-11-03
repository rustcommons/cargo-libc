#!/usr/bin/env python3
"""Qualify clean committed producer/consumer sources in disposable snapshots.

This is source evidence, not registry release qualification. No tracked lockfile
or caller checkout is modified. Requires existing Rust 1.90 plus 1.99 rustfmt/Clippy, GCC and GNU tools.
"""
from pathlib import Path
import hashlib
import json
import os
import platform
import subprocess
import sys
import time

if len(sys.argv) != 4:
    raise SystemExit("usage: qualify-target-libgcc.py CARGO_CHECKOUT PRODUCER_CHECKOUT EVIDENCE_DIR")
if platform.system() != "Linux" or platform.machine() not in ("x86_64", "amd64"):
    raise SystemExit("native fixture qualification requires x86-64 Linux; no platform qualification inferred")
consumer, producer, out = map(lambda x: Path(x).resolve(), sys.argv[1:])
out.mkdir(exist_ok=False, parents=True)
record = {"scope": "source-only patched qualification; not registry release or runtime qualification", "sources": {}, "commands": []}
env = os.environ.copy()
for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR"):
    env.pop(key, None)
env.update(RUSTUP_TOOLCHAIN="1.90.0", CARGO_BUILD_JOBS="2", RUST_TEST_THREADS="2", CARGO_TARGET_DIR=str(out / "target"))

def run(name, command, cwd, timeout=600, **kw):
    print(name, flush=True)
    started = time.monotonic()
    p = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout, **kw)
    (out / (name + ".log")).write_text(p.stdout + "\nSTDERR\n" + p.stderr)
    record["commands"].append(dict(name=name, command=command, cwd=str(cwd), exit=p.returncode, seconds=time.monotonic()-started))
    (out / "qualification.json").write_text(json.dumps(record, indent=2) + "\n")
    if p.returncode:
        raise SystemExit(f"{name} failed ({p.returncode}):\n{p.stderr[-4000:]}\n{p.stdout[-1000:]}")

def snapshot(name, repo):
    if subprocess.check_output(["git", "-C", str(repo), "status", "--porcelain"]):
        raise SystemExit(f"refusing uncommitted checkout: {repo}")
    sha = subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip()
    archive = out / (name + ".tar")
    archive.write_bytes(subprocess.check_output(["git", "-C", str(repo), "archive", sha]))
    target = out / name; target.mkdir()
    subprocess.run(["tar", "-xf", str(archive), "-C", str(target)], check=True)
    record["sources"][name] = dict(commit=sha, archive_sha256=hashlib.sha256(archive.read_bytes()).hexdigest())
    return target

c = snapshot("consumer", consumer)
p = snapshot("producer", producer)
run("tools", ["rustc", "--version", "--verbose"], c)
run("producer-fetch", ["cargo", "fetch", "--locked"], p)
run("producer-format", ["cargo", "+1.99.0", "fmt", "--all", "--", "--check"], p)
run("producer-tests", ["cargo", "test", "--lib", "--no-default-features", "--locked", "--offline"], p)
run("producer-clippy", ["cargo", "+1.99.0", "clippy", "--all-targets", "--no-default-features", "--locked", "--offline", "--", "-D", "warnings"], p)
patch = "patch.crates-io.libc-abi-tools.path=" + json.dumps(str(p))
config = c / ".cargo/config.toml"
if config.exists():
    raise SystemExit("source qualifier refuses to replace a repository Cargo config")
config.parent.mkdir(exist_ok=True)
config.write_text("[patch.crates-io]\nlibc-abi-tools = { path = " + json.dumps(str(p)) + " }\n")
cargo = ["cargo"]
run("consumer-resolve", cargo + ["metadata", "--format-version", "1", "--offline"], c)
run("consumer-format", ["cargo", "+1.99.0", "fmt", "--all", "--", "--check"], c)
run("consumer-tests", cargo + ["test", "--all-targets", "--locked", "--offline"], c)
run("consumer-doctests", cargo + ["test", "--doc", "--locked", "--offline"], c)
run("consumer-clippy", ["cargo", "+1.99.0", "clippy", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"], c)
env["RUSTDOCFLAGS"] = "-D warnings"
run("consumer-docs", cargo + ["doc", "--no-deps", "--locked", "--offline"], c)
run("consumer-build", cargo + ["build", "--bin", "cargo-libc", "--locked", "--offline"], c)
run("native-fixtures", ["python3", str(c / "scripts/check-target-libgcc.py"), str(out / "target/debug/cargo-libc"), str(out / "fixtures")], c)
for name, repo in [("consumer", c), ("producer", p)]:
    run(name + "-release-docs", ["python3", "scripts/check-release-docs.py", "--repo", "rustcommons/" + ("cargo-libc" if name == "consumer" else "linux-abi-tools")], repo)
record["status"] = "passed"
record["native_case_count"] = len(json.loads((out / "fixtures/results.json").read_text()))
record["sha256"] = {str(f.relative_to(out)): hashlib.sha256(f.read_bytes()).hexdigest() for f in out.rglob("*.log")}
(out / "qualification.json").write_text(json.dumps(record, indent=2) + "\n")
print("PASS: source qualification completed; registry adoption/package/install still separate", flush=True)
