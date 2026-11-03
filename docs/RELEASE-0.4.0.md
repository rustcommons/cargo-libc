# cargo-libc 0.4.0 candidate qualification record

2026-11-03. **Candidate package/installation qualification passed within the
scope below; not published.** Final evidence-head review, normal merge,
publication, official archive/docs.rs verification, published installation,
immutable tag and GitHub Release remain pending and are not authorized by this
record. Latest verified published release remains 0.3.0.

[User-facing candidate notes](releases/v0.4.0.md), [changelog](../CHANGELOG.md),
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

## Limits and remaining completion steps

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
Normal merge, any final-commit packaging/VCS audit, publication, official
archive/checksum/non-yanked verification, tag, actual docs.rs rendering,
published clean installation and GitHub Release completion remain separate and
pending. No source/package/install run was repeated for this evidence-only update.
