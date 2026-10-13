#!/usr/bin/env bash
# Exercise the host/target boundary with the host SDK, not a claimed old SDK.
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
test_dir=$(mktemp -d /tmp/cargo-libc-smoke.XXXXXXXX)
trap 'rm -rf -- "$test_dir"' EXIT
test_cc=${CARGO_LIBC_SMOKE_CC:-/usr/bin/clang}
test_glibc=$(getconf GNU_LIBC_VERSION)
test_glibc=${test_glibc#glibc }
test_gcc=$(dirname "$(gcc -print-file-name=crtbeginS.o)")

cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --offline --target-dir "$test_dir/tools"
export PATH="$test_dir/tools/debug:$PATH"
cargo libc --version
cargo libc doctor --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc"
cd "$repo_dir/examples/hello"
test_artifact="$test_dir/target output/x86_64-unknown-linux-gnu/release/glibc-hello"
cargo libc build --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc" \
  --artifact "$test_artifact" -- --release --locked --offline -vv \
  --target-dir "$test_dir/target output" >"$test_dir/build.log" 2>&1
cat "$test_dir/build.log"
"$test_artifact"

# The host build script and procedural macro must not use the target helper.
for crate in build_script_build hello_macro; do
  host_line=$(grep 'Running.*rustc.*--crate-name '"$crate"' ' "$test_dir/build.log")
  if [[ "$host_line" == *libc-linker* || "$host_line" == *'--target x86_64-unknown-linux-gnu'* ]]; then
    echo "Host crate $crate received target linker configuration" >&2
    exit 1
  fi
done
grep -q 'Running.*rustc.*--crate-name glibc_hello .*libc-linker' "$test_dir/build.log"

# Cached outputs remain valid artifacts of the current Cargo invocation.
cargo libc build --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc" \
  --artifact "$test_artifact" -- --release --locked --offline \
  --target-dir "$test_dir/target output"
cargo libc inspect "$test_artifact" --max-glibc "$test_glibc" --json
# Tier 1 scan against the host floors: it must run and report its findings.
cargo libc inspect "$test_artifact" --max-glibc "$test_glibc" --kernel "$(uname -r)" --json   >"$test_dir/scan.json"
grep -q '"scan":{"tier":1,' "$test_dir/scan.json"

# An unrelated existing ELF must never substitute for Cargo's output.
if cargo libc build --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc" \
  --artifact /bin/true -- --release --locked --offline \
  --target-dir "$test_dir/target output" >"$test_dir/rejection.log" 2>&1; then
  echo 'Unrelated artifact was accepted' >&2
  exit 1
fi
grep -q 'not reported' "$test_dir/rejection.log"

# Tier 2: the target executable runs through libc-runner under a 3.10 filter;
# the host build script and procedural macro are not routed through it.
cargo libc run --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc" \
  --kernel 3.10 --runner local -- --release --locked --offline --bin glibc-hello \
  --target-dir "$test_dir/target output" >"$test_dir/run.log" 2>&1
cat "$test_dir/run.log"
grep -q 'Kernel simulation: Linux 3.10 with the local seccomp filter' "$test_dir/run.log"
grep -q 'target executable ran; thread result = 42' "$test_dir/run.log"
# check: same target flags, host units stay native, nothing is linked.
cargo libc check --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc" \
  -- --locked --offline -vv --target-dir "$test_dir/check output" >"$test_dir/check.log" 2>&1
cat "$test_dir/check.log"
for crate in build_script_build hello_macro; do
  host_line=$(grep 'Running.*rustc.*--crate-name '"$crate"' ' "$test_dir/check.log")
  if [[ "$host_line" == *libc-linker* || "$host_line" == *'--target x86_64-unknown-linux-gnu'* ]]; then
    echo "check gave host crate $crate the target configuration" >&2
    exit 1
  fi
done
grep -q 'Running.*rustc.*--crate-name glibc_hello .*--target x86_64-unknown-linux-gnu.*libc-linker' "$test_dir/check.log"
grep -q 'Nothing was linked' "$test_dir/check.log"
if [ -e "$test_dir/check output/x86_64-unknown-linux-gnu/debug/glibc-hello" ]; then
  echo 'check produced a linked executable' >&2
  exit 1
fi
# A type error keeps Cargo's diagnostic and exit status.
mkdir -p "$test_dir/broken/src"
printf '[package]\nname = "broken"\nversion = "0.1.0"\nedition = "2021"\n[workspace]\n' \
  >"$test_dir/broken/Cargo.toml"
printf 'fn main() { let _x: u32 = "text"; }\n' >"$test_dir/broken/src/main.rs"
set +e
(cd "$test_dir/broken" && cargo libc check --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" \
  --cc "$test_cc" -- --offline --color never) >"$test_dir/broken.log" 2>&1
broken_status=$?
set -e
cat "$test_dir/broken.log"
[ "$broken_status" -eq 101 ]
grep -q 'error\[E0308\]: mismatched types' "$test_dir/broken.log"
# doctor checks the runners without toolchain inputs; the local one must work here.
cargo libc doctor --kernel 3.10 --runner local >"$test_dir/doctor.log" 2>&1
cat "$test_dir/doctor.log"
grep -q '  local: ok: seccomp filter for Linux 3.10 installed without root' "$test_dir/doctor.log"
grep -q 'Selected runner: usable' "$test_dir/doctor.log"
# C dependencies (cc crate and CMake) compile for the target through libc-cc with
# the --sysroot selection; scripts/smoke-c-deps.sh covers the providers.
cargo fetch --manifest-path "$repo_dir/examples/c-deps/Cargo.toml" --locked
cargo libc run --sysroot / --gcc-runtime "$test_gcc" --glibc "$test_glibc" --cc "$test_cc"   --kernel 3.10 --runner local -- --manifest-path "$repo_dir/examples/c-deps/Cargo.toml"   --locked --offline --target-dir "$test_dir/c-deps" >"$test_dir/c-deps.log" 2>&1 ||
  { cat "$test_dir/c-deps.log"; exit 1; }
cat "$test_dir/c-deps.log"
grep -q "cc: glibc $test_glibc headers" "$test_dir/c-deps.log"
grep -q "cmake: glibc $test_glibc headers" "$test_dir/c-deps.log"
grep -q 'cmake: memfd_create found = 1' "$test_dir/c-deps.log"
echo 'Linux smoke test passed (host SDK only).'
