#!/usr/bin/env bash
# Build, run and inspect with automatic providers (no --provider or --sysroot)
# for each requested glibc release, and check that a too-new symbol is refused.
# Usage: smoke-providers.sh CLANG [RELEASE...]   (default: every release)
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
test_cc=${1:?Usage: smoke-providers.sh CLANG [RELEASE...]}
shift
releases=("$@")
if [ ${#releases[@]} -eq 0 ]; then
  mapfile -t releases < <(seq -f '2.%g' 17 44)
fi
test_dir=$(mktemp -d /tmp/cargo-libc-providers.XXXXXXXX)
trap 'chmod -R u+w "$test_dir"; rm -rf -- "$test_dir"' EXIT
export CARGO_LIBC_PROVIDER_DIR="$test_dir/providers"
cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --offline --target-dir "$test_dir/tools"
export PATH="$test_dir/tools/debug:$PATH"
host_glibc=$(getconf GNU_LIBC_VERSION)
host_glibc=${host_glibc#glibc }

for release in "${releases[@]}"; do
  target="x86_64-unknown-linux-gnu.$release"
  out="$test_dir/target-$release"
  artifact="$out/x86_64-unknown-linux-gnu/debug/glibc-hello"
  log="$test_dir/build-$release.log"
  cargo libc build --target "$target" --cc "$test_cc" --kernel 3.10 --artifact "$artifact" -- \
    --manifest-path "$repo_dir/examples/hello/Cargo.toml" --target-dir "$out" --locked --offline \
    > "$log" 2>&1 || { cat "$log"; exit 1; }
  grep -F "glibc $release provider (source mode, prepared)" "$log"
  grep -F 'verified every object against the container build' "$log"
  grep -F 'Tier 1 scan (linux-abi-scan): pass' "$log"
  cargo libc inspect "$artifact" --max-glibc "$release"
  # A second build reuses the cached provider.
  cargo libc check --target "$target" --cc "$test_cc" -- \
    --manifest-path "$repo_dir/examples/hello/Cargo.toml" --target-dir "$out" --locked --offline \
    2>&1 | grep -F "glibc $release provider (source mode, cached)"
  # Tier 2 on this host: its glibc must be at least the target's.
  if [ "$(printf '%s\n%s\n' "$release" "$host_glibc" | sort -V | tail -1)" = "$host_glibc" ]; then
    cargo libc run --target "$target" --cc "$test_cc" --kernel 3.10 --runner local -- \
      --manifest-path "$repo_dir/examples/hello/Cargo.toml" --target-dir "$out" --bin glibc-hello \
      --locked --offline > "$test_dir/run-$release.log" 2>&1 || { cat "$test_dir/run-$release.log"; exit 1; }
    grep -F 'target executable ran; thread result = 42' "$test_dir/run-$release.log"
  fi
  # memfd_create is GLIBC_2.27: older targets must refuse it at link time.
  if [ "$(printf '%s\n2.27\n' "$release" | sort -V | head -1)" = "$release" ] && [ "$release" != 2.27 ]; then
    negative="$test_dir/too-new-$release"
    if cargo libc build --target "$target" --cc "$test_cc" \
      --artifact "$negative/x86_64-unknown-linux-gnu/debug/glibc-too-new" -- \
      --manifest-path "$repo_dir/examples/too-new/Cargo.toml" --target-dir "$negative" --locked --offline \
      > "$test_dir/too-new-$release.log" 2>&1; then
      echo "glibc $release accepted memfd_create" >&2; exit 1
    fi
    grep -E 'undefined (reference|symbol).*memfd_create' "$test_dir/too-new-$release.log"
  fi
  cargo libc provider verify --glibc "$release"
done
cargo libc provider list
test "$(cargo libc provider list | wc -l)" -eq $(( ${#releases[@]} + 1 ))
echo "Automatic providers: ${releases[*]} built, checked, inspected and verified."
