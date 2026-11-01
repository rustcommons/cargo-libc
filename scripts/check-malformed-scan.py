#!/usr/bin/env python3
"""Bounded consumer parsing checks, including acceptance of real large Rust ELFs."""
import json
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


def inspect(path, args, memory=512 * 1024 * 1024, timeout=10):
    def limits():
        resource.setrlimit(resource.RLIMIT_AS, (memory,) * 2)
    return subprocess.run([binary, 'inspect', str(path), *args], capture_output=True,
                          timeout=timeout, preexec_fn=limits)


cases = {}
for field in [24, 32]:
    data = bytearray(original)
    struct.pack_into('<Q', data, shoff + dynsym * 64 + field, 2**64 - 1)
    cases[f'overflow-{field}'] = (data, 'reject')
# Only the first dynamic-symbol section is selected. Aliased extra headers are
# accepted; they must not trigger repeated record processing or amplification.
data = bytearray(original)
struct.pack_into('<Q', data, 40, len(original))
struct.pack_into('<H', data, 60, count + 2000)
data += b''.join(sections) + sections[dynsym] * 2000
cases['aliased-headers'] = (data, 'accept')
# Relocate only the section-based version chain so direct program-header
# inspection still succeeds. Its next main record reuses the just-consumed aux
# record: a genuine overlapping chain, not a large valid fixed-size table.
vn = next(i for i, s in enumerate(sections) if struct.unpack_from('<I', s, 4)[0] == 0x6ffffffe)
offset = struct.unpack_from('<Q', sections[vn], 24)[0]
chain = bytearray(original[offset:offset + 32])
struct.pack_into('<H', chain, 2, 1)
struct.pack_into('<I', chain, 8, 16)
struct.pack_into('<I', chain, 12, 16)
struct.pack_into('<I', chain, 28, 0)
data = bytearray(original)
struct.pack_into('<Q', data, shoff + vn * 64 + 24, len(original))
struct.pack_into('<Q', data, shoff + vn * 64 + 32, len(chain))
data += chain
cases['reused-version-chain'] = (data, 'overlap')
with tempfile.TemporaryDirectory(prefix='cargo-libc-malformed-') as tmp:
    for name, (data, expectation) in cases.items():
        path = Path(tmp) / name
        path.write_bytes(data)
        plain = inspect(path, [])
        assert plain.returncode == 0, plain.stderr
        for args in [['--scan'], ['--kernel', '3.10'], ['--distro', 'sles-12-sp5']]:
            result = inspect(path, args + ['--json'])
            if expectation == 'accept':
                assert result.returncode in (0, 2), (name, args, result)
                assert 'scan' in json.loads(result.stdout), result
            else:
                assert result.returncode == 1, (name, args, result)
                assert b'panicked' not in result.stderr and b'memory allocation' not in result.stderr
                if expectation == 'overlap':
                    assert b'overlap' in result.stderr, result.stderr
        print(f'{name}: {expectation}; all scanner entry paths bounded by 512 MiB/10s')

# Real official toolchain binaries exercise valid fixed tables above 65,536
# records. Do not mistake a large fixed table for variable-chain amplification.
cargo = Path(subprocess.check_output(['rustup', 'which', 'cargo'], text=True).strip())
sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
driver = next((sysroot / 'lib').glob('librustc_driver-*.so'))
for path in [cargo, driver]:
    data = path.read_bytes()
    shoff = struct.unpack_from('<Q', data, 40)[0]
    count = struct.unpack_from('<H', data, 60)[0]
    fixed_counts = []
    for i in range(count):
        at = shoff + i * 64
        kind = struct.unpack_from('<I', data, at + 4)[0]
        size, entry = struct.unpack_from('<Q', data, at + 32)[0], struct.unpack_from('<Q', data, at + 56)[0]
        if kind in (4, 11) and entry:
            fixed_counts.append(size // entry)
    assert max(fixed_counts) > 65536, (path, fixed_counts)
    result = inspect(path, ['--scan', '--json'], memory=1536 * 1024 * 1024, timeout=60)
    assert result.returncode in (0, 2), (path, result.stderr)
    assert 'scan' in json.loads(result.stdout), path
    print(f'{path.name}: accepted; largest fixed table {max(fixed_counts)} records; 1536 MiB/60s bound')
