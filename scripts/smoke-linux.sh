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
echo 'Linux smoke test passed (host SDK only).'
