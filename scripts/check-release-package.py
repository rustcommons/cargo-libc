#!/usr/bin/env python3
"""Audit provider-cache adoption against official Cargo 0.6.0, without overrides."""
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
BASE_SHA = '63d7071aa96774b159405e0d6b295e4762f8d0ea0c84d4d69818e4de9c84c62a'
BASE_COMMIT = '460bca52512658df8fea8d76578cd83dd0c2cb84'
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



version = tomllib.loads(Path('Cargo.toml').read_text())['package']['version']
baseline = get('https://static.crates.io/crates/cargo-libc/cargo-libc-0.6.0.crate')
assert sha(baseline) == BASE_SHA and len(baseline) == 85625
registry = json.loads(get('https://crates.io/api/v1/crates/cargo-libc/0.6.0'))['version']
assert registry['checksum'] == BASE_SHA and registry['yanked'] is False
(evidence / 'official-cargo-libc-0.6.0.crate').write_bytes(baseline)
old = payload(baseline, 'cargo-libc-0.6.0')
candidate = package.read_bytes()
new = payload(candidate, 'cargo-libc-' + version)
assert len(old) == len(new) == 28 and old.keys() == new.keys()
assert json.loads(old['.cargo_vcs_info.json'])['git']['sha1'] == BASE_COMMIT
head = git('rev-parse', 'HEAD').decode().strip()
assert not git('status', '--porcelain'), 'package must come from a clean checkout'
vcs = json.loads(new['.cargo_vcs_info.json'])
assert vcs['git']['sha1'] == head and not vcs['git'].get('dirty', False)
assert vcs.get('path_in_vcs', '') == ''
changed = sorted(name for name in old if old[name] != new[name])
assert set(changed) <= {'Cargo.toml', 'Cargo.toml.orig', 'Cargo.lock', '.cargo_vcs_info.json'}
for name, data in new.items():
    if name not in ['Cargo.toml', '.cargo_vcs_info.json']:
        source = 'Cargo.toml' if name == 'Cargo.toml.orig' else name
        assert Path(source).read_bytes() == data, name
for name in ['Cargo.toml.orig', 'Cargo.toml']:
    previous = tomllib.loads(old[name].decode())
    current = tomllib.loads(new[name].decode())
    previous['package']['version'] = version
    dep = previous['dependencies']['libc-abi-tools']
    if isinstance(dep, str):
        assert dep == '0.5.0'
        previous['dependencies']['libc-abi-tools'] = '0.6.0'
    else:
        assert dep['version'] == '0.5.0'
        dep['version'] = '0.6.0'
    assert previous == current, name
    assert not current.get('patch') and not current.get('replace')
for dep in current['dependencies'].values():
    assert not any(k in dep for k in ['path', 'git', 'registry']), dep
previous = tomllib.loads(old['Cargo.lock'].decode())
current = tomllib.loads(new['Cargo.lock'].decode())
next(p for p in previous['package'] if p['name'] == 'cargo-libc')['version'] = version
producer = next(p for p in previous['package'] if p['name'] == 'libc-abi-tools')
producer['version'] = '0.6.0'
producer['checksum'] = '368579c7f6401ead63390c8c273da9d9840a905438b62e603917bb169e084d6e'
assert previous == current, 'unrelated lock entries must remain unchanged'
for dep in current['package']:
    if dep['name'] != 'cargo-libc':
        assert dep['source'] == REGISTRY
        published = json.loads(get(f"https://crates.io/api/v1/crates/{dep['name']}/{dep['version']}"))['version']
        assert published['checksum'] == dep['checksum'] and not published['yanked']
        archive = get(f"https://static.crates.io/crates/{dep['name']}/{dep['name']}-{dep['version']}.crate")
        assert sha(archive) == dep['checksum'], dep['name']
record = dict(audit_passed=True, baseline_version='0.6.0', baseline_sha256=BASE_SHA,
              candidate_version=version, candidate_commit=head, sha256=sha(candidate),
              bytes=len(candidate), files=len(new), changed_from_official=changed,
              source_readme_legal_unchanged=True, registry_checksums_verified=True,
              unrelated_dependencies_unchanged=True)
(evidence / 'package-payload-comparison.json').write_text(json.dumps(record, indent=2) + '\n')
(evidence / 'package-files.txt').write_text('\n'.join(sorted(new)) + '\n')
print(json.dumps(record, indent=2))
