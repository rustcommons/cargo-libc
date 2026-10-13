#!/usr/bin/env bash
# Build examples/python-ext (a PyO3 extension module) into a wheel with maturin and
# `cargo libc env` for each requested glibc release, and check it: maturin's
# manylinux audit, cargo libc inspect, an import on this host when its glibc is new
# enough, and with PYTHON_EXT_CONTAINERS=1 an import inside the manylinux2014
# (CentOS 7, glibc 2.17) and manylinux_2_28 (AlmaLinux 8, glibc 2.28) images.
# Usage: smoke-python-ext.sh CLANG [RELEASE...]   (default: every release)
set -euo pipefail
repo_dir=$(cd "$(dirname "$0")/.." && pwd)
test_cc=${1:?Usage: smoke-python-ext.sh CLANG [RELEASE...]}
shift
releases=("$@")
if [ ${#releases[@]} -eq 0 ]; then
  mapfile -t releases < <(seq -f '2.%g' 17 44)
fi
maturin_version=1.15.0
test_dir=$(mktemp -d /tmp/cargo-libc-python-ext.XXXXXXXX)
trap 'chmod -R u+w "$test_dir"; rm -rf -- "$test_dir"' EXIT
export CARGO_LIBC_PROVIDER_DIR=${CARGO_LIBC_PROVIDER_DIR:-$test_dir/providers}
example="$repo_dir/examples/python-ext"
triple=x86_64-unknown-linux-gnu
cargo build --manifest-path "$repo_dir/Cargo.toml" --locked --offline --target-dir "$test_dir/tools"
cargo fetch --manifest-path "$example/Cargo.toml" --locked
export PATH="$test_dir/tools/debug:$PATH"
python3 -m venv "$test_dir/venv"
"$test_dir/venv/bin/pip" install -q "maturin==$maturin_version"
maturin="$test_dir/venv/bin/maturin"
host_glibc=$(getconf GNU_LIBC_VERSION)
host_glibc=${host_glibc#glibc }

for release in "${releases[@]}"; do
  out="$test_dir/$release"
  wheels="$out/wheels"
  (
    eval "$(cargo libc env --target "$triple.$release" --cc "$test_cc")"
    # --target keeps build scripts (pyo3's) native; --compatibility makes maturin
    # audit the extension against that manylinux policy and tag the wheel with it.
    build() {
      "$maturin" build --release --target "$triple" --compatibility "manylinux_2_${release#2.}" \
        --manifest-path "$example/Cargo.toml" --target-dir "$out/target" --out "$wheels" \
        --locked --offline "$@" > "$out.log" 2>&1
    }
    # maturin's policy list ends before the newest glibc releases; for those it
    # cannot audit, and cargo libc inspect below is the only check.
    build || { grep -q 'policy is not defined by auditwheel yet' "$out.log" &&
      echo "maturin has no manylinux_2_${release#2.} policy: audit skipped" && build --auditwheel skip; } ||
      { cat "$out.log"; exit 1; }
  )
  wheel=$(ls "$wheels"/*.whl)
  case "$wheel" in
    *"-cp310-abi3-manylinux_2_${release#2.}_x86_64"*) ;;
    *) echo "unexpected wheel name: $wheel" >&2; exit 1 ;;
  esac
  mkdir "$out/unpacked"
  python3 -m zipfile -e "$wheel" "$out/unpacked"
  cargo libc inspect "$out/unpacked/glibc_python_ext"/*.so --max-glibc "$release"
  if [ "$(printf '%s\n%s\n' "$release" "$host_glibc" | sort -V | tail -1)" = "$host_glibc" ]; then
    "$test_dir/venv/bin/pip" install -q --force-reinstall --no-deps "$wheel"
    "$test_dir/venv/bin/python" "$example/check.py" "$host_glibc"
  fi
  cp "$wheel" "$test_dir/"
done

if [ "${PYTHON_EXT_CONTAINERS:-0}" = 1 ]; then
  engine=${CONTAINER_ENGINE:-podman}
  # manylinux images keep their Python builds under /opt/python.
  for pair in "2.17 quay.io/pypa/manylinux2014_x86_64" "2.28 quay.io/pypa/manylinux_2_28_x86_64"; do
    set -- $pair
    wheel=$(ls "$test_dir"/*-manylinux_2_${1#2.}_x86_64*.whl 2>/dev/null) || continue
    "$engine" run --rm -v "$wheel:/w/$(basename "$wheel"):ro" -v "$example/check.py:/w/check.py:ro" "$2" \
      bash -c "set -e; /opt/python/cp310-cp310/bin/python -m pip install -q --root-user-action=ignore --no-deps /w/*.whl
        ldd --version | head -1; /opt/python/cp310-cp310/bin/python /w/check.py $1"
  done
fi
echo "Python extension: built and checked for ${releases[*]}."
