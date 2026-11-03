#!/usr/bin/env python3
"""Bounded native installed-tool diagnostics. Mock engine is route evidence only."""
import json
import os
from pathlib import Path
import subprocess
import sys

bin_dir, clang, evidence = Path(sys.argv[1]).resolve(), sys.argv[2], Path(sys.argv[3]).resolve()
evidence.mkdir(parents=True, exist_ok=True)
tool = bin_dir / 'cargo-libc'
env = {k: v for k, v in os.environ.items()
       if not k.startswith('CARGO_LIBC_') and k != 'CARGO_CFG_CARGO_LIBC_PROFILE'}
env['PATH'] = str(bin_dir) + os.pathsep + env['PATH']
results = []


def run(label, args, code=0, environment=None, data=None):
    p = subprocess.run([str(a) for a in args], env=env if environment is None else environment,
                       input=data, capture_output=True, timeout=180)
    (evidence / (label + '.stdout')).write_bytes(p.stdout)
    (evidence / (label + '.stderr')).write_bytes(p.stderr)
    results.append({'case': label, 'argv': [str(a) for a in args], 'status': p.returncode,
                    'expected_status': code})
    (evidence / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    assert p.returncode == code, (label, p.returncode, p.stderr.decode(errors='replace'))
    return p


# A valid native ELF with a deliberately absent interpreter; never launch it.
source = evidence / 'missing-loader.c'
source.write_text('int main(void) { return 0; }\n')
program = evidence / 'missing-loader.elf'
missing = evidence / 'absent-runtime/ld-linux-x86-64.so.2'
assert not missing.exists()
run('compile-missing-loader', [clang, source, '-o', program, '-Wl,--dynamic-linker=' + str(missing)])
run('missing-loader-readelf', ['readelf', '-h', '-l', '-d', '--dyn-syms', '--version-info', '--wide', program])
probe_free = env.copy()
probe_free['PATH'] = '/cargo-libc-no-tools'
for label, args in [('help', ['--help']), ('version', ['--version']),
                    ('inspect-text', ['inspect', program, '--max-glibc', '2.44']),
                    ('inspect-json', ['inspect', program, '--max-glibc', '2.44', '--json'])]:
    result = run(label, [tool, *args], environment=probe_free)
    assert result.stdout and not result.stderr, label
    assert b'Tool build provenance:' not in result.stdout, label
assert json.loads((evidence / 'inspect-json.stdout').read_text())['runtime_tested'] is False
runner_env = env.copy()
runner_env['CARGO_LIBC_RUN_GLIBC'] = '2.44'
for label, kernel in [('direct-missing', None), ('local-missing', '3.10')]:
    selected = runner_env.copy()
    for key in ['CARGO_LIBC_RUN_DISTRO', 'CARGO_LIBC_RUN_KERNEL', 'CARGO_LIBC_RUN_RUNNER', 'CARGO_LIBC_RUN_IMAGE']:
        selected.pop(key, None)
    if kernel:
        selected.update(CARGO_LIBC_RUN_KERNEL=kernel, CARGO_LIBC_RUN_RUNNER='local')
    result = run(label, [bin_dir / 'libc-runner', program], code=2, environment=selected)
    assert not result.stdout and b'ELF PT_INTERP' in result.stderr
    assert b'missing on this execution host' in result.stderr
# Closed diagnostic pipe must preserve the deliberate refusal status.
reader, writer = os.pipe()
os.close(reader)
try:
    closed = subprocess.run([str(bin_dir / 'libc-runner'), str(program)], env=runner_env,
                            stdout=subprocess.PIPE, stderr=writer, timeout=180)
finally:
    os.close(writer)
assert closed.returncode == 2 and not closed.stdout
results.append({'case': 'missing-interpreter-closed-stderr', 'status': closed.returncode})
(evidence / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
# Prove missing interpreter does not block the selected container route. This
# engine records the actual invocation and returns 17; it does NOT run an image.
engine = evidence / 'recording-engine'
record = evidence / 'engine-argv.json'
engine.write_text('#!/usr/bin/python3\nimport json,sys\nfrom pathlib import Path\n'
                  + f'Path({str(record)!r}).write_text(json.dumps(sys.argv[1:]))\n'
                  + 'if sys.argv[1:] == ["--version"]: print("recording engine fixture"); sys.exit(0)\n'
                  + 'sys.exit(17)\n')
engine.chmod(0o755)
container_env = runner_env.copy()
container_env.update(CONTAINER_ENGINE=str(engine), KERNEL_ABI_ENGINE_ARGS='', KERNEL_ABI_RUN_ARGS='',
                     CARGO_LIBC_RUN_KERNEL='4.12', CARGO_LIBC_RUN_RUNNER='container',
                     CARGO_LIBC_RUN_IMAGE='fixture-not-executed')
container_env.pop('CARGO_LIBC_RUN_DISTRO', None)
result = run('container-routing-only', [bin_dir / 'libc-runner', program], 17, container_env)
args = json.loads(record.read_text())
assert 'fixture-not-executed' in args and str(program) in args
assert b'missing on this execution host' not in result.stderr
# Runner-only doctor uses no target preparation. Its selected mock route fails;
# that failure remains status 2 rather than an invented local-loader requirement.
result = run('doctor-container-mock', [tool, 'doctor', '--kernel', '4.12', '--runner', 'container',
                                     '--image', 'fixture-not-executed'], 2, container_env)
assert b'Tool build provenance:' in result.stdout and b'Host observation:' in result.stdout
assert b'no local GNU loader requirement' in result.stdout and b'Target input checks skipped' in result.stdout
# Host wrapper units keep native compilers even with target triples nearby.
for name, language, host_key in [('libc-cc', 'c', 'CARGO_LIBC_HOST_CC'),
                                  ('libc-cxx', 'c++', 'CARGO_LIBC_HOST_CXX')]:
    host = env.copy()
    host.pop('CARGO_CFG_CARGO_LIBC_PROFILE', None)
    host[host_key] = clang
    run(name + '-host-compile', [bin_dir / name, '-x', language, '-c', source,
                               '-o', evidence / (name + '-host.o')], environment=host)
print('Installed capability fixtures passed. Container checks use a recording engine only; real musl/static hosts and complete runtime closure remain unqualified.')
