# cargo-libc 0.4.0 release record

Published on crates.io 2026-11-03 from exact clean merge
`aa2ea56e5d7943aeb4e0023713a089693d2103c1`, annotated tag `v0.4.0`.
Official archive/index/API, docs.rs and a separate fresh published installation
are verified below. GitHub Release completion uses the exact committed notes and
live checks described below; final run links belong to the bookkeeping PR and
release-documentation workflow history, without changing the immutable source tag.

The prepublication qualification sections retain their original tested source,
archive and install identities; they are not relabeled as registry installation.

[User-facing release notes](releases/v0.4.0.md), [changelog](../CHANGELOG.md),
[release policy](RELEASING.md) and [requirements](REQUIREMENTS.md) distinguish
release preparation from publication. The version is minor because a confirmed
missing local interpreter now refuses before launch with status 2; a 0.3.0 direct
run reaching failed OS launch returned helper error 1. System unwinding, clocks,
link state, dependencies, MSRV and native-tool floors remain unchanged. Research
PR14 and the target-libgcc comparator are not included.

## Exact identities and carry-forward

| Role | Exact ref / result |
| --- | --- |
| Reviewed diagnostics implementation | `d6b5f78fb31f7f6a5d8c604a313c78039573f9aa` |
| Normal implementation merge; identical tree | `748ffa23274792dd21e1cb4cd812271f1f61e1a3` |
| Version/docs preparation; clean native package and installation source | `62347644fe1e6808b19de043c702da69077409e8` |
| Reviewed excluded harness; final clean native package/VCS | `d335f8ec52a5b1e6331be57acecab01fe7f226c7` |

The final tested candidate archive is **73,212 bytes, 26 files**, SHA256
`fe29474d3075e14b00c2e454a6ce04c819bb7b86d6b24580d45b1d607233871e`,
with clean `.cargo_vcs_info.json` naming `d335f8e`. The earlier installed-source
archive is **73,210 bytes, 26 files**, SHA256
`da6b4c1a0252072c886a4f76ee4555965747c005f1ac9cebbca47df12140e6e9`,
with clean VCS `6234764`. Comparing every extracted file proves that **only
`.cargo_vcs_info.json` differs**. The installed binaries were built from the
unpacked `6234764` candidate; their runtime evidence carries to `d335f8e` by this
explicit compilation-input equality, not by a claim of an identical rebuilt
binary hash or a second installation.

All `src/` bytes match both reviewed merge `748ffa2` and preparation `6234764`.
The tracked manifest and lock are `6234764`-identical; versus official 0.3.0 only
the root package version changes. LICENSE/NOTICE are unchanged. The packaged
README's only change from official 0.3.0 is the two already-merged Changelog and
GitHub Releases navigation lines. Official 0.3.0 has 25 files, 70,579 bytes,
SHA256 `96201f1b35c093b3ed77867995bc688336fc2cbb5a43f6d471a9e13b9ab3f5c2`
and VCS `a70ce9ffac6be8611b66a2829fefb3093cb41ce6`.

The audited diagnostics delta adds only `src/capability.rs`; common source changes
are `src/cli.rs`, `src/lib.rs`, `src/simulate.rs` and `src/toolchain.rs`.
The provider-link startup/default-library/unwinder construction is byte-identical.
The complete [26-file inventory and SHA256s](evals/2026-11-03-diagnostics-package-inventory.json)
also records all five installed executable identities. No policy/guide/evaluation
files or qualification scripts are packaged. A later excluded evidence commit
changes its VCS identity; the archive hash above remains the exact tested
`d335f8e` artifact, not a predicted final-merge or official registry checksum.

## Environment and commands

Native x86_64 Linux on WSL2, glibc 2.39, Rust/Cargo 1.90.0,
Clang 18.1.3, GCC 13.3.0 and GNU binutils 2.42. CMake was exercised by the fixture.
The host kernel is modern WSL2; neither the 2.17 link target nor the 3.10
simulation identifies an actual old deployment OS. Workers were capped at two,
with one heavy package/build/install operation at a time. The documentation
owner audited supplied native evidence on ARM and did not execute these x86
binaries there.

