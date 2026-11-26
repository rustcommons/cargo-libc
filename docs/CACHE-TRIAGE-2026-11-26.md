# Candidate integration cache triage — 2026-11-26

Reviewed main: `bae69d451350aae07387167ed3a392ff2dd12deb`.
Rule: CodeQL `actions/cache-poisoning/poisonable-step`.

The read-only alert query returned 14 open CodeQL reports, all in
`.github/workflows/candidate-integration.yml`. Line numbers below refer to that
main revision. No alerts or caches were changed through the API.

## Trust and cache evidence

- The workflow has only `workflow_dispatch`, no PR, `pull_request_target`,
  `workflow_run` or reusable-workflow trigger. A maintainer can dispatch it on
  `main` while selecting an arbitrary 40-character producer commit with
  `candidate_ref`. Exact length/hex validation proves identity, not review or
  trust. Producer checkout is conditional on `integration_mode == source`.
- The `integration` job accepts `source`, `registry` and `package` modes. Only
  source mode checks out and compiles the candidate, including its build scripts.
  Registry/package-only step paths cannot execute with that checkout.
- Top-level `contents: read` and checkout's `persist-credentials: false` do not
  remove the job's separate Actions cache capability. No explicit cache action,
  save/restore key or restore prefix exists in this workflow or the checked-in
  workflows. Cargo/provider caches used by the tests are local runner directories,
  not a cross-run cache boundary.
- Nevertheless, the live Actions cache API returned 12 entries, all scoped to
  `refs/heads/main`, with keys starting `codeql-overlay-base-database-1-` for
  Python analysis. Managed CodeQL default setup is configured for Actions,
  Python and Rust, with a weekly schedule. Run `45636751636` is a managed CodeQL
  run on main; its log reports `Cache mode: write`, `overlay-base with caching`,
  and a successful overlay-base database upload. These are managed cross-run
  cache consumers/writers outside the
  checked-in workflow definitions. Absence of `actions/cache` is not sufficient
  evidence for dismissing the source-mode job exposure.
- This is potential exposure, not evidence that any existing entry was poisoned.

## Fix

Set workflow-level `cache-mode: none` in candidate integration, inherited by all
its jobs with no overrides. GitHub enforces this on scoped cache tokens: neither
restore nor save is permitted, including direct cache-service use by candidate
code. This preserves source-candidate testing on main without sharing cache
capabilities with trusted workflows. Artifacts still use their separate APIs;
permissions, action pins, modes and qualification commands are unchanged.
Managed CodeQL and release workflows are unchanged.

