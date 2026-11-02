#!/usr/bin/env python3
"""Disposable native ELF experiment; not a product argument transformer."""
import json, os, pathlib, re, shlex, subprocess, sys, tempfile
bin_dir=pathlib.Path(sys.argv[1]).resolve(); evidence=pathlib.Path(sys.argv[2]);evidence.mkdir(parents=True,exist_ok=True)
clang='/usr/bin/clang-18'
def run(args, **kw):
 p=subprocess.run([str(x) for x in args],capture_output=True,text=True,timeout=180,**kw)
 if p.returncode:raise RuntimeError(f'{args}\n{p.stdout}\n{p.stderr}')
 return p
config=run([bin_dir/'cargo-libc','env','--glibc','2.22','--cc',clang,'--format','cargo'])
import tomllib
env=os.environ.copy();env.update(tomllib.loads(config.stdout)['env']);env['CARGO_CFG_CARGO_LIBC_PROFILE']='prototype'
provider={}
for line in env['CARGO_LIBC_PROVIDER_DESCRIPTOR'].splitlines()[1:]:
 k,v=line.split('\t');provider[k]=v
stubs=pathlib.Path(provider['stubs'])
root=pathlib.Path(tempfile.mkdtemp(prefix='clock-order-prototype-'))
proxy=root/'trace-clang'
proxy.write_text('#!/bin/sh\nexec /usr/bin/clang-18 "$@" -###\n');proxy.chmod(0o755)
trace_env=env.copy();trace_env['CARGO_LIBC_CC']=str(proxy)
# Take addresses to retain all references; never call clock_settime.
source='''#define _GNU_SOURCE
#include <time.h>
#include <sys/types.h>
#include <mqueue.h>
#include <aio.h>
#include <stdint.h>
void *volatile refs[] = {(void *)clock_gettime,(void *)clock_getres,(void *)clock_settime,(void *)clock_nanosleep,(void *)clock_getcpuclockid
EXTRA
};
int main(void) { return refs[0] == 0; }
'''
clocks={'clock_gettime','clock_getres','clock_settime','clock_nanosleep','clock_getcpuclockid'}
def symbols(path):
 versions={};file=None
 for line in run(['readelf','--version-info','--wide',path]).stdout.splitlines():
  match=re.search(r'File: (\S+)',line)
  if match:file=match[1]
  match=re.search(r'Name: (\S+).*Version: (\d+)',line)
  if match and file:versions[int(match[2])]=[file,match[1]]
 result={}
 for line in run(['readelf','--dyn-syms','--wide',path]).stdout.splitlines():
  match=re.search(r'\bUND\s+(\S+)\s+\((\d+)\)',line)
  if match:
   name=match[1].split('@',1)[0];result.setdefault(name,[]).append(versions[int(match[2])])
 return result
rows=[]
for name,extra,options,unit in [
 ('all-five','', ['-Wl,--as-needed','-lrt','-lpthread','-lm','-ldl','-lc'],False),
 ('all-five-no-as-needed','',['-Wl,--no-as-needed','-lrt','-lpthread','-lm','-ldl','-lc'],False),
 ('as-needed-transition','',['-Wl,--no-as-needed','-lrt','-Wl,--as-needed','-lc'],False),
 ('timer-api',', (void *)timer_create, (void *)timer_delete',['-Wl,--as-needed','-lrt','-lc'],False),
 ('mq-api',', (void *)mq_getattr',['-Wl,--as-needed','-lrt','-lc'],False),
 ('aio-api',', (void *)aio_error',['-Wl,--as-needed','-lrt','-lc'],False),
 ('already-libc-first','',['-lc','-lrt'],False),
 ('group','',['-Wl,--start-group','-lrt','-lc','-Wl,--end-group'],False),
 ('push-pop-rt','',['-Wl,--push-state,--as-needed','-lrt','-Wl,--pop-state','-lc'],True),
 ('xlinker','',['-Xlinker','-lrt','-lc'],False),
 ('wl','',['-Wl,-lrt','-lc'],False),
 ('response','',['-lrt','-lc'],False),
 ('fullpath','',[str(stubs/'librt.so.1'),'-lc'],False),
 ('implicit-provider-c','',['-lrt'],False),
 ('legacy-and-default',', (void *)legacy_clock',['-lrt','-lc'],False),
]:
 src=root/(name+'.c')
 text=source.replace('EXTRA',extra)
 if name=='legacy-and-default':text='extern int legacy_clock(int,void*); __asm__(".symver legacy_clock,clock_gettime@GLIBC_2.2.5");\n'+text
 src.write_text(text);obj=root/(name+'.o')
 run([bin_dir/'libc-cc','-c',src,'-o',obj],env=env)
 driver_args=[str(obj),*options,'-o',str(root/(name+'-before'))]
 if name=='response':
  rsp=root/'caller.rsp';rsp.write_text(' '.join(shlex.quote(x) for x in driver_args));driver_args=['@'+str(rsp)]
 trace=run([bin_dir/'libc-cc',*driver_args],env=trace_env)
 (evidence/(name+'-clang-trace.txt')).write_text(trace.stderr)
 commands=[shlex.split(line) for line in trace.stderr.splitlines() if line.lstrip().startswith('"')]
 ld=next(args for args in commands if pathlib.Path(args[0]).name.endswith(('ld','ld.bfd','ld.gold','ld.lld')))
 before=root/(name+'-before');after=root/(name+'-after')
 # Compare the actual Clang-produced ld command, including implicit/default libc.
 run(ld)
 old=symbols(before)
 new=ld.copy();rt=str(stubs/'librt.so.1') if name=='fullpath' else '-lrt'
 pos=new.index(rt);anchor=new.index('-lc')
 if pos<anchor:
  moving=[new.pop(pos)]
  if unit or name=='as-needed-transition':
   state='--no-as-needed' if name=='as-needed-transition' else '--as-needed'
   moving=['--push-state',state,*moving,'--pop-state']
  anchor=new.index('-lc');new[anchor+1:anchor+1]=moving
 new[new.index('-o')+1]=str(after)
 run(new);now=symbols(after)
 assert {k:v for k,v in old.items() if k not in clocks}=={k:v for k,v in now.items() if k not in clocks},name
 for symbol in clocks:
  ordinary=[v for v in now[symbol] if v[1]!='GLIBC_2.2.5']
  assert ordinary and all(v[0]=='libc.so.6' for v in ordinary),(name,symbol,now[symbol])
 if name=='legacy-and-default':assert ['librt.so.1','GLIBC_2.2.5'] in now['clock_gettime'],now
 needed=lambda path: re.findall(r'\(NEEDED\).*Shared library: \[(.*?)\]',run(['readelf','--dynamic','--wide',path]).stdout)
 if name in ['as-needed-transition','all-five-no-as-needed']:assert 'librt.so.1' in needed(after),name
 row={'needed_before':needed(before),'needed_after':needed(after),'name':name,'before':old,'after':now,'before_ld':ld,'after_ld':new}
 rows.append(row);print(name+': all five libc; other versioned bindings unchanged',flush=True)
(evidence/'prototype.json').write_text(json.dumps({'consumer':run(['git','rev-parse','HEAD']).stdout.strip(),'environment':run(['uname','-a']).stdout.strip(),'clang':run([clang,'--version']).stdout.splitlines()[0],'provider':provider,'cases':rows},indent=2)+'\n')
print('Prototype passed',len(rows),'real ELF link comparisons; no target clock setters executed.')
