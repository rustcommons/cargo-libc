#!/usr/bin/env python3
"""Bounded native ELF qualification. Never executes a clock-setting fixture."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

new_bin, old_bin, evidence = map(lambda p: Path(p).resolve(), sys.argv[1:4])
evidence.mkdir(parents=True, exist_ok=True)
clocks = ['clock_gettime', 'clock_getres', 'clock_settime', 'clock_nanosleep', 'clock_getcpuclockid']
base = os.environ.copy()
base['CARGO_LIBC_PROVIDER_DIR'] = str(evidence / 'providers')
counter = 0


def run(args, env=None, cwd=None):
    global counter
    counter += 1
    stem = evidence / f'command-{counter:03}'
    stem.with_suffix('.json').write_text(json.dumps({'argv': list(map(str, args)), 'cwd': str(cwd) if cwd else None}, indent=2))
    p = subprocess.run(list(map(str, args)), env=env or base, cwd=cwd, capture_output=True, timeout=240)
    stem.with_suffix('.stdout').write_bytes(p.stdout)
    stem.with_suffix('.stderr').write_bytes(p.stderr)
    stem.with_suffix('.exit').write_text(str(p.returncode))
    if p.returncode:
        raise RuntimeError(f'{args}: {p.returncode}\n{p.stderr.decode(errors="replace")}')
    return p.stdout.decode()


def config(binary, extra):
    text = run([binary / 'cargo-libc', 'env', '--glibc', '2.22', '--cc', '/usr/bin/clang-18', '--format', 'cargo', *extra])
    env = base.copy()
    env.update(tomllib.loads(text)['env'])
    env['CARGO_CFG_CARGO_LIBC_PROFILE'] = 'clock-native-fixture'
    return env, text


automatic, automatic_toml = config(new_bin, [])
(evidence / 'automatic.toml').write_text(automatic_toml)
descriptor = evidence / 'selected-provider.txt'
descriptor.write_text(automatic['CARGO_LIBC_PROVIDER_DESCRIPTOR'])
provider = dict(line.split('\t') for line in descriptor.read_text().splitlines()[1:])
rt = str(Path(provider['stubs']) / 'librt.so.1')


def inspect(path):
    text = run(['readelf', '-W', '--dyn-syms', '--symbols', '--relocs', '--version-info', '--dynamic', path])
    path.with_suffix('.readelf.txt').write_text(text)
    needs, owner = {}, None
    for line in text.splitlines():
        m = re.search(r'File: (\S+)', line)
        if m:
            owner = m[1]
        m = re.search(r'Name: (\S+).*Version: (\d+)', line)
        if m and owner:
            needs[int(m[2])] = (m[1], owner)
    imports = set()
    # Include defined canonical PLT symbols, not just UND entries.
    for line in text.splitlines():
        m = re.search(r'\s(\S+)@+(\S+)\s+\((\d+)\)', line)
        if m:
            version, library = needs[int(m[3])]
            assert version == m[2], (path, line)
            imports.add((m[1], version, library))
    assert imports, path
    return {'imports': sorted(imports), 'needed': re.findall(r'\(NEEDED\).*Shared library: \[(.*?)\]', text),
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


header = '#define _GNU_SOURCE\n#include <time.h>\n#include <sys/types.h>\n'
ordinary = ','.join(f'(void *){c}' for c in clocks)
legacy_decl = '\n'.join(f'extern __typeof__({c}) old_{c}; __asm__(".symver old_{c},{c}@GLIBC_2.2.5");' for c in clocks)
legacy_refs = ','.join(f'(void *)old_{c}' for c in clocks)
calls = '''struct timespec t={0}; clockid_t id;
clock_gettime(0,&t);clock_getres(0,&t);clock_settime(0,&t);clock_nanosleep(0,0,&t,0);clock_getcpuclockid(0,&id);
old_clock_gettime(0,&t);old_clock_getres(0,&t);old_clock_settime(0,&t);old_clock_nanosleep(0,0,&t,0);old_clock_getcpuclockid(0,&id);'''
simple = header + 'void *volatile refs[]={' + ordinary + '}; int main(void){return refs[0]==0;}\n'
mixed = header + legacy_decl + '\nvoid *volatile refs[]={' + ordinary + ',' + legacy_refs + '}; int main(void){return refs[0]==0;}\n'
call_source = header + legacy_decl + '\nvoid retained_calls(void){' + calls + '}\nvoid *volatile refs[]={' + ordinary + ',(void *)retained_calls}; int main(void){return refs[0]==0;}\n'
timer = simple.replace('}; int main', ',(void *)timer_create}; int main')
cases = [
    ('all-five', simple, ['-Wl,--as-needed', '-lrt', '-lc']),
    ('no-as-needed', simple, ['-Wl,--no-as-needed', '-lrt', '-lc']),
    ('state-transition', simple, ['-Wl,--no-as-needed', '-lrt', '-Wl,--as-needed', '-lc']),
    ('group', simple, ['-Wl,--start-group', '-lrt', '-lc', '-Wl,--end-group']),
    ('nested-state', simple, ['-Wl,--push-state,--no-as-needed', '-lrt', '-Wl,--push-state,--as-needed', '-lc', '-Wl,--pop-state,--pop-state']),
    ('timer', timer, ['-Wl,--as-needed', '-lrt', '-lc']),
    ('mixed-address', mixed, ['-lrt', '-lc']),
    ('mixed-calls', call_source, ['-lrt', '-lc']),
    ('fullpath', simple, [rt, '-lc']),
    ('forwarded-fullpath', simple, ['-Wl,' + rt, '-lc']),
    ('xlinker-fullpath', simple, ['-Xlinker', rt, '-lc']),
    ('exact-name', simple, ['-l:librt.so.1', '-lc']),
]
rows = []
for linker in ['/usr/bin/ld.bfd', '/usr/bin/ld.lld-18']:
    selected = cases if linker.endswith('ld.bfd') else [c for c in cases if c[0] in ['all-five', 'mixed-address', 'mixed-calls', 'timer']]
    new_env, _ = config(new_bin, ['--provider', str(descriptor), '--ld', linker])
    old_env, _ = config(old_bin, ['--provider', str(descriptor), '--ld', linker])
    for name, source, args in selected:
        folder = evidence / (Path(linker).name + '-' + name)
        folder.mkdir()
        src = folder / 'input.c'
        src.write_text(source)
        obj = folder / 'input.o'
        run([new_bin / 'libc-cc', '-c', src, '-o', obj], new_env)
        (folder / 'input.readelf.txt').write_text(run(['readelf', '-Wr', '-Ws', obj]))
        results = {}
        for variant, binary, env in [('before', old_bin, old_env), ('after', new_bin, new_env)]:
            output = folder / (variant + '.elf')
            run([binary / 'libc-cc', obj, *args, '-o', output], env)
            results[variant] = inspect(output)
        (folder / 'comparison.json').write_text(json.dumps(results, indent=2))
        before, after = map(lambda k: set(map(tuple, results[k]['imports'])), ['before', 'after'])
        assert {x for x in before if x[0] not in clocks} == {x for x in after if x[0] not in clocks}, (name, results)
        for c in clocks:
            assert (c, 'GLIBC_2.17', 'libc.so.6') in after, (name, c, results)
            if name.startswith('mixed'):
                assert (c, 'GLIBC_2.2.5', 'librt.so.1') in after, (name, c, results)
        if name in ['no-as-needed', 'state-transition', 'nested-state', 'timer']:
            assert 'librt.so.1' in results['after']['needed'], (name, results)
        if name == 'timer':
            assert ('timer_create', 'GLIBC_2.3.3', 'librt.so.1') in after, results
        assert all(not v.startswith('GLIBC_') or tuple(map(int, v[6:].split('.'))) <= (2, 22) for _, v, _ in after), results
        rows.append({'name': name, 'linker': linker, 'results': results})
        print(f'PASS {Path(linker).name}: {name}', flush=True)

# Same helper paths, same descriptor, same target dir; only the installed tool
# and its generated target profile change. Never clean the target directory.
active = evidence / 'active-bin'
active.mkdir()
project = evidence / 'cargo-fixture'
(project / 'src').mkdir(parents=True)
(project / 'Cargo.toml').write_text('[package]\nname="clock-cargo-fixture"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
rust = '''#[link(name="rt")] unsafe extern "C" {
fn clock_gettime(c:i32,p:*mut i64)->i32;
fn clock_getres(c:i32,p:*mut i64)->i32;
fn clock_settime(c:i32,p:*const i64)->i32;
fn clock_nanosleep(c:i32,f:i32,p:*const i64,r:*mut i64)->i32;
fn clock_getcpuclockid(p:i32,c:*mut i32)->i32;
#[link_name="clock_gettime@GLIBC_2.2.5"] fn old_clock_gettime(c:i32,p:*mut i64)->i32;
}
#[inline(never)] unsafe fn retained_calls() {
let mut t=[0i64;2];let mut id=0;
unsafe {clock_gettime(0,t.as_mut_ptr());clock_getres(0,t.as_mut_ptr());clock_settime(0,t.as_ptr());
clock_nanosleep(0,0,t.as_ptr(),std::ptr::null_mut());clock_getcpuclockid(0,&mut id);old_clock_gettime(0,t.as_mut_ptr());}
}
fn main(){std::hint::black_box(retained_calls as unsafe fn());}
'''
(project / 'src/main.rs').write_text(rust)
upgrade = {}
for variant, binary in [('old', old_bin), ('new', new_bin)]:
    for helper in ['cargo-libc', 'libc-linker', 'libc-cc', 'libc-cxx', 'libc-runner']:
        shutil.copy2(binary / helper, active / helper)
    env, text = config(active, ['--provider', str(descriptor)])
    env.pop('CARGO_CFG_CARGO_LIBC_PROFILE')
    env['CARGO_TARGET_DIR'] = str(evidence / 'same-target-dir')
    env['PATH'] = str(active) + os.pathsep + env['PATH']
    # Shell exports and TOML both carry precisely the same private selection.
    shell = run([active / 'cargo-libc', 'env', '--glibc', '2.22', '--provider', descriptor, '--cc', '/usr/bin/clang-18', '--format', 'shell'])
    (evidence / (variant + '.sh')).write_text(shell)
    (evidence / (variant + '.toml')).write_text(text)
    shell_file = evidence / (variant + '.sh')
    roundtrip = run(['bash', '-c', 'source "$1"; python3 -c \'import json,os;print(json.dumps(dict(os.environ)))\'', 'clock-env', shell_file], base)
    exported = json.loads(roundtrip)
    for key, value in tomllib.loads(text)['env'].items():
        if key.startswith('CARGO_LIBC_CLOCK_'):
            assert exported[key] == value, key
    messages = run(['cargo', '--config', str(evidence / (variant + '.toml')), 'build', '--offline', '--target', 'x86_64-unknown-linux-gnu', '--message-format=json'], env, project)
    (evidence / (variant + '-cargo.jsonl')).write_text(messages)
    executable = evidence / 'same-target-dir/x86_64-unknown-linux-gnu/debug/clock-cargo-fixture'
    saved = evidence / (variant + '-cargo.elf')
    shutil.copy2(executable, saved)
    upgrade[variant] = inspect(saved)
    (evidence / 'upgrade.json').write_text(json.dumps(upgrade, indent=2))
    assert any(m.get('reason') == 'compiler-artifact' and m.get('target', {}).get('name') == 'clock-cargo-fixture' and not m['fresh'] for m in map(json.loads, messages.splitlines())), variant
for c in clocks:
    assert (c, 'GLIBC_2.2.5', 'librt.so.1') in upgrade['old']['imports'], upgrade
    assert (c, 'GLIBC_2.17', 'libc.so.6') in upgrade['new']['imports'], upgrade
assert ('clock_gettime', 'GLIBC_2.2.5', 'librt.so.1') in upgrade['new']['imports'], upgrade
assert {x for x in upgrade['old']['imports'] if x[0] not in clocks} == {x for x in upgrade['new']['imports'] if x[0] not in clocks}, upgrade
(evidence / 'upgrade.json').write_text(json.dumps(upgrade, indent=2))
(evidence / 'results.json').write_text(json.dumps({'native_cases': rows, 'upgrade': upgrade, 'limits': ['No clock-setting fixture executed', 'No tier 3 qualification', 'Container preparation not exercised by this script; selected provider ELF path is shared']}, indent=2))
print(f'PASS {len(rows)} C/ELF comparisons, Cargo mixed calls, same-directory upgrade and shell/TOML selection', flush=True)
