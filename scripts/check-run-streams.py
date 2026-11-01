#!/usr/bin/env python3
"""Native run/test stream regression. Usage: TOOLS_DIR CLANG [--expect-pollution]."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

bin_dir = Path(sys.argv[1]).resolve()
clang = sys.argv[2]
baseline = sys.argv[3:] == ['--expect-pollution']
with tempfile.TemporaryDirectory(prefix='cargo-libc-streams-') as tmp:
    root = Path(tmp)
    (root / 'src').mkdir()
    (root / 'Cargo.toml').write_text('''[package]
name = "stream-probe"
version = "0.0.0"
edition = "2021"
[[test]]
name = "stream-test"
path = "src/main.rs"
harness = false
''')
    (root / 'src/main.rs').write_text(r'''
use std::io::{self, Read, Write};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    assert_eq!(input, "redirected input\n");
    eprintln!("child stderr marker");
    if std::env::args().any(|arg| arg == "env-check") {
        assert_eq!(std::env::var("CARGO_F2_TEST").unwrap(), "cargo value with spaces");
        assert_eq!(std::env::var("RUST_F2_TEST").unwrap(), "rust value with spaces");
        assert!(std::env::var("UNRELATED_F2_TEST").is_err());
    }
    if let Err(e) = io::stdout().write_all(b"{\"ok\":true}\n") {
        if e.kind() != io::ErrorKind::BrokenPipe { std::process::exit(23); }
    }
    if std::env::args().any(|arg| arg == "fail") { std::process::exit(7); }
}
''')
    subprocess.run(['cargo', 'generate-lockfile', '--offline', '--manifest-path',
                    str(root / 'Cargo.toml')], check=True)
    env = os.environ.copy()
    env['PATH'] = str(bin_dir) + os.pathsep + env['PATH']
    env.setdefault('CARGO_LIBC_PROVIDER_DIR', str(root / 'providers'))

    def invoke(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE):
        return subprocess.run(args, input=b'redirected input\n', stdout=stdout,
                              stderr=stderr, env=env, check=False, timeout=180)

    expected = b'{"ok":true}\n'
    for command in ['run', 'test']:
        args = [str(bin_dir / 'cargo-libc'), command, '--target',
                'x86_64-unknown-linux-gnu.2.17', '--cc', clang, '--',
                '--manifest-path', str(root / 'Cargo.toml'), '--locked', '--offline']
        if command == 'test':
            args += ['--test', 'stream-test']
        normal = invoke(args)
        assert normal.returncode == 0, normal.stderr
        if baseline:
            assert b'Target:' in normal.stdout and normal.stdout.endswith(expected), normal
            print(f'{command}: reproduced setup diagnostics before program JSON')
            continue
        assert normal.stdout == expected, normal.stdout
        assert b'Target:' in normal.stderr and b'Cargo command:' in normal.stderr, normal.stderr
        assert b'child stderr marker' in normal.stderr, normal.stderr
        failure = invoke(args + ['--', 'fail'])
        # The harness-free test, like run, preserves its exit status.
        expected_failure = 7
        assert failure.returncode == expected_failure, failure
        assert failure.stdout == expected, failure.stdout
        # Preclosed reader: deterministic, independent of scheduler/head timing.
        reader, writer = os.pipe()
        os.close(reader)
        try:
            assert invoke(args, stdout=writer).returncode == 0
            assert invoke(args + ['--', 'fail'], stdout=writer).returncode == expected_failure
        finally:
            os.close(writer)
        with open('/dev/full', 'wb') as full:
            assert invoke(args, stdout=full).returncode == 23
            diagnostic_error = invoke(args, stderr=full)
            assert diagnostic_error.returncode == 1, diagnostic_error
        dry_run = invoke(args[:args.index('--')] + ['--dry-run'])
        assert dry_run.returncode == 0 and dry_run.stdout == b'', dry_run
        assert b'Cargo command:' in dry_run.stderr, dry_run.stderr
        local_args = args[:args.index('--')] + ['--kernel', '3.10', '--runner', 'local'] + args[args.index('--'):]
        local = invoke(local_args)
        assert local.returncode == 0 and local.stdout == expected, local
        assert b'local seccomp filter' in local.stderr, local.stderr
        print(f'{command}: exact JSON stdout, separate diagnostics, stdin and failure statuses passed')
    if not baseline:
        # Exercise the installed target runner directly, with inherited input/output.
        runner = [str(bin_dir / 'libc-runner'),
                  str(root / 'target/x86_64-unknown-linux-gnu/debug/stream-probe')]
        # Use the same runner environment that cargo libc env exports.
        config = subprocess.run([str(bin_dir / 'cargo-libc'), 'env', '--target',
                                 'x86_64-unknown-linux-gnu.2.17', '--cc', clang,
                                 '--format', 'cargo'], capture_output=True, env=env, check=True)
        import tomllib
        cargo_env = env.copy()
        env.update(tomllib.loads(config.stdout.decode())['env'])
        result = invoke(runner)
        assert result.returncode == 0 and result.stdout == expected, result
        assert invoke(runner + ['fail']).returncode == 7
        print('libc-runner: redirected input/output and exit status passed')
        env = cargo_env  # Plain-runner target settings must not leak into managed Cargo.

        if os.environ.get('CHECK_STREAM_CONTAINERS'):
            env.update(CARGO_F2_TEST='cargo value with spaces', RUST_F2_TEST='rust value with spaces',
                       UNRELATED_F2_TEST='must not be forwarded')
            for engine in ['docker', 'podman']:
                env['CONTAINER_ENGINE'] = engine
                env['KERNEL_ABI_ENGINE_ARGS'] = ('--runtime ' + os.environ['CHECK_STREAM_CONTAINERS']) if engine == 'podman' else ''
                env['KERNEL_ABI_RUN_ARGS'] = '--network none'
                for command in ['run', 'test']:
                    args = [str(bin_dir / 'cargo-libc'), command, '--target',
                            'x86_64-unknown-linux-gnu.2.17', '--cc', clang,
                            '--kernel', '4.12', '--runner', 'container', '--image',
                            'registry.suse.com/suse/sles12sp5:latest', '--',
                            '--manifest-path', str(root / 'Cargo.toml'), '--locked', '--offline']
                    if command == 'test':
                        args += ['--test', 'stream-test']
                    result = invoke(args + ['--', 'env-check'])
                    assert result.returncode == 0 and result.stdout == expected, result
                    failure = invoke(args + ['--', 'env-check', 'fail'])
                    assert failure.returncode == 7 and failure.stdout == expected, failure
                    print(f'{engine} {command}: redirected stdin, exact JSON stdout, spaced environment and exit 7 passed')
