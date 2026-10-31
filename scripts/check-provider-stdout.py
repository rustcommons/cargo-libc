#!/usr/bin/env python3
"""Combined source-candidate regression; needs libc-abi-tools' provider IO fix."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())


def run(args, closed=False, sink=None):
    args = [binary, "provider", *map(str, args)]
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
    listing = ["list", "--cache", cache]
    normal = run(listing)
    assert normal.returncode == 0 and normal.stdout.startswith(b"glibc\t"), normal
    assert not normal.stderr, normal.stderr
    closed = run(listing, closed=True)
    assert closed.returncode == 0 and not closed.stderr, closed

    # Mirror the producer's export regression. Entry a exports and emits its
    # path; entry b copies its manifest, then fails on its missing descriptor.
    # No compiler or real provider build is involved.
    for name in ["a", "b"]:
        entry = cache / "x86_64-unknown-linux-gnu/glibc-2.22" / f"source-{name}"
        entry.mkdir(parents=True)
        (entry / "manifest.json").write_text("{}\n")
        if name == "a":
            (entry / "provider.txt").write_text("fixture\n")
    for closed in [False, True]:
        destination = Path(temp) / ("closed-export" if closed else "normal-export")
        result = run(["export", "--glibc", "2.22", "--cache", cache,
                      "--to", destination], closed=closed)
        assert result.returncode == 1, result
        assert result.stderr.startswith(b"cargo-libc:") and b"panicked" not in result.stderr
        for name in ["a", "b"]:
            assert (destination / f"glibc-2.22/source-{name}/manifest.json").is_file()
        if not closed:
            # Establish the ordinary failure baseline *and* earlier stdout.
            assert str(destination / "glibc-2.22/source-a").encode() in result.stdout, result
            print("Baseline: export emitted the first path, then failed on the second entry.")
        else:
            print("Closed stdout: both exports reached their side effects; later failure retained exit 1.")
    with open("/dev/full", "wb", buffering=0) as full:
        failed = run(listing, sink=full)
    assert failed.returncode == 1 and failed.stderr, failed
    assert b"panicked" not in failed.stderr, failed.stderr
print("Provider stdout: normal output, closed reader, later failure and non-pipe error passed.")
