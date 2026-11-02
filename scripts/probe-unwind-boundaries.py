#!/usr/bin/env python3
"""Native baseline and symbol-interposition probes, NOT LLVM qualification.

Uses existing native cc/ar/readelf and stable rustc. No downloads or installations.
All commands/results survive failures. Run on the intended native host as well;
a passing aarch64 host probe does not qualify cargo-libc's x86-64 target policy.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--evidence', type=Path, required=True)
p.add_argument('--cc', default='cc')
p.add_argument('--rustc', default='rustc')
a = p.parse_args()
a.evidence.mkdir(parents=True, exist_ok=False)
e = a.evidence.resolve()
commands, cases = [], []
completed = False


def run(argv, name, expected=0, env=None):
    result = subprocess.run([str(x) for x in argv], cwd=e, capture_output=True,
                            timeout=90, env=env)
    (e / (name + '.stdout')).write_bytes(result.stdout)
    (e / (name + '.stderr')).write_bytes(result.stderr)
    commands.append({'name': name, 'argv': [str(x) for x in argv],
                     'exit': result.returncode, 'expected': expected})
    (e / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
    if result.returncode != expected:
        raise RuntimeError(f'{name}: exit {result.returncode}, expected {expected}; see saved stderr')
    return result.stdout.decode(errors='replace')


def elf(binary):
    run(['readelf', '-W', '-h', '-d', '-V', '--dyn-syms', binary], binary + '.readelf')


try:
    run([a.cc, '--version'], 'cc-version')
    run(['ar', '--version'], 'ar-version')
    run([a.rustc, '-vV'], 'rustc-version')
    (e / 'context.json').write_text(json.dumps({
        'machine': platform.machine(), 'platform': platform.platform(),
        'repository_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
        'driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'worktree_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'])),
        'scope': 'Native baseline + synthetic archive export/interposition; no LLVM runtime selected',
    }, indent=2) + '\n')
    wrapper = e / 'record-linker.py'
    wrapper.write_text('#!/usr/bin/env python3\nimport json, pathlib, subprocess, sys\n'
                       'root = pathlib.Path(__file__).parent\n'
                       'with (root / "rust-link-commands.jsonl").open("a") as f:\n'
                       '    f.write(json.dumps(sys.argv[1:]) + "\\n")\n'
                       f'sys.exit(subprocess.call([{a.cc!r}, *sys.argv[1:]]))\n')
    wrapper.chmod(0o755)
    # Deliberately synthetic marker, not fake _Unwind implementations. This
    # isolates ELF interposition from real context/personality compatibility.
    (e / 'marker.c').write_text('int runtime_marker(void) { return 7; }\n')
    (e / 'dso.c').write_text('int runtime_marker(void) { return 11; }\n'
                             'int dso_marker(void) { return runtime_marker(); }\n')
    (e / 'app.c').write_text('#include <stdio.h>\nint runtime_marker(void); int dso_marker(void);\n'
                             'int main(void) { printf("exec=%d dso=%d\\n", runtime_marker(), dso_marker()); }\n')
    run([a.cc, '-fPIC', '-c', 'marker.c', '-o', 'marker.o'], 'marker-object')
    run(['ar', 'rcs', 'libprobe_marker.a', 'marker.o'], 'marker-archive')
    run([a.cc, '-fPIC', '-shared', 'dso.c', '-o', 'libprobe_dso.so'], 'marker-dso')
    for name, flags, expected in [
        ('exported-static', ['-Wl,--export-dynamic'], 'exec=7 dso=7\n'),
        ('hidden-static', ['-Wl,--export-dynamic,--exclude-libs,libprobe_marker.a'], 'exec=7 dso=11\n'),
        ('hidden-no-as-needed', ['-Wl,--no-as-needed,--export-dynamic,--exclude-libs,libprobe_marker.a'], 'exec=7 dso=11\n'),
    ]:
        run([a.cc, 'app.c', 'libprobe_marker.a', '-L.', '-lprobe_dso',
             '-Wl,-rpath,$ORIGIN', *flags, '-o', name], name + '-build')
        actual = run([e / name], name + '-run')
        elf(name)
        assert actual == expected, (name, actual, expected)
        cases.append({'case': name, 'stdout': actual, 'pass': True,
                      'scope': 'Synthetic marker ownership; not cross-DSO unwind ABI proof'})
    (e / 'panic_probe.rs').write_text(r'''
use std::{backtrace::{Backtrace, BacktraceStatus}, panic, sync::atomic::{AtomicUsize, Ordering}};
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct Guard;
impl Drop for Guard { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }
#[inline(never)] fn capture() -> Backtrace { Backtrace::force_capture() }
fn main() {
    let mode = std::env::args().nth(1).unwrap();
    match mode.as_str() {
        "plain" => (),
        "catch" => { assert!(panic::catch_unwind(|| { let _g = Guard; panic!("caught"); }).is_err()); assert_eq!(DROPS.load(Ordering::SeqCst), 1); }
        "nested" => { assert!(panic::catch_unwind(|| { let _outer = Guard; assert!(panic::catch_unwind(|| { let _inner = Guard; panic!("inner"); }).is_err()); panic!("outer"); }).is_err()); assert_eq!(DROPS.load(Ordering::SeqCst), 2); }
        "thread" => { assert!(std::thread::spawn(|| { let _g = Guard; panic!("thread"); }).join().is_err()); assert_eq!(DROPS.load(Ordering::SeqCst), 1); }
        "backtrace" => { let trace = capture(); assert_eq!(trace.status(), BacktraceStatus::Captured); eprintln!("{trace}"); }
        "uncaught" => panic!("uncaught"),
        _ => panic!("unknown probe mode"),
    }
    std::fs::write(std::env::var_os("PROBE_MAPS").unwrap(), std::fs::read("/proc/self/maps").unwrap()).unwrap();
    println!("pass {mode}");
}
'''.lstrip())
    run([a.rustc, 'panic_probe.rs', '-C', 'panic=unwind', '-C', f'linker={wrapper}', '-C', 'linker-flavor=gcc', '-g', '-o', 'panic-probe'], 'panic-build')
    elf('panic-probe')
    for mode in ['plain', 'catch', 'nested', 'thread', 'backtrace', 'uncaught']:
        env = dict(os.environ, PROBE_MAPS=str(e / (mode + '.maps')))
        actual = run([e / 'panic-probe', mode], 'panic-' + mode,
                     expected=101 if mode == 'uncaught' else 0, env=env)
        if mode != 'uncaught':
            assert actual == f'pass {mode}\n'
        else:
            assert actual == ''
        loaded = sorted({line.split()[-1] for line in (e / (mode + '.maps')).read_text().splitlines()
                         if len(line.split()) >= 6 and line.split()[-1].startswith('/')}) if mode != 'uncaught' else []
        cases.append({'case': 'panic-' + mode, 'exit': 101 if mode == 'uncaught' else 0,
                      'loaded_files': loaded, 'pass': True, 'scope': 'Stable native Rust baseline, not LLVM candidate'})
    (e / 'library.rs').write_text('pub fn library_value() -> i32 { 7 }\n')
    for kind in ['cdylib', 'dylib', 'rlib', 'staticlib']:
        run([a.rustc, 'library.rs', '--crate-type', kind, '-C', f'linker={wrapper}',
             '-C', 'linker-flavor=gcc', '-o', 'library-' + kind], 'rust-' + kind)
        if kind in ['cdylib', 'dylib']:
            elf('library-' + kind)
    run([a.rustc, 'library.rs', '--test', '-C', f'linker={wrapper}', '-C', 'linker-flavor=gcc',
         '-o', 'test-harness'], 'rust-test-harness')
    elf('test-harness')
    link_commands = [json.loads(line) for line in (e / 'rust-link-commands.jsonl').read_text().splitlines()]
    assert len(link_commands) == 4  # executable, two shared outputs, test; rlib never links.
    assert sum('-shared' in args for args in link_commands) == 2
    (e / 'output-kind-summary.json').write_text(json.dumps({
        'final_executables': 2, 'shared_outputs': 2, 'rlib_linker_calls': 0, 'staticlib_linker_calls': 0,
        'scope': 'Actual stable rustc driver commands; no LLVM policy injected'}, indent=2) + '\n')
    (e / 'native-cleanup.c').write_text('extern void may_unwind(void);\n'
        'static void cleanup(int *p) { (void)p; may_unwind(); }\n'
        'int native_cleanup(void) { int x __attribute__((cleanup(cleanup))) = 7; may_unwind(); return x; }\n')
    run([a.cc, '-fexceptions', '-fPIC', '-c', 'native-cleanup.c', '-o', 'native-cleanup.o'], 'native-cleanup-build')
    run(['readelf', '-W', '-s', '-r', 'native-cleanup.o'], 'native-cleanup-symbols')
    completed = True
finally:
    (e / 'results.json').write_text(json.dumps({'cases': cases,
        'completed': completed, 'failure': str(sys.exc_info()[1]) if sys.exc_info()[1] else None,
        'limits': 'No static LLVM archive, cargo-libc candidate, x86-64 target, C-unwind context crossing or PyO3 qualification is established by these host probes.'}, indent=2) + '\n')
    (e / 'files.sha256').write_text(''.join(f'{hashlib.sha256(f.read_bytes()).hexdigest()}  {f.name}\n'
                                         for f in sorted(e.iterdir()) if f.is_file() and f.name != 'files.sha256'))
print(f'Nine bounded native baseline/interposition probes passed; evidence: {e}')
