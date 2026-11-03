#!/usr/bin/env python3
"""Audit the diagnostics candidate against official 0.3.0 and reviewed source."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tomllib
import urllib.request

package, evidence = map(Path, sys.argv[1:3])
evidence.mkdir(parents=True, exist_ok=True)
BASE_SHA = '96201f1b35c093b3ed77867995bc688336fc2cbb5a43f6d471a9e13b9ab3f5c2'
BASE_COMMIT = 'a70ce9ffac6be8611b66a2829fefb3093cb41ce6'
SOURCE = '748ffa23274792dd21e1cb4cd812271f1f61e1a3'
PREP = '62347644fe1e6808b19de043c702da69077409e8'
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'


def git(*args):
    return subprocess.check_output(['git', *args])


def sha(data):
    return hashlib.sha256(data).hexdigest()


def get(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        return response.read()


def payload(data, prefix):
    result = {}
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for member in archive:
            assert member.name.startswith(prefix + '/'), member.name
            assert member.isdir() or member.isfile(), member.name
            if member.isfile():
                name = member.name[len(prefix) + 1:]
                assert name and '..' not in Path(name).parts and name not in result, name
                result[name] = archive.extractfile(member).read()
    return result


baseline = get('https://static.crates.io/crates/cargo-libc/cargo-libc-0.3.0.crate')
assert sha(baseline) == BASE_SHA and len(baseline) == 70579
registry = json.loads(get('https://crates.io/api/v1/crates/cargo-libc/0.3.0'))['version']
assert registry['checksum'] == BASE_SHA and registry['yanked'] is False
(evidence / 'official-cargo-libc-0.3.0.crate').write_bytes(baseline)
old = payload(baseline, 'cargo-libc-0.3.0')
candidate_bytes = package.read_bytes()
new = payload(candidate_bytes, 'cargo-libc-0.4.0')
record = {
    'baseline': {'version': '0.3.0', 'sha256': BASE_SHA, 'bytes': len(baseline),
                 'files': len(old), 'commit': BASE_COMMIT, 'yanked': False},
    'candidate': {'version': '0.4.0', 'sha256': sha(candidate_bytes),
                  'bytes': len(candidate_bytes), 'files': len(new)},
    'payload': {name: {'bytes': len(data), 'sha256': sha(data)}
                for name, data in sorted(new.items())},
    'audit_passed': False,
}
report = evidence / 'package-payload-comparison.json'
# Preserve inventory/digests even if a later assertion fails.
report.write_text(json.dumps(record, indent=2) + '\n')
(evidence / 'package-files.txt').write_text('\n'.join(sorted(new)) + '\n')
assert len(old) == 25 and len(new) == 26
assert json.loads(old['.cargo_vcs_info.json'])['git']['sha1'] == BASE_COMMIT
head = git('rev-parse', 'HEAD').decode().strip()
assert not git('status', '--porcelain'), 'package must come from a clean checkout'
vcs = json.loads(new['.cargo_vcs_info.json'])
assert vcs['git']['sha1'] == head and not vcs['git'].get('dirty', False), vcs
assert vcs.get('path_in_vcs', '') == ''
assert set(new) - set(old) == {'src/capability.rs'}
assert not set(old) - set(new)
allowed = {'Cargo.toml', 'Cargo.toml.orig', 'Cargo.lock', '.cargo_vcs_info.json', 'README.md',
           'src/cli.rs', 'src/lib.rs', 'src/simulate.rs', 'src/toolchain.rs'}
changed = {name for name in old.keys() & new.keys() if old[name] != new[name]}
assert changed <= allowed, changed - allowed
assert {name for name in changed if name.startswith('src/')} == {
    'src/cli.rs', 'src/lib.rs', 'src/simulate.rs', 'src/toolchain.rs'}
for name, data in new.items():
    if name.startswith('src/') or name in ['README.md', 'LICENSE', 'NOTICE.md',
                                          'Cargo.lock', 'Cargo.toml.orig']:
        source = 'Cargo.toml' if name == 'Cargo.toml.orig' else name
        assert Path(source).read_bytes() == data, name
    if name.startswith('src/'):
        assert git('show', f'{SOURCE}:{name}') == data, name
        assert git('show', f'{PREP}:{name}') == data, name
for name in ['LICENSE', 'NOTICE.md']:
    assert new[name] == old[name] == git('show', f'{PREP}:{name}'), name
readme_links = (b'[Changelog](https://github.com/rustcommons/cargo-libc/blob/main/CHANGELOG.md) \xc2\xb7\n'
                b'[GitHub Releases](https://github.com/rustcommons/cargo-libc/releases) \xc2\xb7\n')
policy_link = b'[Release policy and records]'
assert old['README.md'].count(policy_link) == 1
assert old['README.md'].replace(policy_link, readme_links + policy_link, 1) == new['README.md']
assert new['README.md'] == git('show', f'{PREP}:README.md') == git('show', f'{SOURCE}:README.md')
for name in ['Cargo.toml.orig', 'Cargo.toml']:
    previous = tomllib.loads(old[name].decode())
    current = tomllib.loads(new[name].decode())
    assert previous['package']['version'] == '0.3.0'
    previous['package']['version'] = '0.4.0'
    assert previous == current, name
    assert not current.get('patch') and not current.get('replace')
assert new['Cargo.toml.orig'] == git('show', f'{PREP}:Cargo.toml')
assert new['Cargo.lock'] == git('show', f'{PREP}:Cargo.lock')
manifest = tomllib.loads(new['Cargo.toml'].decode())
for dep in manifest['dependencies'].values():
    assert 'path' not in dep and 'git' not in dep and 'registry' not in dep, dep
previous = tomllib.loads(old['Cargo.lock'].decode())
current = tomllib.loads(new['Cargo.lock'].decode())
for lock, version in [(previous, '0.3.0'), (current, '0.4.0')]:
    root = next(p for p in lock['package'] if p['name'] == 'cargo-libc')
    assert root['version'] == version
    root['version'] = 'ROOT'
assert previous == current, 'only the root lock version may change'
for dep in current['package']:
    if dep['name'] != 'cargo-libc':
        assert dep['source'] == REGISTRY
        published = json.loads(get(f"https://crates.io/api/v1/crates/{dep['name']}/{dep['version']}"))['version']
        assert published['checksum'] == dep['checksum'] and not published['yanked'], dep['name']
        data = get(f"https://static.crates.io/crates/{dep['name']}/{dep['name']}-{dep['version']}.crate")
        assert sha(data) == dep['checksum'], dep['name']
# Preserve target C startup/default-library and unwinder argument construction.
provider_link = lambda b: b.split(b'fn provider_link(\n', 1)[1].split(b'/// rustc\'s arguments', 1)[0]
assert provider_link(old['src/toolchain.rs']) == provider_link(new['src/toolchain.rs'])
record.update({'audit_passed': True, 'candidate_commit': head,
               'source_equal': [SOURCE, PREP], 'changed_from_official': sorted(changed),
               'added': ['src/capability.rs'], 'registry_checksums_verified': True,
               'readme_delta': 'exactly two reviewed Changelog/GitHub Releases navigation lines',
               'root_version_only_manifest_lock': True,
               'provider_link_unwinder_body_unchanged': True})
report.write_text(json.dumps(record, indent=2) + '\n')
print(f"PASS: {len(new)} files, {len(candidate_bytes)} bytes, SHA256 {sha(candidate_bytes)}, clean VCS {head}; diagnostics-only delta from official 0.3.0")