The retained JSON/raw logs record commands, durations, inputs and outputs:

```sh
cargo package --locked --offline
cargo fetch --manifest-path <unpacked>/Cargo.toml --locked
cargo install --path <unpacked> --locked --offline \
  --root <isolated-install> --target-dir <isolated-target>
bash scripts/smoke-installed.sh <isolated-install> /usr/bin/clang-18 0.4.0
python3 scripts/check-installed-capabilities.py \
  <isolated-install>/bin /usr/bin/clang-18 <capability-evidence>
python3 scripts/check-run-streams.py <isolated-install>/bin /usr/bin/clang-18
python3 scripts/check-release-package.py <d335-package.crate> <audit-evidence>
```

Installation used a clean Cargo home: locked fetch followed by offline locked
installation from the unpacked candidate, never unpublished registry 0.4.0.
Metadata contains exactly seven packages: only the root is a path package; all
six dependencies are crates.io registry sources. All six downloaded official
archives match their lock checksums and registry API checksums and are non-yanked.
There are no production path/git patches, replacements or dependency upgrades.
The standalone driver documents clean Cargo homes and input checks; the retained
native evidence is the separately recorded sequence above, not a claim that the
entire driver or a hosted package workflow was dispatched.

## Passing scope

- **Source gates:** [CI 41633902033](https://github.com/rustcommons/cargo-libc/actions/runs/41633902033)
  and [PR CI 41633910012](https://github.com/rustcommons/cargo-libc/actions/runs/41633910012)
  passed 15 jobs each at `d6b5f78`, including corrected Windows, MSRV, Linux,
  native/provider/C/Python/Cargo/container checks. Together with release-doc
  runs [41633902688](https://github.com/rustcommons/cargo-libc/actions/runs/41633902688)
  and [41633909999](https://github.com/rustcommons/cargo-libc/actions/runs/41633909999),
  these are the 32 implementation checks at package version 0.3.0. They are
  source-equivalence evidence, not a newly rerun 0.4.0 runtime matrix.
- **Native candidate package:** clean `6234764` package verification passed
  (10.78s), then final clean `d335f8e` package verification (9.91s) and the
  corrected official-0.3.0 payload/registry audit passed. Full archive inventories,
  VCS identities, manifest/lock/legal/README checks and source comparisons were
  retained. No official cargo-libc 0.4.0 registry archive exists in this evidence.
- **Clean unpacked installation:** locked fetch (2.08s) and offline installation
  (22.37s) passed; all five helpers are installed and version reports 0.4.0.
- **Installed C/CMake smoke:** passed (16.86s), with target 2.17 headers and host
  proc-macro C headers 2.39, actual app execution under the local 3.10 filter,
  direct inspection (highest required GLIBC_2.16), provider verification/stdout
  regression and target C++ refusal. This exercises installed linker/runner/C
  adapters and the native host/target boundary for the named fixture.
- **Installed capability checks:** 13 recorded cases passed (0.74s). A compiled
  native ELF/readelf control supplies a genuinely missing PT_INTERP. Help/version
  and text/JSON inspect succeed with PATH set to a nonexistent tool directory;
  source inspection confirms those routes do not gain host/tool probes. Direct
  and local missing-interpreter cases have empty stdout and status 2; a closed
  diagnostic pipe preserves 2. A recording engine receives the selected container
  invocation and preserves 17 without a local-loader refusal. Doctor reports its
  mock route failure as 2. Native host C and C++ wrapper compilations pass.
- **Installed streams:** passed (18.49s) with the same clean installation and
  `d335f8e` harness. Run/test preserve exact JSON stdout, separate diagnostics,
  redirected stdin and deliberate exit 7; closed pipes and other output failures
  retain expected results. Direct libc-runner redirects and status passed too.
  Optional real-container checks were not enabled.

Independent scoped package/install review approved `d335f8e` and the supplied
phase-three hashes/results. The later excluded evidence head still needs its own
narrow final review; no merge or release authorization is inferred.

## Retained proof identities

These native proof bundles were supplied separately, audited by content, and
remain outside the package. No GitHub package-qualification run ID is invented.

| Bundle | Bytes | SHA256 | Contents |
| --- | ---: | --- | --- |
| `qualification040-phase1-native-proof.tar.gz` | 12816820 | `c2f67eba4ba792d5f83168a77a19da38598ecf1948e83387dc40f381ca28f1da` | 623 package, six dependencies/official baseline, clean install/raw metadata/logs, installed smoke and five binaries; all 30 listed member hashes verified. |
| `qualification040-phase2-native-proof.tar.gz` | 165058 | `7e2dd77093f0b1dcee60e34e7265d3aa06707227443b49ec86004f08220be2b5` | d335 package/re-audit and all 13 capability cases/raw outputs; all 42 listed member hashes verified. |
| `qualification040-phase3-stream-proof.tar.gz` | 1755 | `1924fd8d68a50501b6a9f850732893a9c94eebb03e8c85737d1c4cfcf262796b` | Installed stream command/result/raw log; all three listed member hashes verified. |

## Prepublication limits and completion boundary

The recording engine proves **container routing only**; it never starts an image.
No new actual-container package-execution claim, real musl/static-host
qualification, LLVM replacement/interoperability qualification, universal GCC ABI,
dependency/plugin/load-path closure or old-OS qualification is made. Existing
source CI container checks retain their original bounded scope. Provider source
and container preparation both require host GCC/binutils/startup files; container
execution supplies userland but shares the host kernel. Tier 3 and prebuilt legal
gates remain unchanged.

Only excluded evidence/notes/index changes follow the tested candidate. Verify
source/package-input and harness equality to `d335f8e` during narrow review.
Those candidate-stage next steps are superseded only by the separately recorded
publication evidence below. No source/package/install run was repeated merely
for an excluded evidence-only update.


## Publication and installed verification

The authorized publisher completed `cargo +1.90 publish --locked` from exact
clean merge `aa2ea56e5d7943aeb4e0023713a089693d2103c1`, exit 0. Independent
verification downloaded the official archive and matched its full SHA256 to both
the crates.io version API and sparse index; version 0.4.0 is non-yanked.

| Published identity | Verified value |
| --- | --- |
| Official archive | 73,212 bytes / 26 files |
| SHA256 | `7c6177bbd11ddc43cf2e800ea2b30050eeaf52942eebbec4e531dcbc60ba6e0b` |
| Clean VCS / source commit | `aa2ea56e5d7943aeb4e0023713a089693d2103c1` |
| Annotated tag | [`v0.4.0`](https://github.com/rustcommons/cargo-libc/tree/v0.4.0), tag object `fbb0382d0fbbc02a185355e7cf18329c57b99aaa`, peels to the exact release commit |
| Release-commit docs CI | [41661380882](https://github.com/rustcommons/cargo-libc/actions/runs/41661380882), success at `aa2ea56` |
| Docs.rs | [API](https://docs.rs/cargo-libc/0.4.0/cargo_libc/), [actual README](https://docs.rs/crate/cargo-libc/0.4.0), [build 5140751](https://docs.rs/crate/cargo-libc/0.4.0/builds/5140751), HTTP 200 |

Every source, manifest, lock, README and legal-file byte equals the released
checkout. Comparing the entire official payload with native-tested `d335f8e`
finds only `.cargo_vcs_info.json` different; normalized manifest and all package
payload names are unchanged. The published archive is also byte-equal to the
clean merged package verified by the publisher. No dependency/checksum or raw-data
change was introduced. The tag did not exist locally/remotely before creation;
it was created annotated at the release commit and never moved.

A real browser verified the actual docs.rs README, six package images including
the banner/badges, and absolute guide navigation. API/build/readme pages and the
actual README guide, release-policy and banner targets returned HTTP 200.
Docs.rs uses its own documentation toolchain; it is not the Rust 1.90 MSRV gate.

### Separate published registry installation

A fresh Cargo home and isolated root on native x86_64 WSL/glibc 2.39 used:

```sh
cargo install cargo-libc --version 0.4.0 --locked --root <isolated-root>
bash scripts/smoke-installed.sh <isolated-root> /usr/bin/clang-18 0.4.0
python3 scripts/check-installed-capabilities.py   <isolated-root>/bin /usr/bin/clang-18 <capability-evidence>
python3 scripts/check-run-streams.py <isolated-root>/bin /usr/bin/clang-18
```

Rust 1.90, two build workers, one heavy job at a time; the install took 21.68s,
smoke 14.13s, capabilities 0.31s and streams 17.34s. All four phases exited 0.
All five installed executable hashes were verified. The recorded driver creates
a new work directory/Cargo home, installs registry 0.4.0 rather than a source
path, removes conflicting test overrides and then exercises that installation.

The smoke verifies source provider2.17, real C/CMake target headers and native
host proc-macro separation, actual execution under local 3.10 simulation, inspect,
provider stdout and target C++ refusal. All thirteen capability cases passed:
no-probe help/version/inspect, missing local/direct PT_INTERP and closed diagnostic
pipe status 2, recording-engine container routing/doctor failure semantics and
host C/C++ wrapper compilation. Run/test/direct-runner stdin, exact JSON stdout,
separate diagnostics, intended failure statuses and output-error controls passed.

Published proof: `published040-native-proof.tar.gz`, **7,369,489 bytes**, SHA256
`ba9c39009ca306758ad931607ccc0fd47bb9543190f625558808a26c597adec0`;
all **44** member hashes, the official archive and five installed executable
hashes were independently checked. The
[publication identity record](evals/2026-11-03-diagnostics-publication.json) retains
archive/payload/binary identities, tool scope and result summaries.

This is native recorded verification, **not a hosted published-install run**.
The container fixture uses a recording engine; no real container matrix was
rerun. Real musl/static hosts, actual old OS/tier3, LLVM/runtime replacement,
full GCC/dependency/plugin/load-path compatibility and general execution safety
remain unqualified. System unwinder and all existing legal/qualification gaps
are unchanged. No credential provisioning or crate republication occurred.

### Automatic post-merge CI infrastructure outcome

[Automatic main CI 41661380958](https://github.com/rustcommons/cargo-libc/actions/runs/41661380958)
at exact `aa2ea56` ended with nine successful jobs and six failures during
`install-clang.sh` compiler setup. Every failed job log independently shows
`curl: (6) Could not resolve host: apt.llvm.org` before its product-test step.
The affected jobs are Clang24 linking, source providers, container provider
preparation, Python extensions and two Cargo-command release groups. No actual
product failure was reported in those failed jobs; their tests did not run.

This automatic post-merge run is **not claimed green** and is retained as
infrastructure evidence. The approved 32-check implementation refs, separately
qualified native package/install and actual published-registry installation
above are the applicable named evidence. Release-commit docs CI 41661380882
passed. The setup-only failure does not justify changing product source or
rerunning unchanged broad matrices; no such rerun was requested or performed.

### GitHub Release completion

The canonical body is the exact committed `docs/releases/v0.4.0.md`, used with
existing annotated tag `v0.4.0`, stable/latest status and zero assets after the
publication documentation PR is reviewed, checked and normally merged. This
record does not pre-claim a successful future API check.

Completion verifies the live release's tag, exact body, latest status and asset
count, then runs the registered release-doc workflow with `verify_published=true`
and the equivalent full command:

```sh
python3 scripts/check-release-docs.py --repo rustcommons/cargo-libc --published
```

The final run/result links are retained in the publication bookkeeping PR and
[release-doc workflow history](https://github.com/rustcommons/cargo-libc/actions/workflows/release-docs.yml).
The earlier local-only docs CI is not relabeled as published-mode verification.
No registry upload, source/tag movement, broad matrix rerun or package change is
part of that documentation completion.
