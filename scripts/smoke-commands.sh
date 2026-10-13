#!/usr/bin/env bash
# clippy, doc, rustc, install and plain Cargo (cargo libc env) with automatic
# providers, for each requested glibc release.
# Usage: smoke-commands.sh CLANG RELEASE...
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
test_cc=${1:?Usage: smoke-commands.sh CLANG RELEASE...}
shift
[ $# -gt 0 ] || { echo 'name at least one release' >&2; exit 1; }
test_dir=$(mktemp -d /tmp/cargo-libc-commands.XXXXXXXX)
trap 'chmod -R u+w "$test_dir"; rm -rf -- "$test_dir"' EXIT
export CARGO_LIBC_PROVIDER_DIR=${CARGO_LIBC_PROVIDER_DIR:-$test_dir/providers}
hello="$repo_dir/examples/hello/Cargo.toml"
c_deps="$repo_dir/examples/c-deps/Cargo.toml"
cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --offline --target-dir "$test_dir/tools"
cargo fetch --manifest-path "$c_deps" --locked
export PATH="$test_dir/tools/debug:$PATH"
host_glibc=$(getconf GNU_LIBC_VERSION)
host_glibc=${host_glibc#glibc }
triple=x86_64-unknown-linux-gnu

for release in "$@"; do
  target="$triple.$release"
  out="$test_dir/$release"
  common=(--target "$target" --cc "$test_cc")

  # clippy: Cargo arguments, then clippy's own after the second --.
  cargo libc clippy "${common[@]}" -- --manifest-path "$hello" --target-dir "$out/clippy" \
    --locked --offline -- -D clippy::correctness
  # A linker override after rustc's -- is refused.
  if cargo libc rustc "${common[@]}" -- --manifest-path "$hello" --bin glibc-hello -- -C linker=cc \
    > "$out-refused.log" 2>&1; then
    echo 'rustc accepted a linker override' >&2; exit 1
  fi
  grep -F 'conflicts with the managed target configuration' "$out-refused.log"

  # doc: rustdoc sees the target selection.
  cargo libc doc "${common[@]}" -- --manifest-path "$hello" --target-dir "$out/doc" \
    --locked --offline --no-deps
  test -f "$out/doc/$triple/doc/glibc_hello/index.html"

  # rustc builds and links; its output is checked here, not by the command.
  cargo libc rustc "${common[@]}" -- --manifest-path "$hello" --target-dir "$out/rustc" \
    --locked --offline --bin glibc-hello -- -C opt-level=1
  cargo libc inspect "$out/rustc/$triple/debug/glibc-hello" --max-glibc "$release"

  # install: builds, installs, then checks (and scans) what it installed.
  cargo libc install "${common[@]}" --kernel 3.10 -- --path "$repo_dir/examples/hello" \
    --root "$out/root" --locked --offline --debug > "$out-install.log" 2>&1 ||
    { cat "$out-install.log"; exit 1; }
  grep -F 'Direct glibc requirement check: pass' "$out-install.log"
  grep -F 'Tier 1 scan (linux-abi-scan): pass' "$out-install.log"
  cargo libc inspect "$out/root/bin/glibc-hello" --max-glibc "$release"

  # Plain Cargo with the printed configuration, in both formats.
  (
    eval "$(cargo libc env "${common[@]}")"
    cargo build --target "$triple" --manifest-path "$hello" --target-dir "$out/plain-sh" --locked --offline
    cargo libc inspect "$out/plain-sh/$triple/debug/glibc-hello" --max-glibc "$release"
    # C dependencies through the same variables.
    cargo build --target "$triple" --manifest-path "$c_deps" --target-dir "$out/plain-c" --locked --offline
    cargo libc inspect "$out/plain-c/$triple/debug/glibc-c-deps" --max-glibc "$release"
    # The runner (libc-runner) checks glibc before running, when this host can run it.
    if cargo libc inspect "$out/plain-sh/$triple/debug/glibc-hello" --max-glibc "$host_glibc" > /dev/null; then
      cargo run --target "$triple" --manifest-path "$hello" --target-dir "$out/plain-sh" \
        --locked --offline --bin glibc-hello | grep -F 'target executable ran; thread result = 42'
      cargo run --target "$triple" --manifest-path "$c_deps" --target-dir "$out/plain-c" \
        --locked --offline > "$out-plain-c.log"
      grep -F "cc: glibc $release headers" "$out-plain-c.log"
      grep -F "host proc macro: glibc $host_glibc headers" "$out-plain-c.log"
    fi
  )
  cargo libc env "${common[@]}" --kernel 3.10 --format cargo > "$out-config.toml"
  grep -F "[target.$triple]" "$out-config.toml"
  grep -F 'CARGO_LIBC_RUN_KERNEL = "3.10"' "$out-config.toml"
  cargo build --target "$triple" --config "$out-config.toml" --manifest-path "$hello" \
    --target-dir "$out/plain-cargo" --locked --offline
  cargo libc inspect "$out/plain-cargo/$triple/debug/glibc-hello" --max-glibc "$release"
done
echo "Commands: clippy, doc, rustc, install and plain Cargo passed for $*."
