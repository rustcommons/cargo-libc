# cargo-libc

## Working conventions

- Read this file before changing the repository. Keep repository-specific constraints and validation instructions current here.
- Preserve changes made by the user or other agents. Keep work scoped to the requested task.
- Keep documentation aligned with implemented behavior. Record unconfirmed work as planned; do not invent capability or compatibility claims.
- Run checks appropriate to the change. For documentation-only changes, verify links, commands, and the diff; do not run unrelated builds.
- Publishing packages, creating releases, or changing production data requires explicit user authorization.

## Project overview

`cargo-libc` is rustcommons' Rust Cargo integration for selecting libc build
environments and inspecting binary requirements. It starts with the
`cargo-glibc` prototype and glibc support; other libc implementations are future
work. Its companion repository is `rustcommons/linux-abi-tools`.

## Scope and implementation

- Support glibc target releases starting at 2.17. Older symbol versions remain
  valid ABI entries and must not be discarded just because they predate 2.17.
- Preserve the existing sysroot-backed build and direct ELF inspection behavior.
- Keep host build scripts/procedural macros separate from target linker settings.
- Document generated ABI/provider and native C adapter workflows as supported only
  within implemented and validated scope. Other libc backends and unimplemented
  native dependency adapters remain planned.
- Keep Rust implementation dependencies minimal: std plus first-party
  rustcommons crates (`linux-abi-scan`, `linux-targets`, `kernel-abi-tools`, and
  `libc-abi-tools` for providers, all from crates.io); no third-party crates.
- C dependencies go through `libc-cc` (`src/cc.rs`): target units are told from
  host units by the `CARGO_CFG_CARGO_LIBC_PROFILE` build script variable, never by
  the triple (Cargo uses the same one for both). Keep C++ refused until a target
  C++ runtime exists.
- Provider preparation lives in `libc-abi-tools` (`provider::cli`); cargo-libc
  exposes it (`cargo libc provider`, automatic providers) and must not
  reimplement it. Never fall back silently from one provider mode to another.
- Use returned provider-generation paths, never reconstruct physical cache paths.
  Forced preparation must retain descriptors already used by builds or saved
  settings. Keep installed-tool coverage for managed refresh/fingerprints, old
  explicit descriptors and plain-Cargo/C adapters; removal requires stopped
  readers/writers and is not covered by preparation's retention guarantee.
