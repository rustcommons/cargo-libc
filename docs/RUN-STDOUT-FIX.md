# Cargo child stdout correction

2026-11-01: [PR #2](https://github.com/rustcommons/cargo-libc/pull/2) corrects F2 and
final stderr reporting. This record qualifies the reviewed source, not a new
published version or real-old-system compatibility claim.

## Change and contracts

Cargo command setup/status lines use stderr, including target probes, compiler,
linker, command and runner details and human `--dry-run` output. `run`/`test` leave
stdout to Cargo and the executed program or harness. Adjacent Cargo-wrapping
commands use the same diagnostic path. Explicit `doctor`, `inspect`, `env` and
provider output, and build/install inspection reports retain stdout.

The diagnostic writer ignores only BrokenPipe at the write/flush boundary and
continues the operation. Genuine write errors remain failures. Final error
reporting is non-panicking across all five executables: unwritable diagnostics
preserve deliberate error 1, runner refusal 2 and child statuses. Refusal/error
reporting is best effort once failure is determined. No global early exit, panic
suppression or error-string matching was introduced. Child output retains the
child's own error handling.

## Verified consumer checks

Baseline `92bc2b7874a5ee23975bd04e2ea29eb306b53f00` (same affected Rust source as
released 0.2.1) reproduced setup lines before JSON for both `run` and `test`.
Initial F2 correction passed [run 41263722731](https://github.com/rustcommons/cargo-libc/actions/runs/41263722731)
at `e725a00aef82b0ce653d1d5b83548e521064f6d1`.

Final product source `748500343dfc0508c09b38a0ee0e007cfda98dae` passed
[run 41264124685](https://github.com/rustcommons/cargo-libc/actions/runs/41264124685)
on Ubuntu 24.04 x86_64 / Rust 1.90.0 / Clang 18 with the existing registry lock:

- fmt, 71 unit tests, seven process regressions, all-target Clippy with warnings
  denied, docs, verified packaging and build; Rust 1.90.0 is the declared MSRV;
- exact JSON stdout and separate diagnostics for run/test, redirected stdin,
  child exit 7, closed stdout and `/dev/full` output failures;
- exact statuses 1/2/7 with closed/full stderr across owned errors, runner refusal
  and compiler children; local kernel-floor runs, dry-run stream separation and
  direct `libc-runner` input/output/status.

Downloaded [artifact `run-streams-41264124685`](https://github.com/rustcommons/cargo-libc/actions/runs/41264124685/artifacts/13066941184)
has recorded digest
`sha256:4bfb5a0e71adc94ad789e847c2b0f04e700c07504598f03381d87b55fe8374d2`.
Later candidate-harness changes did not alter this product source.

## Combined candidate qualification

[Run 41265226302](https://github.com/rustcommons/cargo-libc/actions/runs/41265226302)
**passed** with these exact revisions:

- Consumer: `192e0157063f6b2c1ab868f075eb6897b61b15f8`.
- Producer: `6604ceea74152a2e8bb1b56e5605bd4feecd8365` in
  [linux-abi-tools PR #2](https://github.com/rustcommons/linux-abi-tools/pull/2).

The downloaded metadata and lock prove all four consumed packages
(`libc-abi-tools`, `kernel-abi-tools`, `linux-abi-scan`, `linux-targets`) resolve to
paths in that exact producer checkout, still labeled 0.3.2 in the candidate
manifests. These are source candidates, not registry 0.3.3 packages. Patches and
lock refresh stayed in the disposable consumer. Both glibc inputs remain registry
0.1.0 with production checksums unchanged; production manifests/lock are unchanged.

Executed coverage on Ubuntu 24.04 x86_64 / Rust 1.90.0 / Clang 18:

- Portable consumer checks and shared provider closed-stdout/later-error regression.
- Separate run/test stdout/stderr, redirected stdin, dry-run, local kernel-floor
  runner and preserved failure statuses.
- Docker and Podman: cargo run and test receive redirected stdin, spaced CARGO/RUST
  test variables and produce exact JSON stdout; unrelated test variables are not
  forwarded, and child exit 7 is retained. Tests use the SLES 12 SP5 userland with
  a kernel simulation on a modern host, not an actual old kernel.
- Scanner entry paths (`--scan`, `--kernel`, `--distro`): offset overflows and reused
  version-chain records are rejected; aliased dynamic-symbol headers complete
  successfully, under 512 MiB/10-second process limits.
- Official Rust 1.90 Cargo and librustc_driver full scans accept fixed tables of
  79,207 and 146,741 entries respectively, under 1536 MiB/60-second limits.
- Bounded source providers 2.17/2.28, reuse/inspection/symbol refusal, C/CMake and
  C++ refusal, Cargo commands, both plain-Cargo config formats, sysroot builds and
  host/target separation. The sysroot smoke checks simulation diagnostics on stderr.

Downloaded [artifact `candidate-integration-41265226302`](https://github.com/rustcommons/cargo-libc/actions/runs/41265226302/artifacts/13067083108)
contains exact refs, resolutions, disposable manifest/lock and test logs. Recorded
digest: `sha256:379cc82b3f5e9f38975ac1721b9730fa8f5235795fababaf15fa53df61a60456`.

Earlier runs are superseded: 41263562301 expected 101 instead of the correct
harness-free test exit 7; 41264127122 incorrectly expected rejection of unselected
aliased symbol-table headers; 41264363861 was cancelled after the producer's valid
large-table rejection was identified; 41264799864 passed scanner checks but its
fixture leaked plain-runner environment overrides into managed Cargo, correctly
refused by the product. Run 41265169357 was cancelled to make environment isolation
explicit. No product semantics were weakened to satisfy these fixtures.

## Unreleased adoption plan

Recommend **cargo-libc 0.2.2** under [Choosing a version](RELEASING.md#choosing-a-version):
these are compatible bug corrections. A compatible internal dependency patch
minimum is not an MSRV or required native-tool floor increase. New scope, breaking
contracts or those tool-floor increases would instead require a minor release.

1. Publish and verify all five producer **0.3.3** packages first, following the
   producer's separately authorized release process; verify archives, source and
   index availability. No 0.3.3 registry adoption is claimed here.
2. Raise `linux-abi-scan` and `kernel-abi-tools` caret minimums to `"0.3.3"` because
   the consumer relies on their fixes. Deliberately refresh the real registry lock,
   preserving unrelated minimums and glibc inputs. Cargo-libc remains independently
   versioned; no automatic matching of producer versions is required.
3. Validate the registry-only graph without overrides, applicable native/MSRV/
   package checks and clean unpacked-package installation before a separately
   authorized cargo-libc 0.2.2 publication; verify clean published locked installation
   afterward. Source-candidate evidence is not a substitute for these gates.

This work changes no package version, production dependency, lock or tag and does
not publish a release. Duplicate-option rejection remains deferred in the
[roadmap](ROADMAP.md#cli-consistency-review). Signal termination status and provider
cache `--force` replacement remain optional follow-up reviews. Broad unchanged
provider/Python matrices were not repeated; tier 3 remains deferred with its
coverage gap explicit. Container images must be trusted: forwarded Cargo variables
can include registry credentials; the fixture prints no environment dump or secrets.
