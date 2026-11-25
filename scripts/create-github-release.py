#!/usr/bin/env python3
"""Resume a draft, check every asset, then publish. Never replace assets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def gh(*args):
    return subprocess.check_output(['gh', *args])


def release(repo, tag):
    pages = json.loads(gh('api', '--paginate', '--slurp', f'repos/{repo}/releases?per_page=100'))
    matches = [r for page in pages for r in page if r['tag_name'] == tag]
    if len(matches) > 1:
        raise ValueError('ambiguous release tag')
    return matches[0] if matches else None


def check_metadata(record, tag, title, body):
    if (record['tag_name'], record['name'], record['body'], record['prerelease']) != (tag, title, body, False):
        raise ValueError('release metadata differs from committed notes')


def verify_signature(dist):
    subprocess.run(['bash', str(Path(__file__).with_name('release-signing.sh')),
                    'verify', str(dist)], check=True)


def check_assets(repo, record, files, complete):
    assets = json.loads(gh('api', '--paginate', '--slurp',
                          f"repos/{repo}/releases/{record['id']}/assets?per_page=100"))
    assets = [a for page in assets for a in page]
    expected = {p.name: p for p in files}
    names = [a['name'] for a in assets]
    if len(names) != len(set(names)) or set(names) - expected.keys():
        raise ValueError('unexpected or duplicate release assets')
    if complete and set(names) != expected.keys():
        raise ValueError('release assets missing')
    for asset in assets:
        path = expected[asset['name']]
        digest = 'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest()
        if asset['size'] != path.stat().st_size or asset.get('digest') != digest:
            if path.name != 'SHA256SUMS.asc':
                raise ValueError(f"asset size or digest mismatch: {path.name}")
            # Detached signatures include signing time. A rerun may produce a
            # different valid signature; keep the remote bytes if they verify
            # against these exact checksums and the production fingerprint pin.
            data = gh('api', '-H', 'Accept: application/octet-stream',
                      f"repos/{repo}/releases/assets/{asset['id']}")
            if len(data) != asset['size'] or 'sha256:' + hashlib.sha256(data).hexdigest() != asset.get('digest'):
                raise ValueError('downloaded signature differs from asset metadata')
            path.write_bytes(data)
            verify_signature(path.parent)
    return set(names)


def publish(repo, version, dist, notes):
    tag = f'v{version}'
    body = notes.read_text()
    title = next(line[2:] for line in body.splitlines() if line.startswith('# '))
    files = [*sorted(dist.glob('*.crate')), *sorted(dist.glob('*.cdx.json')),
             dist / 'SHA256SUMS', dist / 'SHA256SUMS.asc',
             dist / f'cargo-libc-{version}.crates.intoto.jsonl']
    if not list(dist.glob('*.crate')) or not list(dist.glob('*.cdx.json')):
        raise ValueError('crate or SBOM missing')
    for path in files:
        if not path.is_file():
            raise ValueError(f'missing asset: {path}')
    record = release(repo, tag)
    if record is None:
        gh('release', 'create', tag, '--repo', repo, '--verify-tag', '--draft',
           '--title', title, '--notes-file', str(notes))
        record = release(repo, tag)
        if record is None or not record['draft']:
            raise ValueError('new release is not a draft')
    check_metadata(record, tag, title, body)
    existing = check_assets(repo, record, files, complete=not record['draft'])
    if not record['draft']:
        return  # A published release must already match; never modify it.
    for path in files:
        if path.name not in existing:
            gh('release', 'upload', tag, str(path), '--repo', repo)
            check_assets(repo, record, files, complete=False)
    record = release(repo, tag)
    check_metadata(record, tag, title, body)
    if not record['draft']:
        raise ValueError('release published before final asset checks')
    check_assets(repo, record, files, complete=True)
    gh('release', 'edit', tag, '--repo', repo, '--draft=false')
    record = release(repo, tag)
    check_metadata(record, tag, title, body)
    if record['draft']:
        raise ValueError('release is still a draft')
    check_assets(repo, record, files, complete=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dist', required=True, type=Path)
    parser.add_argument('--version', required=True)
    args = parser.parse_args()
    publish(os.environ['GITHUB_REPOSITORY'], args.version, args.dist,
            Path(f'docs/releases/v{args.version}.md'))
