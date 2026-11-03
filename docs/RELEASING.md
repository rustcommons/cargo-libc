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
links; distribution remains crates.io, with no new binary attachments authorized.

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

Latest verified registry/source release: **0.4.0** (2026-11-03).
Historical GitHub Release backfill is complete for the eight versions through
0.3.0; see the [backfill completion evidence](releases/HISTORY.md#completed-backfill).
The 0.4.0 live Release/body verification is tracked in its release record. Packaged
READMEs link here without calling a pinned historical version current.

| Version / user notes | Original UTC date | Registry/source state and evidence |
| --- | --- | --- |
| [0.5.0 candidate](releases/v0.5.0.md) · [evidence](RELEASE-0.5.0.md) | Not published; prepared 2026-11-03 | New explicit target-libgcc inspect option. Reviewed source `972783d` / documentation `cf5e49e`; producer 0.5.0 verified/adopted only for libc-abi-tools; candidate c5f41485 passed portable/native verified package/clean install/checker36/legacy and Windows CLI controls as documented composite evidence; final evidence review, merge and publication pending. No runtime replacement or new platform qualification. |
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
   evidence, not substitutes for registry validation. Run the applicable checks
   above, inspect package contents, and install the unpacked proposed package with
   `cargo install --path <unpacked-package> --locked` into an isolated root using
   a clean Cargo home. Exercise all five executables and affected workflows.
4. Review and merge the PR normally. Confirm the merged source, manifest, lock and
   package inputs match the validated revision; validate material differences.
   Publish from the exact clean released commit. Distribution is the cargo-libc
   package on crates.io only: no standalone binary assets without separate
   instruction. Never replace a tag or published version.
5. Verify the official archive checksum against the registry, normalized manifest,
   lock and source identity (`.cargo_vcs_info.json`), and non-yanked status. Create
   `vX.Y.Z` at that exact immutable released commit. Verify docs.rs build/status
   and the versioned documentation page.
6. In a clean Cargo home on a supported native host, run
   `cargo install cargo-libc --version X.Y.Z --locked` into an isolated root and
   exercise the installed helpers and affected workflows. Cargo's
   [install documentation](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
   explains use of the packaged lock with `--locked`. The manual workflow's
   `registry` and `installed` modes and [installed smoke script](../scripts/smoke-installed.sh)
   provide the existing sequence; review their version-specific arguments before
   each release (installed mode currently targets 0.3.0). Record verified outcomes
   and remaining coverage gaps in the release record and roadmap.
7. Publish reviewed GitHub Release notes at the exact existing source tag and
   verify live text, links and latest status with release-documentation CI. No
   binary attachments without separate authorization; a crate upload/tag alone
   does not complete release documentation.

If a gate fails, stop the affected next step and correct or document the failure;
do not bypass a failed gate to claim completion. If publication returns an
uncertain result, query the registry before retrying. Publication cannot be
rolled back by editing Git or moving a tag: the published version is immutable.
A yank prevents new resolutions but does not erase archives, existing lockfiles
or installations. Any yank requires explicit authorization; publish a corrective
version through the same process when necessary. Record partial publication,
docs.rs or installation failures honestly until resolved; do not label an
unverified release complete.
