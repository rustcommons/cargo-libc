# Release and versioning policy

Adopted 2026-11-01. This is the maintained policy and release-record index for
cargo-libc. Versions are independent of the five-crate
[linux-abi-tools workspace](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/RELEASING.md)
and of glibc data/source packages.

## Release records

Latest verified published release: **0.2.3**. Version **0.2.4 is a candidate, not
published**. Update this maintained index as verification completes; packaged
READMEs link here without calling a pinned historical version current.

| Version | Date | State and evidence |
| --- | --- | --- |
| [0.2.4](RELEASE-0.2.4.md) | 2026-11-01 | Candidate: registry/package/clean-install run 41275479293 passed at `896ac7b`; review/merge/publication pending. |
| [0.2.3](RELEASE-0.2.3.md) | 2026-11-01 | Published and verified; source/tag `289f28d161027e86decc808fc1745ec4a5b74d39`. |
| [0.2.2](RELEASE-0.2.2.md) | 2026-11-01 | Published and verified; source/tag `b87563c6be6faf89ac26055f88fb520991125b8b`. |
| [0.2.1](RELEASE-0.2.1.md) | 2026-10-31 | Published and verified; source/tag `767c16aef0e49c37018b3907be480363dfdb9175`. |

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

1. Record approved scope, compatibility review, version choice and release notes.
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
   each release (installed mode currently targets 0.2.4). Record verified outcomes
   and remaining coverage gaps in the release record and roadmap.

If a gate fails, stop the affected next step and correct or document the failure;
do not bypass a failed gate to claim completion. If publication returns an
uncertain result, query the registry before retrying. Publication cannot be
rolled back by editing Git or moving a tag: the published version is immutable.
A yank prevents new resolutions but does not erase archives, existing lockfiles
or installations. Any yank requires explicit authorization; publish a corrective
version through the same process when necessary. Record partial publication,
docs.rs or installation failures honestly until resolved; do not label an
unverified release complete.