Reference: [GitHub's cache-mode documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#controlling-cache-access-with-cache-mode).

## Per-alert decisions

Eight reports describe the shared source-job exposure. Six report steps whose
mode conditions exclude the input-controlled producer checkout. Proposed
false-positive text is for coordinator review only; no dismissal was performed.

| Alert | Verdict | Evidence at reviewed main | Proposed dismissal text |
| --- | --- | --- | --- |
| [#1](https://github.com/rustcommons/cargo-libc/security/code-scanning/1) | Real (shared source-job exposure) | Prepare disposable consumer and source overrides (50–90) is unconditional within integration. It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#2](https://github.com/rustcommons/cargo-libc/security/code-scanning/2) | Real (shared source-job exposure) | Resolve candidate and prove sources (90–144) is unconditional within integration. It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#3](https://github.com/rustcommons/cargo-libc/security/code-scanning/3) | False positive (mode-exclusive path) | Portable consumer checks (144–157) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. | Proposed `false positive`: Portable consumer checks (144–157) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. The workflow is manual-only (`workflow_dispatch`), with `contents: read`; no candidate checkout occurs in this step's mode. This is a mode-path false positive, independent of the separate source-mode cache exposure fixed in this PR. |
| [#4](https://github.com/rustcommons/cargo-libc/security/code-scanning/4) | Real (shared source-job exposure) | Scoped source consumer checks (157–167) is source-only. It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#5](https://github.com/rustcommons/cargo-libc/security/code-scanning/5) | False positive (mode-exclusive path) | Shared provider stdout regression (167–173) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. | Proposed `false positive`: Shared provider stdout regression (167–173) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. The workflow is manual-only (`workflow_dispatch`), with `contents: read`; no candidate checkout occurs in this step's mode. This is a mode-path false positive, independent of the separate source-mode cache exposure fixed in this PR. |
| [#6](https://github.com/rustcommons/cargo-libc/security/code-scanning/6) | Real (shared source-job exposure) | Native tools and fixture dependencies (173–184) is unconditional within integration. It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#7](https://github.com/rustcommons/cargo-libc/security/code-scanning/7) | Real (shared source-job exposure) | Clock family native ELF and same-directory upgrade (184–191) is source-only. It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#8](https://github.com/rustcommons/cargo-libc/security/code-scanning/8) | False positive (mode-exclusive path) | Run and test redirected streams (191–197) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. | Proposed `false positive`: Run and test redirected streams (191–197) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. The workflow is manual-only (`workflow_dispatch`), with `contents: read`; no candidate checkout occurs in this step's mode. This is a mode-path false positive, independent of the separate source-mode cache exposure fixed in this PR. |
| [#9](https://github.com/rustcommons/cargo-libc/security/code-scanning/9) | False positive (mode-exclusive path) | Scanner and container IO boundaries (197–206) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. | Proposed `false positive`: Scanner and container IO boundaries (197–206) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. The workflow is manual-only (`workflow_dispatch`), with `contents: read`; no candidate checkout occurs in this step's mode. This is a mode-path false positive, independent of the separate source-mode cache exposure fixed in this PR. |
| [#10](https://github.com/rustcommons/cargo-libc/security/code-scanning/10) | False positive (mode-exclusive path) | Automatic providers, reuse, inspect, run and symbol refusal (206–212) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. | Proposed `false positive`: Automatic providers, reuse, inspect, run and symbol refusal (206–212) runs only in `registry` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. The workflow is manual-only (`workflow_dispatch`), with `contents: read`; no candidate checkout occurs in this step's mode. This is a mode-path false positive, independent of the separate source-mode cache exposure fixed in this PR. |
| [#11](https://github.com/rustcommons/cargo-libc/security/code-scanning/11) | Real (shared source-job exposure) | C dependencies and target C++ refusal (212–220) is source-compatible (`!= package`). It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#12](https://github.com/rustcommons/cargo-libc/security/code-scanning/12) | Real (shared source-job exposure) | Cargo commands and both plain-Cargo configuration formats (220–228) is source-compatible (`!= package`). It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#13](https://github.com/rustcommons/cargo-libc/security/code-scanning/13) | Real (shared source-job exposure) | Existing sysroot and host-target separation (228–236) is source-compatible (`!= package`). It belongs to the source-capable job that checks out an arbitrary producer commit and executes it with default-branch cache-write access on a main dispatch. | None; fix the shared job exposure, do not dismiss as false positive. |
| [#14](https://github.com/rustcommons/cargo-libc/security/code-scanning/14) | False positive (mode-exclusive path) | Registry package and clean unpacked installation (236–262) runs only in `registry` or `package` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. | Proposed `false positive`: Registry package and clean unpacked installation (236–262) runs only in `registry` or `package` mode. The input-controlled producer checkout runs only in `source` mode; these paths cannot coexist in one job execution. The workflow is manual-only (`workflow_dispatch`), with `contents: read`; no candidate checkout occurs in this step's mode. This is a mode-path false positive, independent of the separate source-mode cache exposure fixed in this PR. |

## Validation and limits

Zizmor 1.32.0 accepts the workflow and reports no findings. Actionlint 1.7.12
(the current release at review time) does not yet recognize GitHub's documented
`cache-mode` workflow key. Validate the remaining workflow with the narrow
command-line exclusion below; no repository suppression or weakened CI gate is
added:

```sh
actionlint -shellcheck= -ignore '^unexpected key "cache-mode" for "workflow" section'
zizmor .github
```

PR CI and release dry-run results must be reported separately from local checks.
Static CodeQL versions that do not model `cache-mode` may continue reporting the
fixed source-job exposure; that is not permission to dismiss it automatically.
This triage does not certify the selected producer revisions or remove the need
to review them before dispatch.
