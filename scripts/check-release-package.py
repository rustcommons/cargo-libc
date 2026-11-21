#!/usr/bin/env python3
"""Audit the 0.7.1 package against official cargo-libc 0.7.0, without overrides."""
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
BASE_VERSION = '0.7.0'
BASE_SHA = 'b6b7d6a7e43d76a3a22b19c784b97f234eb9e1538a3341a222b42d7ebd0c3b5d'
BASE_COMMIT = '6317117a2bea3644fa0cba11df5ac61448fd7408'
# The three linux-abi-tools crates move together; libc-abi-tools stays 0.6.0.
MOVED = {
    'kernel-abi-tools': ('0.3.3', '0.6.1', 'c9b45684cc8e511bce6cc2faeeedbb2bc61ea83999af084996eeaaf46731410e'),
    'linux-abi-scan': ('0.3.3', '0.6.1', '24731c0b81bd30c6c78e58d51e5f7454dd5131fb42c6243df01a36f93b01e468'),
    'linux-targets': ('0.3.0', '0.6.1', '56e731b038e838d441d01f141330c5569ca44c34e04c0780a90d6a54e1270792'),
}
# Packaged source changed since 0.7.0 only by cfg(fuzzing) entry points (not
# compiled in shipped builds) and a README badge.
ADDED = {'src/fuzzing.rs'}
SOURCE_CHANGED = {'README.md', 'src/cli.rs', 'src/clock_named.rs', 'src/lib.rs'}
LOCKED_FROM = {'kernel-abi-tools': '0.3.3', 'linux-abi-scan': '0.3.3', 'linux-targets': '0.3.3'}
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
assert version == '0.7.1', version
baseline = get(f'https://static.crates.io/crates/cargo-libc/cargo-libc-{BASE_VERSION}.crate')
assert sha(baseline) == BASE_SHA and len(baseline) == 85624
registry = json.loads(get(f'https://crates.io/api/v1/crates/cargo-libc/{BASE_VERSION}'))['version']
assert registry['checksum'] == BASE_SHA and registry['yanked'] is False
(evidence / f'official-cargo-libc-{BASE_VERSION}.crate').write_bytes(baseline)
old = payload(baseline, f'cargo-libc-{BASE_VERSION}')
candidate = package.read_bytes()
new = payload(candidate, 'cargo-libc-' + version)
assert len(old) == 28 and len(new) == 29 and new.keys() - old.keys() == ADDED, sorted(new.keys() ^ old.keys())
assert not old.keys() - new.keys()
assert json.loads(old['.cargo_vcs_info.json'])['git']['sha1'] == BASE_COMMIT
head = git('rev-parse', 'HEAD').decode().strip()
assert not git('status', '--porcelain'), 'package must come from a clean checkout'
vcs = json.loads(new['.cargo_vcs_info.json'])
assert vcs['git']['sha1'] == head and not vcs['git'].get('dirty', False)
assert vcs.get('path_in_vcs', '') == ''
changed = sorted(name for name in old if old[name] != new[name])
assert set(changed) <= {'Cargo.toml', 'Cargo.toml.orig', 'Cargo.lock', '.cargo_vcs_info.json'} | SOURCE_CHANGED, changed
for name, data in new.items():
    if name not in ['Cargo.toml', '.cargo_vcs_info.json']:
        source = 'Cargo.toml' if name == 'Cargo.toml.orig' else name
        assert Path(source).read_bytes() == data, name
for name in ['Cargo.toml.orig', 'Cargo.toml']:
    previous = tomllib.loads(old[name].decode())
    current = tomllib.loads(new[name].decode())
    previous['package']['version'] = version
    for crate, (before, after, _) in MOVED.items():
        dep = previous['dependencies'][crate]
        if isinstance(dep, str):
            assert dep == before, (crate, dep)
            previous['dependencies'][crate] = after
        else:
            assert dep['version'] == before, (crate, dep)
            dep['version'] = after
    # The only other manifest change declares the cargo-fuzz cfg.
    # Cargo's normalized Cargo.toml adds the default `priority = 0`.
    lint = dict(current['lints']['rust']['unexpected_cfgs'])
    if name == 'Cargo.toml':
        assert lint.pop('priority') == 0, lint
    assert current['lints'] == {'rust': current['lints']['rust']} and len(current['lints']['rust']) == 1
    assert lint == {'level': 'warn', 'check-cfg': ['cfg(fuzzing)']}, current['lints']
    assert 'lints' not in previous
    previous['lints'] = current['lints']
    assert previous == current, name
    assert not current.get('patch') and not current.get('replace')
for dep in current['dependencies'].values():
    assert not any(k in dep for k in ['path', 'git', 'registry']), dep
previous = tomllib.loads(old['Cargo.lock'].decode())
current = tomllib.loads(new['Cargo.lock'].decode())
next(p for p in previous['package'] if p['name'] == 'cargo-libc')['version'] = version
for crate, (_, after, checksum) in MOVED.items():
    entry = next(p for p in previous['package'] if p['name'] == crate)
    assert entry['version'] == LOCKED_FROM[crate], (crate, entry['version'])
    entry['version'] = after
    entry['checksum'] = checksum
assert previous == current, 'unrelated lock entries must remain unchanged'
for dep in current['package']:
    if dep['name'] != 'cargo-libc':
        assert dep['source'] == REGISTRY
        published = json.loads(get(f"https://crates.io/api/v1/crates/{dep['name']}/{dep['version']}"))['version']
        assert published['checksum'] == dep['checksum'] and not published['yanked']
        archive = get(f"https://static.crates.io/crates/{dep['name']}/{dep['name']}-{dep['version']}.crate")
        assert sha(archive) == dep['checksum'], dep['name']
record = dict(audit_passed=True, baseline_version=BASE_VERSION, baseline_sha256=BASE_SHA,
              candidate_version=version, candidate_commit=head, sha256=sha(candidate),
              bytes=len(candidate), files=len(new), added_from_official=sorted(ADDED), changed_from_official=changed,
              packaged_sources_match_checkout=True, legal_files_unchanged=True, registry_checksums_verified=True,
              unrelated_dependencies_unchanged=True)
(evidence / 'package-payload-comparison.json').write_text(json.dumps(record, indent=2) + '\n')
(evidence / 'package-files.txt').write_text('\n'.join(sorted(new)) + '\n')
print(json.dumps(record, indent=2))
