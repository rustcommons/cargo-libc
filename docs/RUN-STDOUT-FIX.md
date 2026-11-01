# Cargo child stdout correction

2026-11-01: F2 is corrected on `fix/run-test-stdout`, reviewed through
[PR #2](https://github.com/rustcommons/cargo-libc/pull/2). This is candidate evidence,
not a release or real-old-system qualification.

## Change and contracts

Cargo command setup/status lines now use stderr, including target probes, compiler,
linker, command and runner details. `run`/`test` leave stdout to Cargo and the
executed program or harness. Adjacent Cargo-wrapping commands use the same diagnostic
path. Explicit `doctor`, `inspect`, `env` and provider output, and build/install
inspection reports retain stdout. Wrappers already send their diagnostics to stderr;
child/helper execution and status mapping are unchanged.

The diagnostic writer handles only BrokenPipe at the write/flush boundary and
continues the operation. Genuine write errors remain failures. No global early
exit, panic suppression or error-string matching was introduced. Existing stdout
BrokenPipe tests remain in place; child output uses the child's own error handling.

## Review follow-up

The final stderr fallback now writes without panicking across all five executables.
A failed diagnostic cannot replace deliberate error 1, runner refusal 2 or a child
status with panic status 101. Setup diagnostics remain fallible; only BrokenPipe
is ignored, and genuine write errors return failure. Runner refusal diagnostics
are best effort because the refusal already determines status 2. Human `--dry-run`
command/probe lines go to stderr as well; this is part of the same patch correction.

Portable regressions assert exact statuses with both preclosed stderr and
`/dev/full`. Native coverage adds explicit local kernel-floor runs and dry-run
stream separation. Combined qualification against producer `c04269a6ea864b3a6371e8813d14f0fa069eb006`
is obsolete: review found its global record cap rejects valid large Rust binaries.
Run 41264363861 was cancelled. A corrected producer revision is required before
fresh qualification; no combined acceptance is claimed. The harness now accepts
aliased dynamic-symbol headers and official Rust toolchain binaries with fixed
tables above 65,536 entries, while rejecting overlapping version chains and offset
overflows under explicit memory/time limits. Production dependencies remain unchanged.

## Verified evidence

- Baseline `92bc2b7874a5ee23975bd04e2ea29eb306b53f00` (same affected Rust source as
  released 0.2.1) reproduced setup lines before JSON for both `run` and `test`.
- Fixed consumer `e725a00aef82b0ce653d1d5b83548e521064f6d1` passed
  [run 41263722731](https://github.com/rustcommons/cargo-libc/actions/runs/41263722731)
  on Ubuntu 24.04 x86_64, Rust 1.90.0 and Clang 18, using the existing registry lock.
- `cargo fmt --all -- --check`, locked/offline tests, all-target Clippy with warnings
  denied, docs, verified packaging and build passed. The compiler is the declared MSRV.
- `python3 scripts/check-run-streams.py target/debug /usr/bin/clang-18` checked
  exact JSON stdout and separate stderr, redirected stdin, normal success, child
  exit 7, deterministically closed stdout and `/dev/full` output errors for both
  commands. It also checked direct `libc-runner` input/output and exit status.
  Source provider 2.17 was prepared/reused. No container behavior is qualified here.
- The existing sysroot smoke now captures `run` stderr separately and checks its
  simulation message there. Its full execution remains part of combined integration.
- Actionlint, shell syntax, documentation links/rendering and diff checks passed.

Downloaded [artifact `run-streams-41263722731`](https://github.com/rustcommons/cargo-libc/actions/runs/41263722731/artifacts/13065884507)
contains the baseline/fixed results and exact consumer revision. Its recorded digest
is `sha256:f0cf2f3303720c03d0a596a10fc3031a254727bd6d6e86640edfb49375b54db0`.

[Run 41263562301](https://github.com/rustcommons/cargo-libc/actions/runs/41263562301)
is superseded: its fixture incorrectly expected Cargo to replace a harness-free
test's exit 7 with 101. The actual preserved exit 7 is correct. The fixture was
corrected; no product exit semantics changed to satisfy it.

## Release and dependency recommendation

Recommend **cargo-libc 0.2.2**, subject to combined validation and approval of the
final scope. Under [Choosing a version](RELEASING.md#choosing-a-version), correcting
stdout contamination is a compatible bug fix. A compatible internal dependency
patch minimum is not an MSRV or required native-tool floor increase; those separate
floor changes, new scope or breaking contracts would require a minor release.
There is no version change or release authorization implied by this record.

1. Record the exact producer candidate SHA containing the scanner malformed-input
   and container environment/stdin fixes. Run fresh isolated combined integration
   with this consumer, redirected streams/input and malformed scanner coverage;
   retain exact resolutions and artifacts. The manual integration workflow now
   includes the stream regression. Source patches stay disposable.
2. Publish and verify required producer versions first. Only then raise the
   `linux-abi-scan` and `kernel-abi-tools` caret minimums to published versions
   containing the fixes needed by the consumer. Review the actual dependency graph;
   do not raise unrelated minimums or glibc inputs automatically. Deliberately
   refresh the real lock and validate the registry-only graph without overrides.
3. Follow [RELEASING.md](RELEASING.md) for applicable native, MSRV, package and
   installed checks before any separately authorized merge/publication. The current
   PR changes no manifest, lockfile, dependency, package version or tag.

Duplicate-option rejection is deferred in the [roadmap](ROADMAP.md#cli-consistency-review),
not silently added to this patch. Existing signal termination status handling and
the provider cache `--force` replacement window remain optional follow-up reviews,
not additional blockers for this stdout correction. Tier 3 stays deferred with its
coverage gap explicit.
