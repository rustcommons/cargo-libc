#!/usr/bin/env bash
# Tier 2 in a real old userland: build with the sles-12-sp5 preset (glibc 2.22,
# kernel 4.12) and run the programs and tests with the container runner, in
# SUSE's SLES 12 SP5 image under the Linux 4.12 profile. Checks that they ran on
# the image's glibc 2.22, not the host's.
# Usage: smoke-container-runner.sh CLANG
# Needs podman (or CONTAINER_ENGINE=docker) and crun 1.28 or later:
#   KERNEL_ABI_ENGINE_ARGS="--runtime /path/to/crun" (see scripts/install-crun.sh).
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
test_cc=${1:?Usage: smoke-container-runner.sh CLANG}
test_dir=$(mktemp -d /tmp/cargo-libc-container.XXXXXXXX)
trap 'chmod -R u+w "$test_dir"; rm -rf -- "$test_dir"' EXIT
export CARGO_LIBC_PROVIDER_DIR=${CARGO_LIBC_PROVIDER_DIR:-$test_dir/providers}
cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --offline --target-dir "$test_dir/tools"
cargo fetch --manifest-path "$repo_dir/examples/c-deps/Cargo.toml" --locked
export PATH="$test_dir/tools/debug:$PATH"
selection=(--distro sles-12-sp5 --runner container --cc "$test_cc")
hello="$repo_dir/examples/hello/Cargo.toml"

check() {
  local name=$1 log="$test_dir/$1.log"
  shift
  "$@" > "$log" 2>&1 || { cat "$log"; exit 1; }
}

# Both runners must work here; the container one is the one under test.
check doctor cargo libc doctor --distro sles-12-sp5 --runner container
grep -F 'container: ok: `true` ran in registry.suse.com/suse/sles12sp5:latest' "$test_dir/doctor.log"
grep -F 'Selected runner: usable' "$test_dir/doctor.log"

check run cargo libc run "${selection[@]}" -- --manifest-path "$hello" \
  --target-dir "$test_dir/hello" --bin glibc-hello --locked --offline
grep -F 'target executable ran; thread result = 42' "$test_dir/run.log"
grep -Fx 'runtime glibc 2.22' "$test_dir/run.log"

# Tests run in the container too; arguments after the second -- reach the harness.
check test cargo libc test "${selection[@]}" -- --manifest-path "$hello" \
  --target-dir "$test_dir/hello" --locked --offline -- --nocapture
grep -F 'test ran on runtime glibc 2.22' "$test_dir/test.log"
grep -F 'test result: ok. 1 passed' "$test_dir/test.log"

# C dependencies (cc and CMake), with the image's glibc at run time.
check c-deps cargo libc run "${selection[@]}" -- \
  --manifest-path "$repo_dir/examples/c-deps/Cargo.toml" --target-dir "$test_dir/c-deps" --locked --offline
grep -F 'cc: glibc 2.22 headers' "$test_dir/c-deps.log"
grep -F 'cmake: running on glibc 2.22' "$test_dir/c-deps.log"
echo "Container runner: built for sles-12-sp5 and ran in its image (glibc 2.22, Linux 4.12 profile)."
