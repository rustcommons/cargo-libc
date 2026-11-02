# Release history reconstruction — 2026-11-02

This audit accompanies the new [changelog](../../CHANGELOG.md) and user-facing
[versioned notes](v0.3.0.md); detailed release records remain validation evidence.
All eight existing tags correspond to non-yanked `cargo-libc` registry versions.
Dates are original registry publication dates in UTC, not the date a GitHub Release
will be backfilled. GitHub Releases were absent when inspected on 2026-11-02.
Publication remains pending review; no new tag/version or crate upload is needed.

## Immutable identities

The registry API and each official archive were read on 2026-11-02. Each archive
checksum matches registry metadata; `.cargo_vcs_info.json` is clean at the tag's
peeled commit. All packaged `src/`, original manifest, lock, README and legal files
match that source. This is a read-only identity audit, not rerun runtime validation.

| Version | Original UTC date | Exact tag commit | Archive SHA-256 | Files |
| --- | --- | --- | --- | --- |
| [0.1.0](v0.1.0.md) | 2026-10-13 | `ce9a7d51122f143091050d9e53b1222b77e67de1` | `c05bff64b909bd7e2e84f4be403f2218fb3c420f0699531b91c4ad8136a6c5cc` | 22 |
| [0.1.1](v0.1.1.md) | 2026-10-13 | `b9d2c4f17848d8b0bba56178d5ee6090b1feffd6` | `a37d78a71ca37ecc8515e31de5e624586f5f25003ac4e308fd27094b95f6b539` | 22 |
| [0.2.0](v0.2.0.md) | 2026-10-14 | `3b283b044dca36d0e5b446d99874e0c46ea8c402` | `fa7f415917d7b9da0416792c10a5347719f625cb13c6ebea695a3fc63326946b` | 22 |
| [0.2.1](v0.2.1.md) | 2026-10-31 | `767c16aef0e49c37018b3907be480363dfdb9175` | `715e9c7eee7896a0e5055df1d67cdc4df7d4a57670f1d769562bcc19f2916398` | 23 |
| [0.2.2](v0.2.2.md) | 2026-11-01 | `b87563c6be6faf89ac26055f88fb520991125b8b` | `7ebc74cde32437217a0e8bcf0fec84c33436fb2ed33d066fac9651e4d0da7caf` | 23 |
| [0.2.3](v0.2.3.md) | 2026-11-01 | `289f28d161027e86decc808fc1745ec4a5b74d39` | `cec36f14679064031545748420e97b3fbadea811be7b504177f6b25c1dfd585a` | 23 |
| [0.2.4](v0.2.4.md) | 2026-11-01 | `889515b191ead51303b1321de858ec4e2209722c` | `b456ab453f55e12c7a890f1b40b759a9d0d8c12c17acedc7af331ca8a1774868` | 23 |
| [0.3.0](v0.3.0.md) | 2026-11-02 | `a70ce9ffac6be8611b66a2829fefb3093cb41ce6` | `96201f1b35c093b3ed77867995bc688336fc2cbb5a43f6d471a9e13b9ab3f5c2` | 25 |

Sources: [registry metadata](https://crates.io/api/v1/crates/cargo-libc),
[existing tags](https://github.com/rustcommons/cargo-libc/tags) and official
`https://static.crates.io/crates/cargo-libc/cargo-libc-VERSION.crate` archives.
The versioned notes link the corresponding immutable tags and original records.
No tag, source, lock or runtime data is changed by this audit.

## Historical evidence limits

- 0.1.0: post-publication commit `fc74c88e4ccfb558a02e0aececec9dc9d32dccd3`
  records 12/12 CI and a registry-installed WSL glibc 2.17 build/run. The original
  exact CI run/artifact set was not recovered/re-audited for this backfill.
- 0.1.1: commit `a564f4b411e4070824da53b87967eedaf7d830ab` records 13/13 CI,
  a registry-installed wheel build and bounded PyO3/maturin/runtime checks.
  Those outcomes remain attributed historical reports, not newly executed tests.
- 0.2.0: tag/source diff establishes the visibility/documentation change;
  `bd22115792cb6e5f1daed70955b38ed77ceeb872` acknowledges publication.
  An exact-release archived CI artifact set was not recovered/re-audited here.
- 0.2.1–0.3.0: maintained detailed `docs/RELEASE-*.md` records identify exact
  registry/native/package/install runs, archived evidence and exclusions. Their
  outcomes are preserved, not relabeled as tests run by this documentation change.
- Historical READMEs can contain outdated relative documentation links or status
  labels. Backfilled notes live on main; they do not change old registry archives.
  Never link to `blob/vOLD/CHANGELOG.md`, which did not exist there.
- Actual old-system tier 3 remains deferred, prebuilt legal clearance remains
  unresolved, and the 0.3.0 strict consumer libgcc_s/non-clock gap remains. None is
  resolved by changelog coverage or GitHub Release publication.

## Bootstrap and publication still pending

The shared local release-doc checker, syntax/link/render/diff checks establish
candidate documentation consistency. A first workflow dispatch may return 404
until the workflow exists on main; that is not a passing hosted gate. After
review GO and normal merge, require a green release-documentation run before
publishing notes for existing tags in ascending version order. Use reviewed note
files, `--verify-tag`, `--latest=false` for older releases and newest stable latest.
No binary assets or crates.io publication are part of that work. Check live body,
tag, links and latest status with the checker's published mode afterwards.
