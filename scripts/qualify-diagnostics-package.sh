#!/usr/bin/env bash
# Native x86/Rust 1.90 qualification; no installs of system tools, no publication.
# QUALIFY_EXPECTED_SHA=40_HEX rustup run 1.90.0 bash SCRIPT CHECKOUT CLANG EVIDENCE
set -euo pipefail
repo=$(cd "${1:?checkout}" && pwd)
test_cc=${2:?Clang path}
evidence=$(realpath -m "${3:?evidence directory}")
expected=${QUALIFY_EXPECTED_SHA:?pin the reviewed checkout SHA}
[[ "$expected" =~ ^[0-9a-f]{40}$ ]]
test "$(git -C "$repo" rev-parse HEAD)" = "$expected"
test -z "$(git -C "$repo" status --porcelain)"
test "$(uname -s)" = Linux
test "$(uname -m)" = x86_64
[[ "$(rustc --version)" == 'rustc 1.90.0 '* ]]
[[ "$(cargo --version)" == 'cargo 1.90.0 '* ]]
df -h "$repo" /tmp
mkdir -p "$evidence"
work=$(mktemp -d /tmp/cargo-libc-package-040.XXXXXXXX)
export CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2
unset CARGO_TARGET_DIR RUSTFLAGS CARGO_ENCODED_RUSTFLAGS RUSTDOCFLAGS CARGO_ENCODED_RUSTDOCFLAGS
export CARGO_HOME="$work/package-cargo-home"
mkdir "$CARGO_HOME"
python3 - "$repo" "$evidence" <<'PY'
import os, sys
from pathlib import Path
repo, evidence = map(Path, sys.argv[1:])
assert not evidence.is_relative_to(repo), 'keep evidence outside clean source checkout'
assert not any(k.startswith('CARGO_SOURCE_') for k in os.environ), 'source replacement environment present'
for root in [repo, *repo.parents]:
    for name in ['config', 'config.toml']:
        assert not (root / '.cargo' / name).exists(), 'unexpected ancestor Cargo config'
PY
cd "$repo"
{
  printf 'candidate=%s\nwork=%s\n' "$expected" "$work"
  uname -a
  rustc -vV
  cargo --version
  "$test_cc" --version
  gcc --version
  ld.bfd --version
  cmake --version
} > "$evidence/environment.txt"
{
  cargo fetch --locked
  cargo metadata --format-version 1 --locked --offline > "$evidence/metadata.json"
  cargo package --locked --target-dir "$work/package-target"
  archive="$work/package-target/package/cargo-libc-0.4.0.crate"
  cp "$archive" "$evidence/"
  sha256sum "$archive"
  python3 scripts/check-release-package.py "$archive" "$evidence"
  mkdir "$work/unpacked"
  tar -xzf "$archive" -C "$work/unpacked"
  unpacked="$work/unpacked/cargo-libc-0.4.0"
  export CARGO_HOME="$work/install-cargo-home"
  mkdir "$CARGO_HOME"
  cargo fetch --manifest-path "$unpacked/Cargo.toml" --locked
  cargo metadata --manifest-path "$unpacked/Cargo.toml" --format-version 1 --locked --offline > "$evidence/unpacked-metadata.json"
  python3 - "$evidence" <<'PY'
import json, sys
from pathlib import Path
root=Path(sys.argv[1])
for name in ['metadata.json', 'unpacked-metadata.json']:
    data=json.loads((root/name).read_text())
    assert len(data['packages']) == 7
    for p in data['packages']:
        if p['name']=='cargo-libc': assert p['version']=='0.4.0'
        else: assert p['source']=='registry+https://github.com/rust-lang/crates.io-index',p['name']
PY
  cargo install --path "$unpacked" --locked --offline --root "$work/install" --target-dir "$work/install-target"
  sha256sum "$work/install/bin/"* > "$evidence/installed-binary-sha256.txt"
  for name in cargo-libc libc-linker libc-runner libc-cc libc-cxx; do test -x "$work/install/bin/$name"; done
  "$work/install/bin/cargo-libc" --version | grep -Fx 'cargo-libc 0.4.0'
  python3 scripts/check-installed-capabilities.py "$work/install/bin" "$test_cc" "$evidence/capabilities"
  bash scripts/smoke-installed.sh "$work/install" "$test_cc" 0.4.0 2>&1 | tee "$evidence/installed-smoke.log"
  # The host C proc macro must see native host headers, not the 2.17 target's.
  host_minor=$(printf '#include <features.h>\n__GLIBC_MINOR__\n' | gcc -E -P -x c - | tail -1)
  grep -Fx "host proc macro: glibc 2.$host_minor headers" "$evidence/installed-smoke.log"
  # The existing C fixture includes a native proc macro using host C headers.
  python3 scripts/check-run-streams.py "$work/install/bin" "$test_cc"
  git diff --exit-code HEAD -- Cargo.toml Cargo.lock src README.md LICENSE NOTICE.md
  printf 'PASS native x86 Rust1.90 package/unpacked-install; candidate=%s\n' "$expected"
} 2>&1 | tee "$evidence/package-install.log"
printf 'Evidence retained at %s; build/install workspace retained at %s\n' "$evidence" "$work"
