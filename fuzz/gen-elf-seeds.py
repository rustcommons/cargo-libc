#!/usr/bin/env python3
"""Write the `elf` fuzz target's seed corpus (stdlib only).

Seeds are generated, not committed, so the repository holds no binary
artifacts. Each seed is byte-for-byte the matching unit-test fixture:
`executable-verneed` and `library-verdef` are `fixture()` and
`library_fixture()` in src/elf.rs; `exact-dynsym` is `fixture()` in
src/elf/exact.rs. The pinned SHA-256 values are the previously committed seed
files; generation fails if the output ever differs.

    python3 fuzz/gen-elf-seeds.py OUTPUT_DIR
"""
import hashlib
import struct
import sys
from pathlib import Path

EXPECTED = {
    'executable-verneed': '7c679c5036d3e93fad45ee90985ae97a06924d846c57e75a4a4f6b16a01264d6',
    'library-verdef': '9198b8cec4c1fa26ddb5358ec51259f64fdf9a1b975e912697210d6c3ac0d727',
    'exact-dynsym': 'f3fdb403afaacda9fed70f89442bc972ede27fffa867e7e4a00904a96a1771a1',
}

PT_LOAD, PT_DYNAMIC, PT_INTERP = 1, 2, 3
DT_NULL, DT_NEEDED, DT_STRTAB, DT_STRSZ = 0, 1, 5, 10
DT_RPATH, DT_RUNPATH = 15, 29
DT_VERDEF, DT_VERDEFNUM = 0x6FFFFFFC, 0x6FFFFFFD
DT_VERNEED, DT_VERNEEDNUM = 0x6FFFFFFE, 0x6FFFFFFF
VER_FLG_BASE, VER_FLG_WEAK = 1, 2


def put(b, fmt, at, value):
    struct.pack_into('<' + fmt, b, at, value)


def header(b, phnum):
    b[:7] = b'\x7fELF\x02\x01\x01'
    put(b, 'H', 16, 3)   # ET_DYN
    put(b, 'H', 18, 62)  # EM_X86_64
    put(b, 'I', 20, 1)
    put(b, 'Q', 32, 64)
    put(b, 'H', 52, 64)
    put(b, 'H', 54, 56)
    put(b, 'H', 56, phnum)


# src/elf.rs tests
BASE, DYNAMIC, STRINGS, VERNEED, INTERP = 0x400000, 0x200, 0x300, 0x400, 0x500


def ph(b, index, kind, offset, size):
    start = 64 + index * 56
    put(b, 'I', start, kind)
    put(b, 'Q', start + 8, offset)
    put(b, 'Q', start + 16, BASE + offset)
    put(b, 'Q', start + 32, size)
    put(b, 'Q', start + 40, size)


def dyn_entry(b, index, tag, value):
    put(b, 'Q', DYNAMIC + index * 16, tag)
    put(b, 'Q', DYNAMIC + index * 16 + 8, value)


def string(b, state, value):
    offset = state[0]
    data = value.encode()
    b[STRINGS + offset:STRINGS + offset + len(data)] = data
    b[STRINGS + offset + len(data)] = 0
    state[0] = offset + len(data) + 1
    return offset


def executable_verneed():
    b = bytearray(0x600)
    header(b, 3)
    ph(b, 0, PT_LOAD, 0, len(b))
    ph(b, 1, PT_DYNAMIC, DYNAMIC, 9 * 16)
    interp = b'/lib64/ld-linux-x86-64.so.2\0'
    ph(b, 2, PT_INTERP, INTERP, len(interp))
    b[INTERP:INTERP + len(interp)] = interp
    state = [1]
    libc = string(b, state, 'libc.so.6')
    libgcc = string(b, state, 'libgcc_s.so.1')
    old = string(b, state, 'GLIBC_2.2.5')
    newer_weak = string(b, state, 'GLIBC_2.34')
    gcc = string(b, state, 'GCC_3.0')
    rpath = string(b, state, '/old/lib:/old/lib64')
    runpath = string(b, state, '$ORIGIN/lib::/usr/lib64')
    for i, (tag, value) in enumerate([
            (DT_STRTAB, BASE + STRINGS), (DT_STRSZ, state[0]), (DT_NEEDED, libc),
            (DT_NEEDED, libgcc), (DT_VERNEED, BASE + VERNEED), (DT_VERNEEDNUM, 2),
            (DT_RPATH, rpath), (DT_RUNPATH, runpath), (DT_NULL, 0)]):
        dyn_entry(b, i, tag, value)
    put(b, 'H', VERNEED, 1)
    put(b, 'H', VERNEED + 2, 2)
    put(b, 'I', VERNEED + 4, libc)
    put(b, 'I', VERNEED + 8, 16)
    put(b, 'I', VERNEED + 12, 48)
    put(b, 'H', VERNEED + 16 + 6, 2)
    put(b, 'I', VERNEED + 16 + 8, old)
    put(b, 'I', VERNEED + 16 + 12, 16)
    put(b, 'H', VERNEED + 32 + 4, VER_FLG_WEAK)
    put(b, 'H', VERNEED + 32 + 6, 3)
    put(b, 'I', VERNEED + 32 + 8, newer_weak)
    put(b, 'H', VERNEED + 48, 1)
    put(b, 'H', VERNEED + 48 + 2, 1)
    put(b, 'I', VERNEED + 48 + 4, libgcc)
    put(b, 'I', VERNEED + 48 + 8, 16)
    put(b, 'H', VERNEED + 64 + 6, 4)
    put(b, 'I', VERNEED + 64 + 8, gcc)
    return b


