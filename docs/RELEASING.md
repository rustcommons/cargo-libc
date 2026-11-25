# Release and versioning policy

Adopted 2026-11-01. This is the maintained policy and release-record index for
cargo-libc. Versions are independent of the five-crate
[linux-abi-tools workspace](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/RELEASING.md)
and of glibc data/source packages.

## User-facing release documentation

Maintain the root [changelog](../CHANGELOG.md), with Unreleased entries for
noteworthy user-visible work and dated summaries of shipped versions. Commit
user-facing notes in `docs/releases/vX.Y.Z.md` before a release: Changes, Upgrade,
Validation and limits, Links. Use absolute links in note bodies because the same
reviewed files become [GitHub Releases](https://github.com/rustcommons/cargo-libc/releases).
Detailed qualification reports below are evidence, not substitutes for release notes.

The exact [AGENTS release rule](../AGENTS.md#release-documentation) applies.
Software release completion requires agreement among immutable source tag,
registry package, changelog/notes, docs and applicable gates, including the live
GitHub Release text/tag/links. GitHub notes provide documentation and source/registry
links; distribution remains crates.io. Since 2026-11-21 the owner authorizes the
pipeline to attach the `.crate` files, SBOM, `SHA256SUMS` and provenance bundle;
no other binaries.

The [release-documentation workflow](../.github/workflows/release-docs.yml) runs
the stdlib-only [checker](../scripts/check-release-docs.py), covering current
manifest-version/changelog/note structure locally and exact live note bodies in
published mode. Run it before routine merges and completion. Bootstrap exception:
review local checker and syntax evidence first; register the workflow through a
normal reviewed merge, then require a green dispatch before completing bootstrap
or publishing notes. A first-dispatch 404 before main registration is expected,
not a successful check.

Historical backfill: [all eight notes](releases/HISTORY.md) are reconstructed
from existing tags, archives and evidence. They did not exist at old tags; link
to main or the new documentation commit, never a nonexistent old changelog.
Changelog dates are original registry UTC dates; GitHub publication timestamps are
the later backfill date. Original records/archives/tags stay unchanged. The
[completed backfill](releases/HISTORY.md#completed-backfill) records the successful
bootstrap and full live verification; the following procedure remains the rule.
After merge and green docs CI, create existing-tag Releases in ascending order
from reviewed notes with `--verify-tag`, older versions `--latest=false`, newest
stable latest. Then check all live bodies/latest with:

```sh
python3 scripts/check-release-docs.py --repo rustcommons/cargo-libc
python3 scripts/check-release-docs.py --repo rustcommons/cargo-libc --published
```

The second command intentionally fails while GitHub Releases are missing; it
must not be described as passed merely because local structure passes. No crate
publication, tag movement or new runtime validation is part of backfilling notes.
The [deferred unwinder investigation](UNWINDER-0.4.0-PLAN.md) has no assigned
release; its historical filename does not identify the 0.4.0 diagnostics release.

## Release records

Latest verified registry/source release: **0.7.1** (2026-11-21).
See the [release record](RELEASE-0.7.1.md) for exact registry/source/tag/docs.rs
and clean installed evidence and remaining limits.
Historical GitHub Release backfill is complete for the eight versions through
0.3.0; see the [backfill completion evidence](releases/HISTORY.md#completed-backfill).
The 0.4.0 live Release/body verification is tracked in its release record. Packaged
READMEs link here without calling a pinned historical version current.

| Version / user notes | Original UTC date | Registry/source state and evidence |
| --- | --- | --- |
| [0.7.1](releases/v0.7.1.md) / [evidence](RELEASE-0.7.1.md) | 2026-11-21 | Published through Trusted Publishing from `15d27d50d7be9b5d7e2822e47b493147fb382c3a`; linux-abi-tools 0.6.1 crate set with unchanged scan data/behavior. Official archive/API/index/VCS, audited 0.7.0 delta, annotated tag, docs.rs and fresh registry installation verified. |
| [0.7.0](releases/v0.7.0.md) / [evidence](RELEASE-0.7.0.md) | 2026-11-08 | Published from `6317117a2bea3644fa0cba11df5ac61448fd7408`; producer 0.6.0 adoption, exact official archive/index/VCS/tag/docs.rs verified. Fresh registry installs and bounded generation/C/CMake/capability/stream checks passed; hosted Docker/Podman run/test evidence and full Release closeout are recorded separately. |
| [0.6.0](releases/v0.6.0.md) / [evidence](RELEASE-0.6.0.md) | 2026-11-07 | Published from `460bca52512658df8fea8d76578cd83dd0c2cb84`; duplicate-option contract, unchanged dependency graph, qualified package and clean installation. Official archive/index/VCS, annotated tag, docs.rs and fresh registry installation verified; full scope and Release closeout are recorded separately. |
| [0.5.0](releases/v0.5.0.md) · [evidence](RELEASE-0.5.0.md) | 2026-11-03 | Published/non-yanked from `1761260a52e192e55f774602ca05add64bd88994`; exact official/index checksum, source/payload, annotated tag and docs.rs verified. Candidate qualification retained separately; actual fresh native registry locked installation/all-five/checker36/legacy/capability/stream verification passed without failures. Final Release documentation uses the reviewed committed body and published-mode check. No runtime replacement or new platform qualification. |
| [0.4.0](releases/v0.4.0.md) · [evidence](RELEASE-0.4.0.md) | 2026-11-03 | Published/non-yanked; source/tag `aa2ea56e5d7943aeb4e0023713a089693d2103c1`; official 26-file archive/index checksum, docs.rs actual README/API/navigation and fresh native registry locked install verified. Diagnostics/missing-interpreter status change only; no runtime replacement or new platform qualification. |
| [0.3.0](releases/v0.3.0.md) · [evidence](RELEASE-0.3.0.md) | 2026-11-02 | Published and verified; source/tag `a70ce9ffac6be8611b66a2829fefb3093cb41ce6`; five-clock profile, archive/docs.rs and clean published-install run 41382187049 passed; explicit coverage limits retained. |
| [0.2.4](releases/v0.2.4.md) · [evidence](RELEASE-0.2.4.md) | 2026-11-01 | Published and verified; source/tag `889515b191ead51303b1321de858ec4e2209722c`; archive, actual README render and published-install run 41276430527 passed. |
| [0.2.3](releases/v0.2.3.md) · [evidence](RELEASE-0.2.3.md) | 2026-11-01 | Published and verified; source/tag `289f28d161027e86decc808fc1745ec4a5b74d39`. |
| [0.2.2](releases/v0.2.2.md) · [evidence](RELEASE-0.2.2.md) | 2026-11-01 | Published and verified; source/tag `b87563c6be6faf89ac26055f88fb520991125b8b`. |
| [0.2.1](releases/v0.2.1.md) · [evidence](RELEASE-0.2.1.md) | 2026-10-31 | Published and verified; source/tag `767c16aef0e49c37018b3907be480363dfdb9175`. |
| [0.2.0](releases/v0.2.0.md) | 2026-10-14 | Published; documented entry-point API; exact tag/archive and historical gaps in the notes. |
| [0.1.1](releases/v0.1.1.md) | 2026-10-13 | Published; provider-backed shared libraries/Python extensions; original evidence is attributed, not newly rerun. |
| [0.1.0](releases/v0.1.0.md) | 2026-10-13 | Published; first Cargo integration; exact archive/source identity and historical gaps in the notes. |

## Choosing a version

Release when approved scope is ready and applicable gates pass, not automatically
for every dependency update or GitHub documentation change. Our conservative 0.x
policy is:

| Change | Release |
| --- | --- |
| Compatible bug fixes, accurate documentation/packaging, or compatible dependency updates preserving documented contracts | Patch (`0.2.1` to `0.2.2`) |
| New command, API, workflow or platform scope; breaking CLI, output, config or provider contracts | Minor (`0.2.x` to `0.3.0`) |
| Increased MSRV or required native-tool floor, or removal of support | Explicit compatibility review and minor |

These are project choices, not a universal Cargo requirement that every feature
or MSRV increase use a minor version. Cargo's
[SemVer guidance](https://doc.rust-lang.org/cargo/reference/semver.html) and
[dependency requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
explain compatibility and 0.x caret ranges. For example, `"0.3.2"` means
`>=0.3.2, <0.4.0`, not an unbounded minimum.

Correcting behavior that violates an existing documented contract can be a patch,
even if users encounter different results. Deliberately adopting a new stricter
validation or refusal policy changes the contract and requires a minor. Review
borderline cases explicitly; release notes explain the old behavior, correction
or new rule, observable impact and any migration. The 0.2.1 owned/shared stdout
broken-pipe fixes and required `libc-abi-tools = "0.3.2"` minimum are a patch
example: later failures and scan findings still retain their failure status.

The 0.4.0 diagnostics release is a minor under this rule: after existing checks
allow a direct run, a missing PT_INTERP previously reached failed OS launch and
helper error 1; the new preflight refuses with status 2 before launch. Improved
wording alone could be a patch, but this deliberate refusal/status change is not.
See [0.4.0 upgrade guidance](releases/v0.4.0.md#upgrade). This version does not
reserve or adopt the separate unwinder proposal.

A stable 1.0 contract and subsequent stable-major policy require separate
adoption. No 1.0 schedule or stable API guarantee is implied here.

## Contracts to review

Review supported CLI arguments and environment variables, documented exit
statuses, documented JSON/TSV fields and semantics, generated Cargo configuration,
and provider descriptors, cache formats and validation semantics. Include shared
provider behavior delegated to dependencies; follow the
[provider contract](PROVIDER-CONTRACT.md) and [user guide](USER-GUIDE.md).
Do not infer that every internal detail or undocumented format is guaranteed.

Human-readable help and diagnostic wording may improve without preserving exact
text; it is not automatically a machine interface. Machine-readable contracts
need compatibility review, including changes to interpretation and failure status,
not just field spelling. Generated config review includes host/target isolation
and machine-specific paths. Cache/descriptor changes must consider existing
providers, invalidation, reuse and refusal behavior.

Internal Rust entrypoints are not advertised as a public library API, although
the crate contains a library and generated API documentation. Review package and
API changes for consumer impact rather than assuming they are harmless.

## Validation and deferred qualification

Run automated checks appropriate to changed scope. Rust changes use the commands
in [AGENTS.md](../AGENTS.md); release validation includes MSRV, fmt, tests, Clippy,
documentation and package checks. The maintained commands live in
[CI](../.github/workflows/ci.yml) and the manual
[integration workflow](../.github/workflows/candidate-integration.yml).
Use bounded native coverage for affected providers, C/CMake, plain Cargo config
formats, sysroots and host/target separation; expand it when changed inputs or
behavior require it. Unchanged inputs do not automatically require every provider,
Python or container matrix. Record exact revisions, commands, platforms, results,
artifacts and exclusions, as in [registry validation](REGISTRY-VALIDATION-0.2.1.md).

As decided on 2026-11-01, real old-system (tier 3) environments are unavailable;
the maintainer cannot provide or run them. Tier 3 is deferred qualification and
infrastructure work, not a blocker for ordinary releases. Such releases may ship
with passing applicable automated gates and an explicit tier 3 coverage gap.
Tier 3 is not complete: tier 1 inspection, tier 2 simulation and containers on a
modern kernel do not replace actual old-system evidence. Any new claim of real
old-system compatibility requires actual evidence for the claimed environment
and workload. A full-VM approach may be investigated later; no platform, hardware,
funding or delivery commitment exists. See the [roadmap](ROADMAP.md).

Documentation-only repository changes need link, command, rendering and diff
checks, not unrelated builds. Policy and roadmap text excluded by the package
include list needs no new crate publication. Packaged README changes reach
crates.io only with a later release of cargo-libc; repository edits do not alter
an existing registry archive.

## Preparing and publishing

1. Record approved scope, compatibility review, version choice, changelog entry
   and versioned user-facing notes; pass the release-documentation check.
   Obtain explicit user authorization for publication; existing authorization for
   the same release remains valid and does not require repeated approval.
2. Publish required producer versions first. Verify registry/index availability,
   non-yanked status, official archive checksum and source identity. Raise a direct
   caret minimum only when the consumer's feature or fix relies on it. Deliberately
   update the real lockfile, preserving unrelated inputs and their checksums.
3. Validate the registry-only dependency graph without path/git overrides or
   patches. Source-candidate tests with disposable patches are useful prerelease
   evidence, not substitutes for registry validation. Update the
   [package audit](../scripts/check-release-package.py) for the new version and
   its expected delta from the previous official archive. The release PR
   touches `CHANGELOG.md`, `docs/releases/` and the audit, so it gets a
   [Trusted publishing](#trusted-publishing) dry run: `gate` and `build` run
   the full release qualification (package audit, SBOM, checksums, portable
   checks, native providers/C/CMake/commands/sysroot/streams/malformed scans and
   a clean locked install of the packaged crate) with `publish=false`. That
   dry run must be green before merge.
4. Write `docs/releases/vX.Y.Z.md` in its final, published form before tagging:
   the tagged file becomes the GitHub Release body unchanged, and the
   release-documentation check compares the live body with it. Put
   post-publication evidence in the release record (`docs/RELEASE-X.Y.Z.md`),
   not in the notes.
5. Review and merge the PR normally and wait for CI on the merge commit. Push an
   annotated `vX.Y.Z` tag at that exact commit. The pipeline then runs:
   **Release gate** (annotated tag equal to the version, commit reachable from
   `main` with green CI, release crate set, dated changelog entry and notes) →
   **Build** → **Checksums and attestations** → the owner approves the
   `crates-io` environment → **Publish to crates.io** → **GitHub release**
   (created automatically from the committed notes, with files attached) →
   **Verify publication**. Distribution is the cargo-libc package on crates.io;
   the release attachments document it. Never replace a tag or published version.
6. The maintainer no longer creates the GitHub Release by hand. `verify` proves
   that crates.io's checksum equals the attested `.crate`, the attached files
   pass `gh attestation verify`, docs.rs built the version (a queue still
   pending after its bounded wait is reported as pending, not passed) and a clean
   `cargo install cargo-libc --version X.Y.Z --locked` passes the installed-helper
   checks. Then run
   `python3 scripts/check-release-docs.py --repo rustcommons/cargo-libc --published`
   and record the run, identities, docs.rs state and any remaining gaps in the
   release record through a normal PR.

If a gate fails, stop the affected next step and correct or document the failure;
do not bypass a failed gate to claim completion. If publication returns an
uncertain result, query the registry before retrying. Publication cannot be
rolled back by editing Git or moving a tag: the published version is immutable.
A yank prevents new resolutions but does not erase archives, existing lockfiles
or installations. Any yank requires explicit authorization; publish a corrective
version through the same process when necessary. Record partial publication,
docs.rs or installation failures honestly until resolved; do not label an
unverified release complete.

## Trusted Publishing

[`trusted-publishing.yml`](../.github/workflows/trusted-publishing.yml) is the
Rust Commons
[standard release pipeline](https://github.com/rustcommons/.github/blob/main/docs/TRUSTED-PUBLISHING.md)
(workflow `Trusted publishing`) and the only crate publication route: the owner
revoked the owner-machine crates.io token on 2026-11-16, so local
`cargo publish` is unavailable. cargo-libc adopted it as the pilot on
2026-11-21; its first real use is the next authorized release, and no published
version is re-tagged or re-released.

| Job | Display name | Runs | Extra permissions |
| --- | --- | --- | --- |
| `gate` | Release gate | every run | `actions: read` |
| `build` | Build | every run | none |
| `attest` | Checksums and attestations | tag runs | `id-token: write`, `attestations: write` |
| `publish` | Publish to crates.io | tag runs, after owner approval | `id-token: write`, environment `crates-io` |
| `github-release` | GitHub release | tag runs | `contents: write` |
| `verify` | Verify publication | tag runs | none |

[`release-gate.py`](../scripts/release-gate.py) holds the gate logic. On a tag
it refuses unless the tag is annotated and equals `v` plus the version, the
commit is reachable from `main` with a successful CI run, every crate at the
release version is in the release set (others must already be on crates.io),
`CHANGELOG.md` has a dated `## [X.Y.Z] - YYYY-MM-DD` entry and
`docs/releases/vX.Y.Z.md` exists and passes the release-documentation check.
Pull requests and manual runs report the same findings with `publish=false`.
`build` packages with `--locked`, runs the package audit and the release
qualification, generates a CycloneDX SBOM (`cargo-cyclonedx` 0.5.10,
`--locked`), writes `SHA256SUMS` and uploads the `release-<version>` artifact.
`attest` re-checks the sums and signs SLSA build provenance for the `.crate`
and SBOM, saved as `cargo-libc-<version>.crates.intoto.jsonl`. `publish`
re-checks the sums. Nothing compiles in the publish job, because it holds
OIDC and the `crates-io` environment: `build` already built the packaged
source. Each release crate is archived again with `cargo package --locked
--no-verify` and compared byte-for-byte with the attested `.crate`. Before the
token is obtained, `SHA256SUMS` is signed and verified as described below. The
token is then obtained, and a single step uses it only for `cargo publish --locked
--no-verify` in dependency order. It
skips a version crates.io already has, so a re-run is safe. Pull requests from
forks skip the dry run.
`github-release` creates `vX.Y.Z` with `--verify-tag` from the committed notes
(title: their first heading) and attaches the `.crate`, SBOM, `SHA256SUMS`,
`SHA256SUMS.asc` and provenance bundle. It creates a draft, checks its tag,
title, body and state, uploads missing assets, and checks every asset by name,
size and SHA-256 digest before publishing and again afterwards. A re-run
resumes a draft without replacing assets; a published release must already
match and is never modified. A regenerated OpenPGP signature may have a
different signing time; an existing signature asset is retained only after
downloading it, checking its size/digest and verifying it against the pinned
key and the exact checksum file. A tag run is never cancelled.

### OpenPGP release signatures

The owner authorized `SHA256SUMS.asc` on 2026-11-25 for future releases
(after 0.7.1); earlier releases keep their existing assets and verification.
The committed [public key](../.github/release-signing-key.asc) has primary
fingerprint `B281C3010A76A11737B4D066C7396F6EA7647CC6` and signing subkey
`248DFD639B75D1EFF2A87289D7C7DE7BBCCF35EC`.

Configure `RELEASE_SIGNING_KEY` (the armored signing secret subkey) and
`RELEASE_SIGNING_PASSPHRASE` only as `crates-io` environment secrets. The publish job installs Rust using the
runner's own `rustup`, so no third-party action runs before signing secrets are
used; nothing compiles in this job. The primary secret key stays offline. The publish step passes the passphrase only
on stdin using loopback pinentry. It imports the subkey in a temporary mode-700
GnuPG home, signs with the exact subkey, and immediately verifies in a second
home containing only the committed public key. Both the imported primary key
and exactly one signature's `VALIDSIG` primary fingerprint must match the constant in
[`release-signing.sh`](../scripts/release-signing.sh). It kills the agents and
removes both homes on success or failure. Verification rejects revoked, bad,
unknown-key and expired-signature statuses. A signature from a now-expired
subkey remains valid only if its recorded signing timestamp precedes that
subkey's expiry. Exactly one `GOODSIG` (or eligible `EXPKEYSIG`) and one
`VALIDSIG` are required. Signing and artifact upload run even
if every crate version is already published; either must succeed before token
minting or publishing.

The GitHub release and publication verification jobs use the same pinned
verification command, followed by `sha256sum --check --strict` over every
listed file. To verify downloaded assets from a trusted repository checkout:

```sh
bash scripts/release-signing.sh verify /path/to/downloaded-assets
```

For manual verification, download the public key and all listed assets, inspect
its primary fingerprint against the pin above, then run:

```sh
gpg --show-keys --with-fingerprint release-signing-key.asc
gpg --import release-signing-key.asc
gpg --verify SHA256SUMS.asc SHA256SUMS
sha256sum --check SHA256SUMS
```

Both commands must succeed, but a zero GnuPG exit status alone does not reject
revoked or expired signing keys. Use the pinned script above for the full
status/count/signing-time rules. The PR build runs
[`test-release-signing.py`](../scripts/test-release-signing.py) with an ephemeral,
passphrase-protected signing subkey, and
[`test-create-github-release.py`](../scripts/test-create-github-release.py) with
mocked GitHub responses. They cover success, tampering, wrong keys/passphrases,
revoked and expired signing subkeys, duplicate signatures, cleanup, draft resume and refusal to replace mismatched published assets.
The production CLI has no key or fingerprint override; the throwaway key is
also tested against that CLI and must be rejected. Real signing remains
untested until the next owner-approved tag run.

crates.io binds publication to this file name and the `crates-io` environment;
never rename either. The registered Trusted Publisher is:

| Field | Value |
| --- | --- |
| Crate | `cargo-libc` |
| GitHub repository owner | `rustcommons` |
| GitHub repository | `cargo-libc` |
| Workflow filename | `trusted-publishing.yml` |
| Environment | `crates-io` |

The environment must retain its owner reviewer and `v*` tag restrictions. Only
the `publish` job uses that environment, so only it can obtain a crates.io
token; `attest` and Scorecard use OIDC only to sign or authenticate results.
No long-lived registry credential is stored. See the
[crates.io Trusted Publishing documentation](https://crates.io/docs/trusted-publishing).

PR security checks run zizmor and cargo-deny. Dependabot proposes monthly grouped
minor/patch Cargo/action version updates, with a seven-day cooldown and at most
two open PRs per ecosystem. Security updates remain immediate and are independent
of this version-update schedule. `dtolnay/rust-toolchain` is the explicit manual
exception: its pinned commits encode different Rust versions. Review updates,
including security fixes, manually without collapsing MSRV/current coverage.
Dependabot-authored PRs run core fmt/Clippy/tests,
cargo-deny, zizmor, lf-only and managed CodeQL checks; expensive native matrices
and qualification jobs are skipped, including on human reruns. Core green is not
dependency compatibility or release qualification: maintainers must still review
and arrange applicable evidence before adoption. Human PRs and main retain full
CI. PR workflows cancel superseded runs for the same PR; non-PR runs stay
independent. CI and release-doc push triggers cover main and release tags,
avoiding duplicate branch-push/PR runs. The license policy preserves the existing
embedded glibc/Linux input
licenses and syscall exceptions, with permissions scoped to their reviewed crate
versions; it does not waive NOTICE or redistribution duties. RustSec checks cannot
establish the absence of unknown vulnerabilities or audit embedded native code.
Every workflow install is pinned (actions by SHA, hashed pip requirements,
locked exact `cargo install`, image digests, SHA-256-checked downloads); commit
and merge messages never use `[skip ci]`. Parsers of external input are fuzzed
per PR and weekly; see [FUZZING.md](FUZZING.md).
Scorecard runs on main and weekly, publishes its public results and uploads SARIF.
Its first live publication is pending this workflow's reviewed main merge. No
SLSA level, binary provenance or reproducible-build claim is made: this repository
currently distributes a source crate, not GitHub Release binary assets.
