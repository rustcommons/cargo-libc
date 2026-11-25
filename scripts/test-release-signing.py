#!/usr/bin/env python3
"""Real GnuPG round trip with a throwaway key; no release secrets required."""
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / 'scripts/release-signing.sh'
PIN = 'B281C3010A76A11737B4D066C7396F6EA7647CC6'


def run(args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True, **kwargs)


class SigningTests(unittest.TestCase):
    def test_signing(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            home = root / 'keygen'
            home.mkdir(mode=0o700)
            def gpg(*args, **kwargs):
                return run(['gpg', '--homedir', str(home), '--batch', *args], **kwargs)
            try:
                # Pin the committed key, independently of the production constant.
                gpg('--import', str(ROOT / '.github/release-signing-key.asc'))
                listing = gpg('--with-colons', '--list-keys').stdout.decode()
                self.assertEqual(listing.split('fpr:::::::::')[1].split(':')[0], PIN)
                self.assertIn('readonly RELEASE_PRIMARY_FINGERPRINT=' + PIN, SCRIPT.read_text())
                gpg('--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-generate-key', 'Release test <test@example.invalid>',
                    'rsa2048', 'cert', '1d', input=b'test-passphrase')
                listing = gpg('--with-colons', '--list-secret-keys').stdout.decode()
                fingerprint = listing.split('fpr:::::::::')[1].split(':')[0]
                gpg('--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-add-key', fingerprint, 'rsa2048', 'sign', '1d', input=b'test-passphrase')
                listing = gpg('--with-colons', '--list-secret-keys').stdout.decode()
                subkey = listing.split('fpr:::::::::')[2].split(':')[0]
                public = root / 'test.asc'
                public.write_bytes(gpg('--armor', '--export', fingerprint).stdout)
                secret = gpg('--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                             '--armor', '--export-secret-subkeys', fingerprint,
                             input=b'test-passphrase').stdout.decode()
                dist = root / 'dist'
                dist.mkdir()
                (dist / 'asset').write_text('release bytes\n')
                sums = run(['sha256sum', 'asset'], cwd=dist).stdout
                (dist / 'SHA256SUMS').write_bytes(sums)
                homes = root / 'homes'
                homes.mkdir()
                env = dict(os.environ, TMPDIR=str(homes), RELEASE_SIGNING_KEY=secret,
                           RELEASE_SIGNING_PASSPHRASE='test-passphrase')
                def helper(name, pin=fingerprint, signing=False, ok=True, extra_env=None):
                    args = ['bash', '-c', 'source "$1"; ' + name + ' "${@:2}"',
                            'test', str(SCRIPT), str(public), pin]
                    if signing:
                        args.append(subkey)
                    args.append(str(dist))
                    result = subprocess.run(args, env=extra_env or env, capture_output=True)
                    self.assertEqual(result.returncode == 0, ok, result.stderr.decode())
                    self.assertEqual(list(homes.iterdir()), [], 'temporary homes leaked')
                helper('sign_with_key', signing=True)
                helper('verify_with_key')
                helper('verify_with_key', pin=PIN, ok=False)
                # Production CLI cannot be redirected to the test key with environment overrides.
                result = subprocess.run(['bash', str(SCRIPT), 'verify', str(dist)],
                    env=dict(env, RELEASE_PUBLIC_KEY=str(public), RELEASE_PRIMARY_FINGERPRINT=fingerprint),
                    capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                (dist / 'asset').write_text('tampered')
                helper('verify_with_key', ok=False)
                (dist / 'asset').write_text('release bytes\n')
                (dist / 'SHA256SUMS').write_bytes(sums + b'\n# tampered\n')
                helper('verify_with_key', ok=False)
                (dist / 'SHA256SUMS').write_bytes(sums)
                (dist / 'SHA256SUMS.asc').unlink()
                helper('sign_with_key', signing=True, ok=False,
                       extra_env=dict(env, RELEASE_SIGNING_PASSPHRASE='wrong'))
                self.assertEqual(list(homes.iterdir()), [])
            finally:
                subprocess.run(['gpgconf', '--homedir', str(home), '--kill', 'gpg-agent'], check=True)

    def test_revoked_and_expired_subkeys(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            home = root / 'keygen'
            home.mkdir(mode=0o700)
            epoch = int(time.time()) - 10 * 86400
            def gpg(at, *args, **kwargs):
                return run(['gpg', '--homedir', str(home), '--batch',
                            '--faked-system-time', str(at), *args], **kwargs)
            try:
                gpg(epoch, '--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-generate-key', 'Expiry test <expiry@example.invalid>',
                    'rsa2048', 'cert', '0', input=b'\n')
                listing = gpg(epoch, '--with-colons', '--list-keys').stdout.decode()
                primary = listing.split('fpr:::::::::')[1].split(':')[0]
                gpg(epoch, '--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-add-key', primary, 'rsa2048', 'sign', '0', input=b'\n')
                listing = gpg(epoch, '--with-colons', '--list-keys').stdout.decode()
                subkey = listing.split('fpr:::::::::')[2].split(':')[0]
                (root / 'asset').write_text('release bytes\n')
                (root / 'SHA256SUMS').write_bytes(run(['sha256sum', 'asset'], cwd=root).stdout)
                def sign(at):
                    return gpg(at, '--local-user', subkey + '!', '--armor',
                               '--detach-sign', '--output', '-', str(root / 'SHA256SUMS')).stdout
                before = sign(epoch + 60)
                after = sign(epoch + 2 * 86400)
                # Shorten the subkey lifetime after producing the two signatures.
                gpg(epoch + 120, '--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-set-expire', primary, '1d', subkey, input=b'\n')
                # A newer signing subkey has no expiry; its expired sibling
                # still causes GnuPG to emit KEYEXPIRED when checking it.
                gpg(epoch + 200, '--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-add-key', primary, 'rsa2048', 'sign', '0', input=b'\n')
                listing = gpg(epoch + 200, '--with-colons', '--list-keys').stdout.decode()
                newer = listing.split('fpr:::::::::')[3].split(':')[0]
                new_signature = gpg(epoch + 3 * 86400, '--local-user', newer + '!', '--armor',
                    '--detach-sign', '--output', '-', str(root / 'SHA256SUMS')).stdout
                public = root / 'test.asc'
                def export():
                    public.write_bytes(gpg(epoch + 360, '--armor', '--export', primary).stdout)
                export()
                def verify(signature, expected, status_word, sibling_expired=False):
                    (root / 'SHA256SUMS.asc').write_bytes(signature)
                    status = subprocess.run(['gpg', '--homedir', str(home), '--batch',
                        '--status-fd', '1', '--verify', str(root / 'SHA256SUMS.asc'),
                        str(root / 'SHA256SUMS')], capture_output=True).stdout.decode()
                    self.assertIn('[GNUPG:] ' + status_word, status)
                    self.assertIn('[GNUPG:] VALIDSIG ', status)
                    if sibling_expired:
                        self.assertIn('[GNUPG:] KEYEXPIRED ', status)
                        self.assertNotIn('[GNUPG:] EXPKEYSIG ', status)
                    result = subprocess.run(['bash', '-c',
                        'source "$1"; verify_with_key "$2" "$3" "$4"', 'test',
                        str(SCRIPT), str(public), primary, str(root)], capture_output=True)
                    self.assertEqual(result.returncode == 0, expected, result.stderr.decode())
                verify(new_signature, True, 'GOODSIG', sibling_expired=True)
                verify(before, True, 'EXPKEYSIG')
                verify(after, False, 'EXPKEYSIG')
                verify(before + before, False, 'EXPKEYSIG')
                # Remove expiry to exercise revocation independently.
                gpg(epoch + 240, '--pinentry-mode', 'loopback', '--passphrase-fd', '0',
                    '--quick-set-expire', primary, '0', subkey, input=b'\n')
                # Revoke only the signing subkey, using the throwaway certify key.
                gpg(epoch + 300, '--yes', '--command-fd', '0', '--edit-key', primary,
                    input=b'key 1\nrevkey\ny\n0\nTest revocation\n\ny\nsave\n')
                export()
                verify(before, False, 'REVKEYSIG')
            finally:
                subprocess.run(['gpgconf', '--homedir', str(home), '--kill', 'gpg-agent'], check=True)


if __name__ == '__main__':
    unittest.main()
