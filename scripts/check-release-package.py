#!/usr/bin/env python3
"""Audit the 0.3.0 package against source and official 0.2.4, without publishing."""
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile
import tomllib
import urllib.request

package, evidence = map(Path, sys.argv[1:3])
with urllib.request.urlopen('https://static.crates.io/crates/cargo-libc/cargo-libc-0.2.4.crate', timeout=60) as r:
    baseline = r.read()
assert hashlib.sha256(baseline).hexdigest() == 'b456ab453f55e12c7a890f1b40b759a9d0d8c12c17acedc7af331ca8a1774868'


def payload(archive):
    return {m.name.split('/', 1)[1]: archive.extractfile(m).read() for m in archive if m.isfile()}


with tarfile.open(fileobj=io.BytesIO(baseline)) as tar:
    old = payload(tar)
with tarfile.open(package) as tar:
    new = payload(tar)
vcs = '.cargo_vcs_info.json'
assert json.loads(old[vcs])['git']['sha1'] == '889515b191ead51303b1321de858ec4e2209722c'
assert set(new) - set(old) - {vcs} == {'src/clock.rs', 'src/clock_named.rs'}
assert not (set(old) - set(new) - {vcs})
allowed_changes = {'Cargo.toml', 'Cargo.toml.orig', 'Cargo.lock', vcs,
                   'src/cli.rs', 'src/elf.rs', 'src/lib.rs', 'src/report.rs',
                   'src/simulate.rs', 'src/toolchain.rs'}
changed = {name for name in old.keys() & new.keys() if old[name] != new[name]}
assert not changed - allowed_changes, changed - allowed_changes
for name, data in new.items():
    if name.startswith('src/') or name in ['README.md', 'LICENSE', 'NOTICE.md', 'Cargo.lock', 'Cargo.toml.orig']:
        source = Path('Cargo.toml' if name == 'Cargo.toml.orig' else name)
        assert source.read_bytes() == data, name
manifest = tomllib.loads(new['Cargo.toml.orig'].decode())
normalized = tomllib.loads(new['Cargo.toml'].decode())
previous = tomllib.loads(old['Cargo.toml.orig'].decode())
assert previous['package']['version'] == '0.2.4'
previous['package']['version'] = '0.3.0'
previous['dependencies']['libc-abi-tools'] = '0.4.0'
assert previous == manifest
assert normalized['package']['version'] == '0.3.0'
assert not manifest.get('patch') and not manifest.get('replace')
assert not normalized.get('patch') and not normalized.get('replace')
assert all('path' not in dependency and 'git' not in dependency for dependency in normalized['dependencies'].values())
old_lock = tomllib.loads(old['Cargo.lock'].decode())
new_lock = tomllib.loads(new['Cargo.lock'].decode())
keep = lambda d: {p['name']: p for p in d['package'] if p['name'] not in ['cargo-libc', 'libc-abi-tools']}
assert keep(old_lock) == keep(new_lock)
root = next(p for p in new_lock['package'] if p['name'] == 'libc-abi-tools')
assert root['version'] == '0.4.0' and root['checksum'] == '915611aa54c3fe77eefedb34bac233b077e59669130a116e2f20bc25297af009'
assert all(p['source'] == 'registry+https://github.com/rust-lang/crates.io-index' for p in new_lock['package'] if p['name'] != 'cargo-libc')
# Preserve the existing C startup/default-library/unwinder implementation.
provider_link = lambda b: b.split(b'fn provider_link(\n', 1)[1].split(b'/// rustc\'s arguments', 1)[0]
assert provider_link(old['src/toolchain.rs']) == provider_link(new['src/toolchain.rs'])
assert old['src/cli.rs'].split(b'"#;', 1)[1] == new['src/cli.rs'].split(b'"#;', 1)[1]
for name in ['src/report.rs', 'src/simulate.rs']:
    assert old[name].split(b'#[cfg(test)]', 1)[0] == new[name].split(b'#[cfg(test)]', 1)[0], name
readme = new['README.md'].decode()
assert '[Release policy and records](https://github.com/rustcommons/cargo-libc/blob/main/docs/RELEASING.md)' in readme
assert 'Current release:' not in readme
record = {'baseline': 'official cargo-libc 0.2.4', 'candidate': 'cargo-libc 0.3.0',
          'changed': sorted(changed - {vcs}), 'added': sorted(set(new) - set(old) - {vcs}),
          'provider_link_and_unwinder_body_unchanged': True, 'other_registry_lock_entries_unchanged': True,
          'payload_sha256': {name: hashlib.sha256(data).hexdigest() for name, data in sorted(new.items())}}
evidence.mkdir(parents=True, exist_ok=True)
(evidence / 'package-payload-comparison.json').write_text(json.dumps(record, indent=2) + '\n')
print('Package matches candidate source; only approved clock source/metadata changes versus official 0.2.4; provider/unwinder body and other registry entries unchanged.')
