#!/usr/bin/env python3
"""Consumer boundary checks for malformed section metadata, with memory/time limits."""
import resource
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

binary = str(Path(sys.argv[1]).resolve())
original = Path('/bin/true').read_bytes()
shoff = struct.unpack_from('<Q', original, 40)[0]
count = struct.unpack_from('<H', original, 60)[0]
sections = [original[shoff + i * 64:shoff + (i + 1) * 64] for i in range(count)]
dynsym = next(i for i, s in enumerate(sections) if struct.unpack_from('<I', s, 4)[0] == 11)

def limits():
    resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 * 1024,) * 2)

cases = {}
for field in [24, 32]:
    data = bytearray(original)
    struct.pack_into('<Q', data, shoff + dynsym * 64 + field, 2**64 - 1)
    cases[f'overflow-{field}'] = data
# One selected dynamic-symbol table exceeds the documented record budget.
# Appended records are file-backed; program headers remain untouched.
data = bytearray(original)
struct.pack_into('<Q', data, shoff + dynsym * 64 + 24, len(original))
struct.pack_into('<Q', data, shoff + dynsym * 64 + 32, 65537 * 24)
data += bytes(65537 * 24)
cases['excess-records'] = data
with tempfile.TemporaryDirectory(prefix='cargo-libc-malformed-') as tmp:
    for name, data in cases.items():
        path = Path(tmp) / name
        path.write_bytes(data)
        plain = subprocess.run([binary, 'inspect', str(path)], capture_output=True,
                               timeout=10, preexec_fn=limits)
        assert plain.returncode == 0, plain.stderr
        for option in ['--scan', '--kernel', '--distro']:
            args = [binary, 'inspect', str(path), option]
            if option == '--kernel':
                args += ['3.10']
            elif option == '--distro':
                args += ['sles-12-sp5']
            result = subprocess.run(args, capture_output=True, timeout=10, preexec_fn=limits)
            assert result.returncode == 1, (name, option, result)
            assert b'panicked' not in result.stderr and b'memory allocation' not in result.stderr
            if name == 'excess-records':
                assert b'budget' in result.stderr or b'record limit' in result.stderr, result.stderr
        print(f'{name}: direct inspection passes, all scanner entry paths reject within 512 MiB/10s')
