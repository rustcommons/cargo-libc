#!/usr/bin/env bash
# Build examples/c-deps (C through the cc crate and CMake, plus a proc macro with
# host C code) with automatic providers for each requested glibc release.
# Usage: smoke-c-deps.sh CLANG [RELEASE...]   (default: every release)
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
test_cc=${1:?Usage: smoke-c-deps.sh CLANG [RELEASE...]}
shift
releases=("$@")
if [ ${#releases[@]} -eq 0 ]; then
  mapfile -t releases < <(seq -f '2.%g' 17 44)
fi
test_dir=$(mktemp -d /tmp/cargo-libc-c-deps.XXXXXXXX)
trap 'chmod -R u+w "$test_dir"; rm -rf -- "$test_dir"' EXIT
export CARGO_LIBC_PROVIDER_DIR=${CARGO_LIBC_PROVIDER_DIR:-$test_dir/providers}
manifest="$repo_dir/examples/c-deps/Cargo.toml"
cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --offline --target-dir "$test_dir/tools"
cargo fetch --manifest-path "$manifest" --locked
export PATH="$test_dir/tools/debug:$PATH"
host_glibc=$(getconf GNU_LIBC_VERSION)
host_glibc=${host_glibc#glibc }
at_least() { [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | head -1)" = "$2" ]; }

for release in "${releases[@]}"; do
  target="x86_64-unknown-linux-gnu.$release"
  out="$test_dir/target-$release"
  artifact="$out/x86_64-unknown-linux-gnu/debug/glibc-c-deps"
  log="$test_dir/build-$release.log"
  cargo libc build --target "$target" --cc "$test_cc" --artifact "$artifact" -- \
    --manifest-path "$manifest" --target-dir "$out" --locked --offline \
    > "$log" 2>&1 || { cat "$log"; exit 1; }
  # The C objects linked against the provider: no requirement above the release.
  cargo libc inspect "$artifact" --max-glibc "$release"
  # Tier 2 run (when this host's glibc has every version the executable needs):
  # the C code saw the target's headers, CMake's configure check linked against
  # the target's stubs, and the proc macro's C code saw the host's headers.
  if cargo libc inspect "$artifact" --max-glibc "$host_glibc" > /dev/null; then
    run="$test_dir/run-$release.log"
    cargo libc run --target "$target" --cc "$test_cc" --kernel 3.10 --runner local -- \
      --manifest-path "$manifest" --target-dir "$out" --locked --offline \
      > "$run" 2>&1 || { cat "$run"; exit 1; }
    grep -F "cc: glibc $release headers" "$run"
    grep -F 'cc: strtol parsed 42' "$run"
    grep -F "cmake: glibc $release headers" "$run"
    # memfd_create is GLIBC_2.27.
    if at_least "$release" 2.27; then expected=1; else expected=0; fi
    grep -F "cmake: memfd_create found = $expected" "$run"
    grep -F "host proc macro: glibc $host_glibc headers" "$run"
  fi
done

# C++ for the target is refused, not compiled with the host's C++ runtime.
cxx="$test_dir/cxx.log"
if cargo libc build --target "x86_64-unknown-linux-gnu.${releases[0]}" --cc "$test_cc" \
  --artifact "$test_dir/cxx/x86_64-unknown-linux-gnu/debug/glibc-c-deps" -- \
  --manifest-path "$manifest" --target-dir "$test_dir/cxx" --locked --offline --features cxx \
  > "$cxx" 2>&1; then
  echo 'C++ for the target was accepted' >&2; exit 1
fi
grep -F 'C++ for the target is not supported yet' "$cxx"

# Without cargo-libc the same C code follows the host's headers: with glibc
# 2.38+ headers, strtol becomes __isoc23_strtol@GLIBC_2.38.
if at_least "$host_glibc" 2.38; then
  cargo build --manifest-path "$manifest" --target-dir "$test_dir/plain" --locked --offline
  readelf --dyn-syms --wide "$test_dir/plain/debug/glibc-c-deps" | grep -F '__isoc23_strtol@GLIBC_2.38'
fi
echo "C dependencies: ${releases[*]} built with cc and CMake for the target, host C kept native."
