#!/usr/bin/env python3
"""Focused rustc linker probe. Not production linker policy."""
import json, os, pathlib, subprocess, sys
args = sys.argv[1:]
mode = os.environ['PROBE_MODE']
original = list(args)
if mode != 'gcc':
    count = args.count('-lgcc_s')
    if count != 1:
        sys.exit(f'REFUSE: expected exactly one rustc -lgcc_s; got {count}; inspect recorded toolchain')
    i = args.index('-lgcc_s')
    # Preserve compiler helpers explicitly; never replace them with unwind code.
    args[i:i+1] = [os.environ['PROBE_ARCHIVE'], '-lgcc']
    if mode == 'hidden':
        args += ['-Wl,--exclude-libs=llvm-unwind-probe.a']
    elif mode == 'exported':
        args += ['-Wl,--export-dynamic-symbol=_Unwind_*']
    else:
        sys.exit('unknown probe mode')
linker = os.environ.get('PROBE_LINKER', 'gnu')
if linker == 'lld': args += ['-fuse-ld=lld']
elif linker == 'gnu': args += ['-fuse-ld=bfd']
else: sys.exit('PROBE_LINKER must be gnu or lld')
args += ['-Wl,-Map=' + os.environ['PROBE_MAP']]
cc = os.environ.get('PROBE_CC', 'cc')
pathlib.Path(os.environ['PROBE_TRACE']).write_text(json.dumps({
    'mode':mode, 'cc':cc, 'original':original, 'effective':args}, indent=2)+'\n')
sys.exit(subprocess.call([cc] + args))
