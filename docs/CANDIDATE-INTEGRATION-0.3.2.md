# linux-abi-tools 0.3.2 source-candidate integration

Historical evidence for the exact revisions/version below. Current release and
registry state: [cargo-libc 0.2.2](RELEASE-0.2.2.md); current constraints and gaps:
[requirements index](REQUIREMENTS.md).

Historical evidence for the revisions below. For the revised stdout fixes, see
the superseding [combined integration report](COMBINED-STDOUT-INTEGRATION-0.3.2.md).

On 2026-10-31, [run 41238413717](https://github.com/rustcommons/cargo-libc/actions/runs/41238413717)
passed every step of the manual [candidate integration workflow](../.github/workflows/candidate-integration.yml).
This validates an isolated source candidate, not published registry adoption.

| Input | Exact commit |
| --- | --- |
| Producer: `rustcommons/linux-abi-tools` | [`d2f9abbfa56fc8352b4639f9dd35048e474bf79c`](https://github.com/rustcommons/linux-abi-tools/commit/d2f9abbfa56fc8352b4639f9dd35048e474bf79c) |
| Consumer: `rustcommons/cargo-libc` | [`27d2a672b783d774637d4ea51bad38f1612e6a23`](https://github.com/rustcommons/cargo-libc/commit/27d2a672b783d774637d4ea51bad38f1612e6a23) |

The producer was the `release/0.3.2-candidate` branch's exact commit, proposed in
[draft PR #1](https://github.com/rustcommons/linux-abi-tools/pull/1). The workflow
requires a full SHA, checks it out from that repository and verifies HEAD before
building. It has read-only contents permissions and only a `workflow_dispatch`
trigger; the workflow commit used `[skip ci]` to avoid the default matrix.

## Resolution evidence

The workflow archived the consumer into a disposable directory and appended
`[patch.crates-io]` paths there. Only that copy's lockfile was refreshed.
Downloaded artifact `candidate-integration-41238413717` contains `commits.txt`,
`metadata.json`, the disposable manifest and lock, resolved source paths,
tool versions and the check logs. Its
[artifact](https://github.com/rustcommons/cargo-libc/actions/runs/41238413717/artifacts/13056198554)
digest is `sha256:192d8e7c93b432934510e201bb97a393bd15414198c4f1e9196ee342cb90c373`.

Both Cargo metadata and the saved lock confirm exactly one resolution for each
of the following packages, all version **0.3.2**, with no registry source:

| Package | Manifest relative to producer checkout |
| --- | --- |
| `libc-abi-tools` | `Cargo.toml` |
| `kernel-abi-tools` | `crates/kernel-abi-tools/Cargo.toml` |
| `linux-abi-scan` | `crates/linux-abi-scan/Cargo.toml` |
| `linux-targets` | `crates/linux-targets/Cargo.toml` |

The recorded checkout prefix is
`/home/runner/work/cargo-libc/cargo-libc/candidate/`. The workspace's umbrella
`linux-abi-tools` package is not a cargo-libc dependency and was not added.
`glibc-abilists` and `glibc-provider-sources` remain crates.io **0.1.0** inputs;
their saved lock entries, including registry checksums, match production exactly.
Production `Cargo.toml`, `Cargo.lock` and the existing CI workflow were unchanged.

## Executed coverage

Hosted execution used Ubuntu 24.04 **x86_64**, Rust/Cargo 1.90.0, Clang 18.1.3,
GCC 13.3.0, GNU binutils 2.42, CMake 3.31.6 and host glibc 2.39. Two build workers
and two Rust test threads were configured.

- Formatting, all 69 consumer unit tests and Clippy with warnings denied passed.
  The test command included doctests (none present). The same portable checks
  also passed locally on ARM64 against the exact producer and consumer code from
  `0ef4cd9e9fec6534b84babda38929f288ea5db92`, before adding the workflow.
- `smoke-providers.sh`, restricted to **glibc 2.17 and 2.28**, passed source-provider
  preparation, object verification, linking, direct inspection, tier 1 scans,
  cached reuse, local tier 2 execution, cache verification/listing and rejection
  of `memfd_create` for 2.17.
- `smoke-c-deps.sh` passed C through `cc` and CMake, correct target headers,
  native host proc-macro C, the `memfd_create` boundary, target C++ refusal and
  the host-header comparison.
- `smoke-commands.sh` passed `clippy`, `doc`, `rustc`, local fixture installation,
  linker-override refusal, shell exports and generated Cargo TOML. Both plain
  Cargo formats rejected the too-new symbol for 2.17 and accepted it for 2.28;
  their outputs were inspected separately.
- `smoke-linux.sh` passed the existing host-sysroot path, host/target separation,
  paths containing spaces, cached artifact validation, unrelated-artifact refusal,
  JSON inspection/scanning, local execution, `check` diagnostics, runner diagnosis
  and C dependencies.

No consumer regression was found in this coverage. The final workflow check
confirmed production files were unchanged and uploaded the evidence successfully.

## Exclusions and remaining gates

This run did not exercise the full glibc matrix, Python/cdylib packaging, container
providers/runners, Windows or real old kernels/userlands. Local runs used the
host's glibc 2.39; tier 1 scans and tier 2 simulation are not tier 3 qualification.
The fixture `cargo libc install --path` check is not a registry
`cargo install cargo-libc --locked` test.

Registry adoption still requires the producer's release review and explicitly
authorized publication of the required packages. After registry availability,
review the consumer's minimum dependency requirements, refresh its production
lock deliberately, and rerun checks without source patches, including package
contents and locked installation. The current `0.3.0` requirements allow 0.3.2,
but the production lock remains on 0.3.0. A future cargo-libc release carrying
an adopted lock needs separate authorization; existing published artifacts are
not changed by this source integration. No packages, tags or releases were
published by this work.
