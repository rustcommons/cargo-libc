#!/usr/bin/env python3
"""Exercise draft creation, resume, immutable published assets and failures."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('create-github-release.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleaseTests(unittest.TestCase):
    def test_release(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            for name in ('cargo-libc-1.0.0.crate', 'cargo-libc-1.0.0.cdx.json', 'SHA256SUMS',
                         'SHA256SUMS.asc', 'cargo-libc-1.0.0.crates.intoto.jsonl'):
                (dist / name).write_text(name)
            notes = dist / 'notes.md'
            notes.write_text('# Release\n\nNotes.\n')
            state = {'record': None, 'assets': [], 'calls': [], 'bytes': {}}
            def gh(*args):
                state['calls'].append(args)
                if args[0] == 'api':
                    if '/releases/assets/' in args[-1]:
                        return state['bytes'][int(args[-1].rsplit('/', 1)[1])]
                    if '/assets?' in args[-1]:
                        return json.dumps([state['assets']]).encode()
                    return json.dumps([[state['record']] if state['record'] else []]).encode()
                command = args[1]
                if command == 'create':
                    self.assertIn('--draft', args)
                    self.assertIn('--verify-tag', args)
                    state['record'] = dict(id=1, tag_name='v1.0.0', name='Release',
                                           body=notes.read_text(), prerelease=False, draft=True)
                elif command == 'upload':
                    self.assertTrue(state['record']['draft'])
                    self.assertNotIn('--clobber', args)
                    path = Path(args[3])
                    asset_id = len(state['assets']) + 1
                    state['bytes'][asset_id] = path.read_bytes()
                    state['assets'].append(dict(id=asset_id, name=path.name, size=path.stat().st_size,
                        digest='sha256:' + hashlib.sha256(path.read_bytes()).hexdigest()))
                elif command == 'edit':
                    self.assertEqual(len(state['assets']), 5)
                    state['record']['draft'] = False
                return b''
            with patch.object(module, 'gh', side_effect=gh), patch.object(module, 'verify_signature') as verify:
                module.publish('owner/repo', '1.0.0', dist, notes)
                self.assertIn('SHA256SUMS.asc', {a['name'] for a in state['assets']})
                # Published rerun performs only reads.
                state['calls'].clear()
                module.publish('owner/repo', '1.0.0', dist, notes)
                self.assertTrue(all(c[0] == 'api' for c in state['calls']))
                # A regenerated signature keeps the existing verified remote bytes.
                (dist / 'SHA256SUMS.asc').write_text('new signature')
                state['calls'].clear()
                module.publish('owner/repo', '1.0.0', dist, notes)
                verify.assert_called_once_with(dist)
                self.assertEqual((dist / 'SHA256SUMS.asc').read_text(), 'SHA256SUMS.asc')
                self.assertTrue(all(c[0] == 'api' for c in state['calls']))
                # An invalid existing signature must fail before any mutation.
                (dist / 'SHA256SUMS.asc').write_text('new signature')
                with patch.object(module, 'verify_signature', side_effect=ValueError('bad signature')):
                    with self.assertRaises(ValueError):
                        module.publish('owner/repo', '1.0.0', dist, notes)
                # Resume an incomplete draft; keep all matching assets.
                state['record']['draft'] = True
                state['assets'].pop()
                state['calls'].clear()
                module.publish('owner/repo', '1.0.0', dist, notes)
                self.assertEqual(sum(c[:2] == ('release', 'upload') for c in state['calls']), 1)
                # Existing corrupted bytes fail, never replaced, even in a draft.
                for draft in (False, True):
                    state['record']['draft'] = draft
                    state['assets'][0]['digest'] = 'sha256:bad'
                    state['calls'].clear()
                    with self.assertRaises(ValueError):
                        module.publish('owner/repo', '1.0.0', dist, notes)
                    self.assertTrue(all(c[0] == 'api' for c in state['calls']))
                state['record']['name'] = 'Wrong title'
                with self.assertRaises(ValueError):
                    module.publish('owner/repo', '1.0.0', dist, notes)
                (dist / 'SHA256SUMS.asc').unlink()
                with self.assertRaises(ValueError):
                    module.publish('owner/repo', '1.0.0', dist, notes)


if __name__ == '__main__':
    unittest.main()
