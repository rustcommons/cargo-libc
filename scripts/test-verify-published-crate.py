#!/usr/bin/env python3
"""Offline success and refusal controls for the publication verifier."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "verifier", Path(__file__).with_name("verify-published-crate.py"))
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)

COMMIT = "a" * 40


class PublicationVerification(unittest.TestCase):
    def package(self, directory, dirty=None):
        archive = Path(directory) / "cargo-libc-1.2.3.crate"
        vcs = {"git": {"sha1": COMMIT}, "path_in_vcs": ""}
        if dirty is not None:
            vcs["git"]["dirty"] = dirty
        with tarfile.open(archive, "w:gz") as package:
            for filename, content in {
                "Cargo.toml": '[package]\nname = "cargo-libc"\nversion = "1.2.3"\n',
                ".cargo_vcs_info.json": json.dumps(vcs),
            }.items():
                data = content.encode()
                member = tarfile.TarInfo("cargo-libc-1.2.3/" + filename)
                member.size = len(data)
                package.addfile(member, io.BytesIO(data))
        return archive

    def test_clean_archive_and_exact_vcs(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = self.package(directory)
            self.assertEqual(verifier.identity(archive, COMMIT), (
                "cargo-libc", "1.2.3", hashlib.sha256(archive.read_bytes()).hexdigest()))
            with self.assertRaises(ValueError):
                verifier.identity(archive, "b" * 40)
            with self.assertRaises(ValueError):
                verifier.identity(archive, "main")

    def test_dirty_archive_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ValueError):
                verifier.identity(self.package(directory, dirty=True), COMMIT)

    def registry(self, *, api_hash=None, index_hash=None, yanked=False,
                 index_yanked=False, missing=False, official=b"official"):
        checksum = hashlib.sha256(b"official").hexdigest()
        def fetch(url):
            if "/api/v1/" in url:
                return json.dumps({"version": {
                    "checksum": api_hash or checksum, "yanked": yanked}}).encode()
            if "index.crates.io" in url:
                return b"" if missing else json.dumps({
                    "vers": "1.2.3", "cksum": index_hash or checksum,
                    "yanked": index_yanked}).encode()
            return official
        verifier.verify_registry("cargo-libc", "1.2.3", checksum, fetch)

    def test_matching_registry(self):
        self.registry()

    def test_checksum_disagreements(self):
        for change in [{"api_hash": "wrong"}, {"index_hash": "wrong"},
                       {"official": b"changed"}]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.registry(**change)

    def test_yanks_refused(self):
        for change in [{"yanked": True}, {"index_yanked": True}]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.registry(**change)

    def test_index_propagation_incomplete(self):
        with self.assertRaises(LookupError):
            self.registry(missing=True)


if __name__ == "__main__":
    unittest.main()
