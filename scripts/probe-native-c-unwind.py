#!/usr/bin/env python3
"""Diagnostic plain-C versus -fexceptions unwind controls, never product policy.

Requires caller-verified prototype archives and existing native tools. All worker
frames are C. Records every failure and exits nonzero if a required case fails.
No downloads, runtime selection fallback, Cargo changes or target claims.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--evidence', type=Path, required=True)
parser.add_argument('--cc', default='/usr/bin/gcc')
parser.add_argument('--linker', choices=['bfd', 'lld'], required=True)
parser.add_argument('--hidden-archive', type=Path, required=True)
parser.add_argument('--hidden-sha256', required=True)
parser.add_argument('--minimal-archive', type=Path, required=True)
parser.add_argument('--minimal-sha256', required=True)
a = parser.parse_args()
source = Path(__file__).resolve().parents[1] / 'tests/fixtures/unwind-coexistence/native_load_controls.c'
archives = {}
for mode, path, expected in [('hidden', a.hidden_archive, a.hidden_sha256),
                             ('minimal', a.minimal_archive, a.minimal_sha256)]:
    path = path.resolve(strict=True)
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        sys.exit(f'{mode} archive checksum mismatch: {actual}')
    archives[mode] = path
out = a.evidence.resolve()
out.mkdir(parents=True, exist_ok=False)
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
records = []
failed = False


def save():
    (out / 'results.json').write_text(json.dumps(records, indent=2) + '\n')


def capture(argv, name, env=None, seconds=120):
    command = ['timeout', '--kill-after=2s', f'{seconds}s', *map(str, argv)]
    result = subprocess.run(command, capture_output=True, timeout=seconds + 8, env=env)
    (out / (name + '.stdout')).write_bytes(result.stdout)
    (out / (name + '.stderr')).write_bytes(result.stderr)
    return result, command


(out / 'inputs.json').write_text(json.dumps({
    'driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
    'archives': {k: {'path': str(v), 'sha256': hashlib.sha256(v.read_bytes()).hexdigest()}
                 for k, v in archives.items()},
    'cc': a.cc, 'linker': a.linker,
    'scope': 'Standalone C-only worker diagnostic; prototype whole-archive/export controls, not production linkage',
}, indent=2) + '\n')
try:
    for tool, args in [('cc', [a.cc, '--version']), ('readelf', ['readelf', '--version'])]:
        result, command = capture(args, tool + '-version')
        if result.returncode:
            raise RuntimeError(f'{tool} version probe failed')
    for variant, extra in [('plain', []), ('eh', ['-fexceptions'])]:
        obj = out / (variant + '.o')
        result, command = capture([a.cc, '-std=c11', '-O2', '-g', '-pthread', *extra,
                                   '-c', source, '-o', obj], variant + '-compile')
        records.append({'kind': 'compile', 'variant': variant, 'command': command,
                        'exit': result.returncode, 'pass': result.returncode == 0})
        save()
        if result.returncode:
            failed = True
            continue
        result, _ = capture(['nm', '-u', obj], variant + '-undefined')
        if result.returncode:
            raise RuntimeError('undefined-symbol inventory failed')
        if variant == 'eh' and b'__gcc_personality_v0' not in result.stdout:
            raise RuntimeError('EH control did not emit the required native C personality; scope is not demonstrated')
        for mode in ['gcc', 'hidden', 'minimal']:
            binary = out / (variant + '-' + mode)
            args = [a.cc, obj, '-pthread', '-o', binary, '-fuse-ld=' + a.linker,
                    '-Wl,-Map=' + str(binary) + '.map']
            if mode != 'gcc':
                args += ['-Wl,--whole-archive', archives[mode], '-Wl,--no-whole-archive']
            if mode == 'minimal':
                args += ['-Wl,--export-dynamic-symbol=_Unwind_*']
            result, command = capture(args, binary.name + '-link')
            records.append({'kind': 'link', 'variant': variant, 'mode': mode,
                            'command': command, 'exit': result.returncode,
                            'pass': result.returncode == 0})
            save()
            if result.returncode:
                failed = True
                continue
            for suffix, flags in [('elf', ['-W', '-h', '-d', '-V', '--dyn-syms']),
                                  ('comment', ['-p', '.comment'])]:
                result, _ = capture(['readelf', *flags, binary], binary.name + '-' + suffix)
                if result.returncode:
                    raise RuntimeError('ELF evidence capture failed')
            for case in ['backtrace', 'pthread-exit', 'cancel']:
                env = dict(os.environ)
                for key in ['LD_PRELOAD', 'LD_LIBRARY_PATH', 'LD_AUDIT', 'LD_DEBUG_OUTPUT']:
                    env.pop(key, None)
                env['LD_DEBUG'] = 'libs,versions,bindings'
                result, command = capture([binary, case], binary.name + '-' + case, env, 12)
                passed = result.returncode == 0 and (
                    bool(result.stdout) if case == 'backtrace' else b'native-cleanup-ok\n' in result.stdout)
                records.append({'kind': 'runtime', 'variant': variant, 'mode': mode,
                                'case': case, 'command': command, 'exit': result.returncode,
                                'expected': 0, 'pass': passed})
                failed |= not passed
                save()
finally:
    save()
print(f'Native C unwind diagnostic: {"FAIL (inspect retained cases)" if failed else "PASS in named scope"}: {out}')
sys.exit(1 if failed else 0)