def library_verdef():
    verdef = 0x480
    b = executable_verneed()
    state = [0x80]
    soname = string(b, state, 'libc.so.6')
    old = string(b, state, 'GLIBC_2.2.5')
    new = string(b, state, 'GLIBC_2.34')
    dyn_entry(b, 1, DT_STRSZ, state[0])
    dyn_entry(b, 6, DT_VERDEF, BASE + verdef)
    dyn_entry(b, 7, DT_VERDEFNUM, 3)
    records = [(VER_FLG_BASE, 1, soname, None), (0, 1, old, None), (0, 2, new, old)]
    for index, (flags, count, name, parent) in enumerate(records):
        at = verdef + index * 36
        put(b, 'H', at, 1)
        put(b, 'H', at + 2, flags)
        put(b, 'H', at + 6, count)
        put(b, 'I', at + 12, 20)
        put(b, 'I', at + 16, 0 if index == 2 else 36)
        put(b, 'I', at + 20, name)
        if parent is not None:
            put(b, 'I', at + 24, 8)
            put(b, 'I', at + 28, parent)
    return b


# src/elf/exact.rs tests
def sysv_hash(name):
    h = 0
    for byte in name.encode():
        h = ((h << 4) + byte) & 0xFFFFFFFF
        g = h & 0xF0000000
        h ^= g >> 24
        h &= ~g & 0xFFFFFFFF
    return h


def exact_dynsym():
    b = bytearray(0x1000)
    header(b, 2)
    put(b, 'I', 64, PT_LOAD)
    put(b, 'Q', 80, BASE)
    put(b, 'Q', 96, 0x1000)
    put(b, 'Q', 104, 0x1000)
    put(b, 'I', 120, PT_DYNAMIC)
    put(b, 'Q', 128, 0x100)
    put(b, 'Q', 136, BASE + 0x100)
    put(b, 'Q', 152, 9 * 16)
    put(b, 'Q', 160, 9 * 16)
    strings = b'\0f\0libgcc_s.so.1\0GCC_4.2.0\0'
    b[0x400:0x400 + len(strings)] = strings
    for i, (tag, value) in enumerate([
            (DT_STRTAB, BASE + 0x400), (DT_STRSZ, len(strings)), (6, BASE + 0x500),
            (11, 24), (4, BASE + 0x700), (0x6FFFFFF0, BASE + 0x800),
            (DT_VERNEED, BASE + 0x900), (DT_VERNEEDNUM, 1)]):
        put(b, 'Q', 0x100 + i * 16, tag)
        put(b, 'Q', 0x108 + i * 16, value)
    put(b, 'I', 0x518, 1)
    b[0x51C] = 0x12
    put(b, 'I', 0x700, 1)
    put(b, 'I', 0x704, 2)
    put(b, 'I', 0x708, 1)
    put(b, 'H', 0x802, 2)
    put(b, 'H', 0x900, 1)
    put(b, 'H', 0x902, 1)
    put(b, 'I', 0x904, 3)
    put(b, 'I', 0x908, 16)
    put(b, 'I', 0x910, sysv_hash('GCC_4.2.0'))
    put(b, 'H', 0x916, 2)
    put(b, 'I', 0x918, 17)
    return b


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    seeds = {'executable-verneed': executable_verneed(),
             'library-verdef': library_verdef(),
             'exact-dynsym': exact_dynsym()}
    for name, data in seeds.items():
        digest = hashlib.sha256(data).hexdigest()
        if digest != EXPECTED[name]:
            sys.exit(f'{name}: generated SHA-256 {digest} differs from {EXPECTED[name]}')
        (out / name).write_bytes(data)


if __name__ == '__main__':
    main()
