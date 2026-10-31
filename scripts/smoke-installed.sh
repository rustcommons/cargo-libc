#!/usr/bin/env bash
# Exercise an isolated installation, never rebuilding cargo-libc itself.
# Usage: smoke-installed.sh INSTALL_ROOT CLANG EXPECTED_VERSION
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
install_root=${1:?install root}
test_cc=${2:?Clang}
expected=${3:?expected version}
test_dir=$(mktemp -d /tmp/cargo-libc-installed.XXXXXXXX)
trap 'chmod -R u+w "$test_dir"; rm -rf -- "$test_dir"' EXIT
export PATH="$install_root/bin:$PATH"
export CARGO_LIBC_PROVIDER_DIR="$test_dir/providers"
for name in cargo-libc libc-linker libc-runner libc-cc libc-cxx; do
  test -x "$install_root/bin/$name"
done
test "$(cargo libc --version)" = "cargo-libc $expected"
cargo libc --help > "$test_dir/help"
grep -F 'libraries (cdylib)' "$test_dir/help"
cargo libc inspect /bin/true --max-glibc "$(getconf GNU_LIBC_VERSION | cut -d ' ' -f2)"
python3 "$repo_dir/scripts/check-provider-stdout.py" "$install_root/bin/cargo-libc"
# Use the installed linker, runner and C wrappers through a real C/CMake fixture.
manifest="$repo_dir/examples/c-deps/Cargo.toml"
cargo fetch --manifest-path "$manifest" --locked
cargo libc run --target x86_64-unknown-linux-gnu.2.17 --cc "$test_cc" \
  --kernel 3.10 --runner local -- --manifest-path "$manifest" \
  --target-dir "$test_dir/app" --locked --offline > "$test_dir/run.log" 2>&1 ||
  { cat "$test_dir/run.log"; exit 1; }
cat "$test_dir/run.log"
grep -F 'cc: glibc 2.17 headers' "$test_dir/run.log"
grep -F 'cmake: glibc 2.17 headers' "$test_dir/run.log"
cargo libc inspect "$test_dir/app/x86_64-unknown-linux-gnu/debug/glibc-c-deps" --max-glibc 2.17
cargo libc provider verify --glibc 2.17
# Exercise the installed libc-cxx refusal path as well.
if cargo libc build --target x86_64-unknown-linux-gnu.2.17 --cc "$test_cc" \
  --artifact "$test_dir/cxx/x86_64-unknown-linux-gnu/debug/glibc-c-deps" -- \
  --manifest-path "$manifest" --target-dir "$test_dir/cxx" --locked --offline --features cxx \
  > "$test_dir/cxx.log" 2>&1; then
  echo 'installed tool accepted target C++' >&2; exit 1
fi
grep -F 'C++ for the target is not supported yet' "$test_dir/cxx.log"
echo "Installed cargo-libc $expected and all five executables passed."
