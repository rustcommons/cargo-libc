#!/usr/bin/env python3
"""run.py LLVM_ARCHIVE_OR_- NEW_OUTPUT_DIR [local|global]
'-' runs baseline only. All execution is native and externally timeout-bounded.
"""
import hashlib, json, os, pathlib, shutil, subprocess, sys
src = pathlib.Path(__file__).resolve().parent
if not 3 <= len(sys.argv) <= 4: sys.exit(__doc__)
archive_arg, out_arg = sys.argv[1:3]
scope = sys.argv[3] if len(sys.argv) == 4 else 'local'
if scope not in ('local','global'): sys.exit('scope must be local or global')
out = pathlib.Path(out_arg).resolve()
out.mkdir(parents=True, exist_ok=False)
env = dict(os.environ)
# Do not inherit preload/library search injection into this diagnostic process.
for key in ('LD_PRELOAD','LD_LIBRARY_PATH','LD_AUDIT','LD_DEBUG','LD_DEBUG_OUTPUT'):
    env.pop(key, None)
rustc = env.get('PROBE_RUSTC', 'rustc')
wrapper = str(src/'link.py')
common = ['--edition=2021','-C','panic=unwind','-C','opt-level=0',
          '-C','debuginfo=2','-C','codegen-units=1','-C','force-unwind-tables=yes',
          '-C','linker-flavor=gcc','-C','linker='+wrapper]
results = []
def capture(command, file, use_env=env, seconds=120):
    with (out/file).open('wb') as stream:
        run = subprocess.run(['timeout','--kill-after=3s',str(seconds)+'s']+command,
                             env=use_env,stdout=stream,stderr=subprocess.STDOUT)
    return run.returncode
def build(name, source, mode, shared=False):
    build_env = dict(env, PROBE_MODE=mode, PROBE_MAP=str(out/(name+'.map')),
                     PROBE_TRACE=str(out/(name+'.link.json')))
    command=[rustc,str(src/source),'--crate-name',name]+common
    if shared: command += ['--crate-type=cdylib']
    command += ['-o',str(out/name)]
    rc=capture(command,name+'.build.log',build_env)
    if rc: sys.exit(f'build failed {name}: {rc}; see log (no fallback)')
    for suffix, flags in [('dynamic',['-dW']),('symbols',['-sW']),('versions',['-VW']),
                          ('relocs',['-rW']),('headers',['-hlW'])]:
        if capture(['readelf']+flags+[str(out/name)],name+'.'+suffix+'.txt'):
            sys.exit('readelf failed')
    return out/name
for name,command in [('rustc',[rustc,'-vV']),('cc',[env.get('PROBE_CC','cc'),'--version']),
                     ('ld',['ld','--version']),('uname',['uname','-a'])]:
    if capture(command,'tool-'+name+'.txt'): sys.exit('tool record failed: '+name)
(out/'inputs.json').write_text(json.dumps({
    'source_sha256': {p.name:hashlib.sha256(p.read_bytes()).hexdigest()
                      for p in src.iterdir() if p.is_file()},
    'scope':scope,'linker':env.get('PROBE_LINKER','gnu'),
    'archive_input':archive_arg},indent=2)+'\n')
modes=['gcc']
if archive_arg != '-':
    archive=pathlib.Path(archive_arg).resolve(strict=True)
    staged=out/'llvm-unwind-probe.a'
    shutil.copyfile(archive,staged)
    env['PROBE_ARCHIVE']=str(staged)
    (out/'archive.sha256').write_text(hashlib.sha256(staged.read_bytes()).hexdigest()+'\n')
    if capture(['ar','t',str(staged)],'archive.members.txt'): sys.exit('invalid archive')
    modes += ['hidden','exported']
dso=build('bridge_dso','bridge.rs','gcc',True)
if 'libgcc_s.so.1' not in (out/'bridge_dso.dynamic.txt').read_text():
    sys.exit('REFUSE: DSO is not expected dynamic GCC control')
for mode in modes:
    host=build('host_'+mode,'host.rs',mode)
    direct_gcc='libgcc_s.so.1' in (out/(host.name+'.dynamic.txt')).read_text()
    results.append({'kind':'link','mode':mode,'direct_libgcc_s':direct_gcc,
                    'pass':direct_gcc == (mode=='gcc')})
    for case in ('u07','u10','u11'):
        name=mode+'-'+case
        runenv=dict(env, LD_DEBUG='libs,versions,bindings')
        rc=capture([str(host),str(dso),case,scope],name+'.loader.log',runenv,20)
        (out/(name+'.exit')).write_text(str(rc)+'\n')
        # A zero exit alone is not enough: assert completed counter checks.
        log=(out/(name+'.loader.log')).read_text(errors='replace')
        passed=rc==0 and ('PASS '+case+' '+scope+' ') in log
        results.append({'kind':'runtime','mode':mode,'case':case,'exit':rc,'pass':passed})
        selected=[line for line in log.splitlines() if any(s in line for s in
                  ('_Unwind_', 'rust_eh_personality', 'libgcc_s', 'libunwind', 'PASS ', 'panicked'))]
        (out/(name+'.bindings.txt')).write_text('\n'.join(selected)+'\n')
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
# Binding graph and actual hidden/exported visibility require independent review.
sys.exit(0 if all(r['pass'] for r in results) else 1)
