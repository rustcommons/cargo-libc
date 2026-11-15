#!/usr/bin/env python3
"""Read-only check of a locally packaged crate against crates.io API and index."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import tarfile
import time
import tomllib
import urllib.error
import urllib.request


def identity(archive, commit):
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("expected exact release commit")
    with tarfile.open(archive, "r:gz") as package:
        manifests = [m for m in package.getmembers() if m.name.count("/") == 1
                     and m.name.endswith("/Cargo.toml")]
        if len(manifests) != 1:
            raise ValueError("expected one root crate manifest")
        manifest = manifests[0]
        metadata = tomllib.loads(package.extractfile(manifest).read().decode())["package"]
        name, version = metadata["name"], metadata["version"]
        if name != "cargo-libc" or manifest.name != f"{name}-{version}/Cargo.toml":
            raise ValueError("unexpected package identity")
        vcs = json.load(package.extractfile(f"{name}-{version}/.cargo_vcs_info.json"))
        if vcs["git"]["sha1"] != commit or vcs["git"].get("dirty", False):
            raise ValueError("package VCS does not match clean release source")
    return name, version, hashlib.sha256(Path(archive).read_bytes()).hexdigest()


def get(url):
    request = urllib.request.Request(url, headers={"User-Agent": "cargo-libc-release-verifier"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def verify_registry(name, version, checksum, fetch=get):
    registry = json.loads(fetch(f"https://crates.io/api/v1/crates/{name}/{version}"))["version"]
    if registry["checksum"] != checksum or registry["yanked"]:
        raise ValueError("API checksum/yank state disagrees with local package")
    path = f"{name[:2]}/{name[2:4]}/{name}"
    entries = [json.loads(line) for line in fetch(f"https://index.crates.io/{path}").splitlines()]
    entry = next((e for e in entries if e["vers"] == version), None)
    if entry is None:
        raise LookupError("version not yet present in sparse index")
    if entry["cksum"] != checksum or entry["yanked"]:
        raise ValueError("sparse index checksum/yank state disagrees with local package")
    official = fetch(f"https://static.crates.io/crates/{name}/{name}-{version}.crate")
    if hashlib.sha256(official).hexdigest() != checksum:
        raise ValueError("official archive bytes disagree with local package")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True)
    parser.add_argument("--commit", required=True)
    args = parser.parse_args()
    name, version, checksum = identity(args.archive, args.commit)
    # Publication propagation is bounded; checksum disagreement never retries.
    for attempt in range(12):
        try:
            verify_registry(name, version, checksum)
            break
        except (urllib.error.URLError, TimeoutError, LookupError):
            if attempt == 11:
                raise
            time.sleep(10)
    print(json.dumps(dict(crate=name, version=version, checksum=checksum,
                          commit=args.commit, api_index_archive_verified=True)))


if __name__ == "__main__":
    main()
