#!/usr/bin/env bash
# Disposable source qualification, usable on a native x86-64 Linux host or WSL.
# Usage: qualify-clock-source.sh CHECKOUT PRODUCER_CHECKOUT OLD_INSTALL_ROOT EVIDENCE EXPECTED_PRODUCER_SHA
set -euo pipefail
checkout=$(realpath "${1:?checkout}")
producer=$(realpath "${2:?producer checkout}")
old_root=$(realpath -m "${3:?old installation root}")
evidence=$(realpath -m "${4:?fresh evidence directory}")
expected_producer=${5:?exact independently reviewed producer SHA}
[[ "$expected_producer" =~ ^[0-9a-f]{40}$ ]] || { echo 'Expected producer ref must be a full commit SHA' >&2; exit 1; }
if [ "$expected_producer" = 1b3b51c8afad88e05b0dc238c15f5ee879f94184 ]; then
  echo 'Producer 1b3b51c8 is review-blocked; use the corrected reviewed ref, not release gates on that source.' >&2
  exit 1
fi
export CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2
test "$(uname -m)" = x86_64
test "$(git -C "$producer" rev-parse HEAD)" = "$expected_producer"
git -C "$producer" diff --quiet HEAD --
for tool in cargo rustc python3 clang-18 gcc ld.bfd readelf; do
  command -v "$tool" >/dev/null || { echo "Missing existing native qualification tool: $tool" >&2; exit 1; }
done
test ! -e "$evidence/consumer"
mkdir -p "$evidence/consumer"
git -C "$checkout" archive HEAD | tar -x -C "$evidence/consumer"
{
  git -C "$checkout" rev-parse HEAD
  git -C "$producer" rev-parse HEAD
  rustc -vV
  cargo -V
  uname -a
  clang-18 --version
  gcc --version
  ld.bfd --version
} > "$evidence/refs-tools.txt"
python3 - "$evidence/consumer/Cargo.toml" "$producer" <<'PY'
import json,sys,tomllib
from pathlib import Path
p=Path(sys.argv[1])
original=p.read_text()
before=tomllib.loads(original)
assert not before.get('patch') and not before.get('replace'), 'production manifest must not contain source overrides'
patch=f'''
[patch.crates-io]
libc-abi-tools = {{ path = {json.dumps(sys.argv[2])} }}
'''
text=original+patch
after=tomllib.loads(text)
assert after['patch']['crates-io']=={'libc-abi-tools':{'path':sys.argv[2]}}
assert after['dependencies']==before['dependencies']
p.write_text(text)
PY
cd "$evidence/consumer"
{
  cargo update -p libc-abi-tools
  cargo fetch --locked
  cargo metadata --format-version 1 --locked --offline > "$evidence/metadata.json"
  cp Cargo.toml Cargo.lock "$evidence/"
  python3 - "$checkout/Cargo.lock" "$producer" "$evidence/metadata.json" <<'PY'
import json,sys,tomllib
from pathlib import Path
old=tomllib.loads(Path(sys.argv[1]).read_text())
new=tomllib.loads(Path('Cargo.lock').read_text())
keep=lambda d:{p['name']:p for p in d['package'] if p['name'] not in ['cargo-libc','libc-abi-tools']}
assert keep(old)==keep(new), 'unrelated registry lock entries changed'
data=json.loads(Path(sys.argv[3]).read_text())
assert {p['name']:p['version'] for p in data['packages']}=={'cargo-libc':'0.3.0','libc-abi-tools':'0.4.0','kernel-abi-tools':'0.3.3','linux-abi-scan':'0.3.3','linux-targets':'0.3.3','glibc-abilists':'0.1.2','glibc-provider-sources':'0.1.1'}
for p in data['packages']:
    if p['name']=='libc-abi-tools':
        assert p['source'] is None and Path(p['manifest_path']).resolve()==Path(sys.argv[2]).resolve()/'Cargo.toml'
    elif p['name']!='cargo-libc':
        assert p['source']=='registry+https://github.com/rust-lang/crates.io-index'
print('Exact root-only source adoption; other tools and glibc pins/checksums unchanged')
PY
  cargo fmt --all -- --check
  cargo test --lib clock:: --locked --offline
  cargo test --lib elf:: --locked --offline
  cargo clippy --all-targets --locked --offline -- -D warnings
  cargo build --locked --offline
  if [ ! -x "$old_root/bin/cargo-libc" ]; then
    cargo install cargo-libc --version 0.2.4 --locked --root "$old_root"
  fi
  test "$("$old_root/bin/cargo-libc" --version)" = 'cargo-libc 0.2.4'
  python3 scripts/check-clock-profile.py target/debug "$old_root/bin" "$evidence/clocks"
  for fixture in hello too-new c-deps; do cargo fetch --manifest-path "examples/$fixture/Cargo.toml" --locked; done
  bash scripts/smoke-commands.sh /usr/bin/clang-18 2.22
  if command -v cmake >/dev/null; then
    bash scripts/smoke-c-deps.sh /usr/bin/clang-18 2.22
    CARGO_LIBC_SMOKE_CC=/usr/bin/clang-18 bash scripts/smoke-linux.sh
  else
    echo 'Coverage gap: CMake-dependent C/host-target smoke scripts not run; cmake is absent. Core C/Cargo and actual host-sysroot ELF cases ran.' | tee "$evidence/coverage-gaps.txt" >&2
  fi
} 2>&1 | tee "$evidence/source-qualification.log"
echo 'Source/native qualification passed; registry/package/install and container preparation remain separate gates.'