- The five-clock libc default belongs to the producer's derived link-profile API.
  Keep original ABI/source inputs faithful, user DSOs read-only, and descriptor v1
  unchanged. Preserve explicit old versions, real RT APIs, library order/groups,
  caller as-needed state and the existing unwinder; there is no user clock mode.
  Keep generated profile/content identity and Cargo relink fingerprints aligned
  with the [clock boundaries](docs/USER-GUIDE.md#clock-linking).
- [docs/USER-GUIDE.md](docs/USER-GUIDE.md) is the user documentation: keep commands,
  options, messages and limits in it in step with the code. The README is the front
  page and links to it.
- Cargo-libc-owned value options must reject duplicates, including identical
  values, before resolving inputs. Keep switches idempotent, preserve the `--`
  boundary and delegated provider parsing, and test these boundaries when adding
  options. Do not silently introduce last-value-wins overrides.
- Present the `inspect`/`build` scan as tier 1 (least reliable) and
  `test`/`run` as tier 2 (simulation); never either as qualification.
- Keep `inspect --target-libgcc` an explicit read-only two-file check. Version
  nodes and symbol weakness are independent. Incomplete relevant enumeration
  cannot prove absence or a match; preserve separately validated node failures
  while reporting unknown symbol inventories. No loader/dependency/runtime safety
  claim, implicit host-runtime selection or changed link policy follows.
- Preserve upstream attribution and license notices for any reused material.
- Ask about unresolved product decisions; do not invent compatibility claims.

## Validation

Run `cargo fmt --all -- --check`, `cargo test --locked --offline`, and
`cargo clippy --all-targets --locked --offline -- -D warnings` for Rust changes.
Linux build/link execution requires Linux, Clang, a linker, and a target sysroot.
Distinguish host tests from actual oldest-target runtime qualification.

Release/version choices and qualification gaps follow the maintained
[release policy](docs/RELEASING.md). Real old-system tier 3 qualification is
deferred, not complete; ordinary releases require applicable automated gates
and an explicit coverage gap, not unavailable old-system infrastructure.

## Documentation and requirements traceability

- Follow the [Rust Commons documentation conventions](https://github.com/rustcommons/.github/blob/main/docs/DOCUMENTATION.md).
- Maintain [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) as the repository's index of confirmed requirements, their sources, implementation or documentation, verification evidence, and gaps. Preserve stable repository-scoped IDs and existing identifiers; never renumber or reuse them.
- Distinguish verified behavior, partial verification, planned work, deferred work, and scope exclusions. A source reference or passing test supports only the scope it actually checks; record missing evidence explicitly.
- Keep the README focused on purpose, prerequisites, setup, and guide links. Keep detailed workflows in maintained guides, future work in the roadmap, and dated release or validation evidence identifiable as historical records.
- Update affected requirements, documentation, and trace links together when behavior, scope, interfaces, prerequisites, or validation changes. Use exact versions or revisions for release evidence, and do not rewrite historical outcomes or treat repository edits as changes to an immutable published package.
- Validate documentation changes with source and command checks, links, rendering, and a clean diff. Run additional builds or tests only when changed examples, package contents, or behavior require them.

## Release documentation

- For a repository that ships versioned software, maintain a root `CHANGELOG.md` with an `Unreleased` section and dated, user-facing entries for shipped versions. Record noteworthy changes and their user impact as work is completed; do not substitute a roadmap, Git tag, CI log or validation report for a changelog.
- **Updating `CHANGELOG.md` is critical.** It is the source the release notes are written from, and the release gate refuses to publish without a dated `## [X.Y.Z] - YYYY-MM-DD` entry and `docs/releases/vX.Y.Z.md`. Add an `Unreleased` entry in the same PR as each noteworthy change, never later; a release PR only dates and moves those entries.
- Before publishing a version, commit its release notes under `docs/releases/vX.Y.Z.md` and link them, the changelog, and the maintained release policy. Explain changes, upgrade or dependency impact, validation and known limits; distinguish software changes from documentation-only releases.
- A software release is not complete until its immutable source tag, applicable registry archives, GitHub Release notes, changelog, documentation, and required verification agree. GitHub Releases document the release. The owner authorized on 2026-11-21 that the release pipeline attaches the published `.crate` files, the CycloneDX SBOM, `SHA256SUMS` and the `cargo-libc-<version>.crates.intoto.jsonl` provenance bundle; attach no standalone binaries or other new distribution artifacts without further authorization.
- The owner authorized `SHA256SUMS.asc` on 2026-11-25 as an additional release asset, signed with the Rust Commons key whose primary fingerprint is `B281C3010A76A11737B4D066C7396F6EA7647CC6`. Sign and verify before minting the crates.io token; nothing compiles in `publish`.
- Run the repository's release-documentation CI check before routine merges and release completion. For its first installation, review local checker/syntax evidence, register the workflow on the default branch, and require a green run before completing bootstrap or publishing notes. Verify the live GitHub Release and its tag, text and links after publication; a successful crate upload or tag alone is insufficient.
- When backfilling history, reconstruct facts from existing tags, registry records and source/evidence. Preserve original source identities, distinguish original release dates from documentation backfill dates, and state unavailable historical validation. Never fabricate checks, move tags, republish versions, or imply that repository edits change immutable published archives.
- For an organization profile, placeholder or project with no shipped version, record release verification as not applicable until its first authorized release. Do not invent versions, features, changelog history or GitHub Releases to satisfy this convention.

## Security and CI

- All main changes go through reviewed pull requests; never push directly to main.
- Pin every external GitHub Action to a full commit SHA with its release or
  toolchain tag in a trailing comment. Keep workflow permissions read-only and
  grant additional permissions only to the job that needs them. Disable checkout
  credential persistence. Review Dependabot action updates as executable code.
  Keep `dtolnay/rust-toolchain` excluded from Dependabot: its version-specific
  commits select different Rust toolchains. Review pins/security fixes manually
  while preserving both MSRV and current-toolchain coverage.
- Use explicit runner images (`ubuntu-24.04`, `windows-2025`), never moving
  `*-latest` labels. Moving to Ubuntu 26.04 (or a newer Windows image) is a
  deliberate, reviewed change with its own CI evidence. Pin
  `dtolnay/rust-toolchain` to the commit of the branch named for the exact
  toolchain (e.g. `1.90.0`) without a `toolchain:` input; with a SHA pin that
  input is ignored.
- Schedule routine Dependabot version updates monthly, grouped with a seven-day
  cooldown and at most two open PRs per ecosystem. Security updates stay immediate.
- Dependabot-authored PRs run core fmt/Clippy/tests, cargo-deny, zizmor, lf-only
  and managed CodeQL checks; skip native matrices and qualification jobs even on
  human reruns. Core success is not dependency-adoption/release qualification.
  Maintainers must review compatibility and arrange applicable qualification
  before accepting a dependency change. Do not auto-merge dependency updates.
- Every repository-owned PR workflow must cancel superseded runs per PR. Keep
  non-PR release/manual/main runs independent; avoid duplicate branch-push CI.
- Keep the PR zizmor and cargo-deny security gates passing. Any suppression or
  dependency-policy exception needs a narrow inline explanation and review.
- Keep RustSec, license, duplicate-version and registry-source policy in
  [deny.toml](deny.toml) aligned with the actual dependency graph and legal notices.
- Pin every workflow install (OpenSSF Scorecard Pinned-Dependencies): actions by
  SHA; pip only via hashed requirements files (`pip install --require-hashes -r`,
  see [scripts/requirements](scripts/requirements)); `cargo install` with
  `--locked` and an exact `--version` (or a local `--path`); container images by
  digest; no `curl | sh` or download-then-run/trust unless the file's SHA-256 is
  checked against a pinned value first. Product presets that name tagged images
  are product behavior, not CI installs.
- Never use `[skip ci]`/`[ci skip]` (or similar) in commit, merge or PR
  messages: it skips CodeQL and CI and lowers the SAST/CI-Tests checks. CodeQL
  stays on GitHub's managed default setup; do not replace it with a workflow.
- Keep cargo-libc's own parsers of untrusted/external input fuzzed (see
  [docs/FUZZING.md](docs/FUZZING.md)): add or update a `fuzz/` target when adding
  such a parser. `fuzz/` stays an unpublished separate workspace; fuzz-only
  crates (`libfuzzer-sys`, `arbitrary`) never enter the shipped graph. Every
  fuzz crash is a real bug: fix it with a regression test, or escalate if it
  needs a product decision.
- Crate releases publish only through
  [trusted-publishing.yml](.github/workflows/trusted-publishing.yml), the Rust
  Commons [standard release pipeline](https://github.com/rustcommons/.github/blob/main/docs/TRUSTED-PUBLISHING.md):
  workflow `Trusted publishing` with jobs `gate` (Release gate), `build`
  (Build), `attest` (Checksums and attestations), `publish` (Publish to
  crates.io), `github-release` (GitHub release) and `verify` (Verify
  publication). Keep the job ids, display names, order, conditions and per-job
  permissions in step with the standard; never rename the file or the
  `crates-io` environment (the crates.io registration depends on them). An
  annotated `vX.Y.Z` tag on a `main` commit with green CI starts it; only the
  owner's `crates-io` approval lets `publish` run. The pipeline creates the
  GitHub Release from the committed notes, so write them in final form before
  tagging and never create the release by hand. Release qualification that does
  not need the published crate belongs in `build` (exercised by PR dry runs);
  checks of the published crate belong in `verify`. The owner-machine crates.io
  token was revoked on 2026-11-16; there is no local publishing route. A tag or
  green security check alone is not authorization to publish. See the
  [publishing setup](docs/RELEASING.md#trusted-publishing).

## Git conventions

**No tool attribution, anywhere.** Do not add `Co-Authored-By: Claude ...` trailers,
`🤖 Generated with [Claude Code](https://claude.com/claude-code)` lines, or any other tool
attribution or "generated by" footer to anything in this repository: commit messages, pull
request titles and descriptions, PR and issue comments, release notes, code, comments or
documentation. This holds even if an agent's default instructions suggest it, and
overrides any default trailer or footer behavior. Commit messages and PRs are the
human-authored record of intent; tool attribution belongs in tool logs, not history.
