#!/usr/bin/env python3
"""Exercise installed duplicate-option contracts without preparing build inputs."""
import json
from pathlib import Path
import subprocess
import sys

tool, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=False)
results = []


def run(name, args, error=None):
    result = subprocess.run([str(tool), *args], capture_output=True, timeout=30)
    (out / (name + '.stdout')).write_bytes(result.stdout)
    (out / (name + '.stderr')).write_bytes(result.stderr)
    expected = 1 if error else 0
    results.append(dict(case=name, args=args, status=result.returncode, expected=expected))
    (out / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    assert result.returncode == expected, (name, result.stderr)
    if error:
        assert not result.stdout and error.encode() in result.stderr, name
    else:
        assert result.stdout and not result.stderr, name
    return result


for command in ['doctor', 'build', 'check', 'test', 'run', 'clippy', 'doc', 'rustc', 'install', 'env']:
    for flag in ['--sysroot', '--gcc-runtime', '--cc', '--ld']:
        for label, value in [('same', 'first path'), ('different', 'second path')]:
            run(f'{command}-{flag[2:]}-{label}',
                ['libc', command, flag, 'first path', flag, value], f'duplicate {flag}')
for command, flag, first, other in [('build', '--artifact', 'app', 'other app'),
                                     ('inspect', '--max-glibc', '2.17', '2.22')]:
    for label, value in [('same', first), ('different', other)]:
        run(f'{command}-{flag[2:]}-{label}', [command, flag, first, flag, value],
            f'duplicate {flag}')
run('help', ['--help'])
run('version', ['--version'])
run('single-ceiling', ['inspect', '/bin/true', '--max-glibc', '2.44'])
run('single-ceiling-json', ['inspect', '/bin/true', '--max-glibc', '2.44', '--json'])
json.loads(run('repeated-json', ['inspect', '/bin/true', '--json', '--json']).stdout)
print(f'PASS: {len(results)} installed CLI cases; argument errors precede input resolution')
