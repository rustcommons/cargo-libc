#!/usr/bin/env python3
"""Combined source-candidate regression; needs libc-abi-tools' provider IO fix."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())


def run(cache, closed=False, sink=None):
    args = [binary, "provider", "list", "--cache", str(cache)]
    if closed:
        reader, writer = os.pipe()
        os.close(reader)
        try:
            return subprocess.run(args, stdout=writer, stderr=subprocess.PIPE)
        finally:
            os.close(writer)
    return subprocess.run(args, stdout=sink or subprocess.PIPE, stderr=subprocess.PIPE)


with tempfile.TemporaryDirectory(prefix="cargo-libc-provider-stdout-") as temp:
    cache = Path(temp) / "cache"
    cache.mkdir()
    normal = run(cache)
    assert normal.returncode == 0 and normal.stdout.startswith(b"glibc\t"), normal
    assert not normal.stderr, normal.stderr
    closed = run(cache, closed=True)
    assert closed.returncode == 0 and not closed.stderr, closed

    # list writes its heading before discovering this invalid cache. Closing
    # stdout must not skip that later error or turn the command into success.
    bad_cache = Path(temp) / "not-a-directory"
    bad_cache.write_text("not a cache")
    normal = run(bad_cache)
    closed = run(bad_cache, closed=True)
    assert normal.returncode == closed.returncode == 1, (normal, closed)
    assert normal.stderr == closed.stderr and b"panicked" not in closed.stderr
    with open("/dev/full", "wb", buffering=0) as full:
        failed = run(cache, sink=full)
    assert failed.returncode == 1 and failed.stderr, failed
    assert b"panicked" not in failed.stderr, failed.stderr
print("Provider stdout: normal output, closed reader, later failure and non-pipe error passed.")
