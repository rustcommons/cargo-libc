#!/usr/bin/env python3
"""Bounded static ELF comparison regressions; fixtures are linked, never run.

Usage: python3 scripts/check-target-libgcc.py /path/to/cargo-libc EVIDENCE_DIR
Requires an x86-64 Linux GCC/linker. No runtime/toolchain installation occurs.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys

binary = str(Path(sys.argv[1]).resolve())
out = Path(sys.argv[2]).resolve()
out.mkdir(exist_ok=False, parents=True)
results = []

def command(args):
    p = subprocess.run(args, capture_output=True, text=True, timeout=30)
    if p.returncode:
        raise RuntimeError(f"fixture link failed: {args!r}\n{p.stderr}")

def library(name, version="GCC_4.2.0", symbol="gcc_api", soname="libgcc_s.so.1", extra=(), source=None, linker="bfd", style="both"):
    c = out / (name + ".c")
    c.write_text(source or f"int {symbol}(void) {{ return 7; }}\n")
    script = out / (name + ".map")
    script.write_text(f"{version} {{ global: {symbol}; local: *; }};\n")
    target = out / (name + ".so")
    command(["gcc", "-shared", "-fPIC", "-nostdlib", "-fuse-ld=" + linker, str(c), "-o", str(target),
             "-Wl,--hash-style=" + style, "-Wl,-soname," + soname, "-Wl,--version-script=" + str(script), *extra])
    return target

def compare(name, app, target, code, status, extra=()):
    p = subprocess.run([binary, "inspect", str(app), "--target-libgcc", str(target), "--json", *extra], capture_output=True, text=True, timeout=15)
    (out / (name + ".stdout.json")).write_text(p.stdout)
    (out / (name + ".stderr")).write_text(p.stderr)
    assert p.returncode == code, (name, p.returncode, p.stderr, p.stdout[:1500])
    value = json.loads(p.stdout)
    gcc = value["gcc_abi_comparison"]
    assert gcc["status"] == status, (name, gcc)
    for field in ["runtime_tested", "transitive_dependencies_checked", "selected_by_target_loader_verified", "runtime_implementation_authenticated"]:
        assert gcc[field] is False, (name, field)
    if Path(target).is_file():
        assert gcc["target_identity"]["sha256"] == hashlib.sha256(Path(target).read_bytes()).hexdigest()
    results.append(dict(name=name, exit=p.returncode, status=status))
    return value

def tables(blob):
    phoff = struct.unpack_from("<Q", blob, 32)[0]
    size, count = struct.unpack_from("<HH", blob, 54)
    loads = []
    dynamic = None
    for i in range(count):
        off = phoff + i * size
        kind = struct.unpack_from("<I", blob, off)[0]
        f, v, _, n = struct.unpack_from("<QQQQ", blob, off + 8)
        if kind == 1: loads.append((f, v, n))
        if kind == 2: dynamic = (f, n)
    def mapped(a):
        return next(f + a - v for f, v, n in loads if v <= a < v+n)
    tags = {}
    for off in range(dynamic[0], sum(dynamic), 16):
        t, v = struct.unpack_from("<QQ", blob, off)
        if t == 0: break
        tags[t] = (off, v)
    return mapped, tags

good = library("good")
app_c = out / "app.c"
app_c.write_text("extern int gcc_api(void); int use_api(void) { return gcc_api(); }\n")
app = out / "app.so"
command(["gcc", "-shared", "-fPIC", "-nostdlib", str(app_c), "-o", str(app), "-Wl,--hash-style=both", str(good)])
compare("exact", app, good, 0, "match")
compare("wrong_version", app, library("newer", "GCC_99.0"), 2, "missing")
compare("right_node_wrong_symbol", app, library("wrong_name", symbol="other_api"), 2, "missing")
compare("wrong_soname", app, library("wrong_owner", soname="libother.so"), 2, "missing")
compare("missing_file", app, out / "absent.so", 3, "unresolved")
script = out / "script.so"; script.write_text("INPUT(/do/not/follow/libgcc_s.so.1)\n")
compare("linker_script", app, script, 3, "unresolved")
for label, index, value in [("wrong_machine",18,183),("wrong_class",4,1),("wrong_endian",5,2)]:
    data = bytearray(good.read_bytes())
    if index == 18:struct.pack_into("<H", data, index, value)
    else:data[index] = value
    target = out / (label+".so");target.write_bytes(data)
    compare(label, app, target, 2, "missing")
for style in ["sysv", "gnu"]:
    target = library(style, style=style)
    compare("hash_"+style, app, target, 0, "match")
    data=bytearray(target.read_bytes());struct.pack_into("<Q",data,40,0);struct.pack_into("<HHH",data,58,0,0,0)
    target=out/(style+"-sectionless.so");target.write_bytes(data)
    compare("sectionless_"+style, app, target, 0, "match")
if shutil.which("ld.lld"):
    compare("lld_runtime",app,library("lld",linker="lld"),0,"match")
nondefault=library("nondefault", source='int impl(void) { return 7; }\n__asm__(".symver impl,gcc_api@GCC_4.2.0");\n')
compare("nondefault_export",app,nondefault,0,"match")
# Optional exact extraction must not erase valid legacy metadata.
data=bytearray(app.read_bytes());mapped,tags=tables(data)
for tag in [4,0x6ffffef5]:
    if tag in tags:struct.pack_into("<Q",data,tags[tag][0],21) # ignored DT_DEBUG, no count evidence
unsupported=out/"no-supported-hash.so";unsupported.write_bytes(data)
compare("optional_parser_failure",unsupported,good,3,"unresolved")
compare("independent_bad_target",unsupported,library("another_wrong",soname="other.so"),2,"missing")
# Even sectionless input cannot hide a relocated import using forged counts.
data=bytearray(app.read_bytes());mapped,tags=tables(data)
sysv=mapped(tags[4][1]);nb=struct.unpack_from("<I",data,sysv)[0]
struct.pack_into("<I",data,sysv+4,1)
for i in range(nb):struct.pack_into("<I",data,sysv+8+i*4,0)
gnu=mapped(tags[0x6ffffef5][1]);nb,_,bloom,_=struct.unpack_from("<IIII",data,gnu)
struct.pack_into("<I",data,gnu+4,1)
for i in range(nb):struct.pack_into("<I",data,gnu+16+bloom*8+i*4,0)
struct.pack_into("<Q",data,40,0);struct.pack_into("<HHH",data,58,0,0,0)
forged=out/"forged-count.so";forged.write_bytes(data)
v=compare("forged_hash_cannot_hide_import",forged,good,3,"unresolved")
assert "outside the declared hash inventory" in str(v["gcc_abi_comparison"]["findings"])
data=bytearray(good.read_bytes());mapped,tags=tables(data)
struct.pack_into("<I",data,mapped(tags[0x6ffffffc][1])+8,0)
corrupt=out/"corrupt-version-hash.so";corrupt.write_bytes(data)
compare("corrupt_version_hash",app,corrupt,3,"unresolved")
# Required VERNEED nodes cannot be erased just because their dynsym references
# are weak or absent. Use the actual ELF joins to patch only binding/index bytes.
data=bytearray(app.read_bytes());mapped,tags=tables(data)
strings=mapped(tags[5][1]);sym=mapped(tags[6][1]);versym=mapped(tags[0x6ffffff0][1]);count=struct.unpack_from("<I",data,mapped(tags[4][1])+4)[0]
api_index=None
for i in range(count):
    name=struct.unpack_from("<I",data,sym+i*24)[0];end=data.index(0,strings+name)
    if data[strings+name:end]==b"gcc_api":api_index=i
assert api_index is not None
data[sym+api_index*24+4]=0x22
weak=out/"weak-import.so";weak.write_bytes(data)
compare("weak_import_missing_node",weak,library("weak_missing_node",version="GCC_3.0"),2,"missing")
compare("weak_import_optional_absence",weak,library("weak_missing_symbol",symbol="other"),0,"match")
data[sym+api_index*24+4]=0x02;struct.pack_into("<H",data,versym+api_index*2,1)
zero=out/"zero-reference.so";zero.write_bytes(data)
compare("zero_reference_node_missing",zero,out/"weak_missing_node.so",2,"missing")
compare("zero_reference_node_present",zero,good,0,"match")
# Runtime's GLIBC floor is independent of its matching GCC interfaces.
fake = library("fake_libc",version="GLIBC_2.34",symbol="new_glibc_api",soname="libc.so.6")
new_glibc=library("new_glibc",source="extern int new_glibc_api(void); int gcc_api(void) { return new_glibc_api(); }\n",extra=(str(fake),))
v=compare("candidate_glibc_floor",app,new_glibc,2,"match",["--max-glibc","2.22"])
assert v["gcc_abi_comparison"]["target_runtime_metadata"]["direct_glibc_requirement_check"]=="fail"
compare("independent_target_floor_despite_optional_failure",unsupported,new_glibc,2,"unresolved",["--max-glibc","2.22"])
# Version nodes are independently validated before optional symbol enumeration.
missing_node=library("missing_node",version="GCC_3.0")
v=compare("node_failure_despite_artifact_symbol_failure",unsupported,missing_node,2,"missing")
assert v["gcc_abi_comparison"]["version_enumeration_complete"] is True
assert v["gcc_abi_comparison"]["enumeration_complete"] is False
assert v["gcc_abi_comparison"]["version_requirements"][0]["status"]=="missing"
data=bytearray(missing_node.read_bytes());mapped,tags=tables(data)
for tag in [4,0x6ffffef5]:
    if tag in tags:struct.pack_into("<Q",data,tags[tag][0],21)
missing_bad_hash=out/"missing-node-no-hash.so";missing_bad_hash.write_bytes(data)
v=compare("node_failure_despite_target_symbol_failure",app,missing_bad_hash,2,"missing")
assert v["gcc_abi_comparison"]["version_enumeration_complete"] is True
assert v["gcc_abi_comparison"]["enumeration_complete"] is False
# No DT_VERDEF at all is distinct from a versioned runtime lacking a node:
# unmodeled versionless lookup fallback is unresolved, not proven failure.
versionless_c=out/"versionless.c";versionless_c.write_text("int gcc_api(void) { return 7; }\n")
versionless=out/"versionless.so"
command(["gcc","-shared","-fPIC","-nostdlib","-fuse-ld=bfd",str(versionless_c),"-o",str(versionless),"-Wl,--hash-style=both","-Wl,-soname,libgcc_s.so.1"])
compare("wholly_versionless_fallback",app,versionless,3,"unresolved")
compare("versionless_weak_import_node",weak,versionless,3,"unresolved")
compare("versionless_zero_reference_node",zero,versionless,3,"unresolved")
# Corrupt version metadata cannot establish an independently missing node.
data=bytearray(missing_node.read_bytes());mapped,tags=tables(data)
struct.pack_into("<I",data,mapped(tags[0x6ffffffc][1])+8,0)
bad_nodes=out/"bad-node-hash.so";bad_nodes.write_bytes(data)
v=compare("invalid_node_metadata_not_absence",app,bad_nodes,3,"unresolved")
assert v["gcc_abi_comparison"]["version_enumeration_complete"] is False
# Non-UTF-8 filenames and explicit symlinks refer to the same inspected bytes.
if os.name == "posix":
    native=out/os.fsdecode(b"runtime-\xff.so");native.write_bytes(good.read_bytes())
    v=compare("non_utf8",app,native,0,"match")
    assert v["gcc_abi_comparison"]["target_identity"]["path_display_lossy"] is True
    fifo=out/"fifo";os.mkfifo(fifo)
    compare("fifo_refused",app,fifo,3,"unresolved")
    primary_fifo=subprocess.run([binary,"inspect",str(fifo),"--target-libgcc",str(good),"--json"],capture_output=True,text=True,timeout=5)
    assert primary_fifo.returncode==1 and not primary_fifo.stdout and "regular file" in primary_fifo.stderr
    results.append(dict(name="primary_fifo_refused",exit=1,status="primary_error"))
alias=out/"alias.so";alias.symlink_to(good)
compare("symlink",app,alias,0,"match")
# Legacy JSON fields survive optional extraction failures, without a new field
# unless explicitly requested. An optional baseline binary may be supplied.
legacy = subprocess.run([binary, "inspect", str(app), "--json"],capture_output=True,text=True,timeout=15)
assert legacy.returncode == 0
normal = json.loads(legacy.stdout)
assert "gcc_abi_comparison" not in normal
compared = json.loads((out/"exact.stdout.json").read_text())
compared.pop("gcc_abi_comparison")
assert normal == compared
if len(sys.argv) > 3:
    for mode in [[],["--json"]]:
        old = subprocess.run([sys.argv[3],"inspect",str(app),*mode],capture_output=True,timeout=15)
        new = subprocess.run([binary,"inspect",str(app),*mode],capture_output=True,timeout=15)
        assert (old.returncode,old.stdout,old.stderr)==(new.returncode,new.stdout,new.stderr)
results.append(dict(name="legacy_report_preserved",exit=0,status="unchanged"))
# Actual GNU Rust binary/runtime positive control, without executing either file.
runtime=Path('/lib/x86_64-linux-gnu/libgcc_s.so.1')
if runtime.is_file():
    compare("native_rust_binary",Path(binary),runtime,0,"match")
(out/"results.json").write_text(json.dumps(results,indent=2)+"\n")
print(f"PASS: {len(results)} static comparison cases; no fixture was executed")
