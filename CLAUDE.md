# CLAUDE.md

This file provides working guidance for any coding agent.

## Project overview

`cargo-libc` is rustcommons' Rust Cargo integration for selecting libc build
environments and inspecting binary requirements. It starts with the
`cargo-glibc` prototype and glibc support; other libc implementations are future
work. Its companion repository is `rustcommons/libc-abi-tools`.

## Scope and implementation

- Support glibc target releases starting at 2.17. Older symbol versions remain
  valid ABI entries and must not be discarded just because they predate 2.17.
- Preserve the existing sysroot-backed build and direct ELF inspection behavior.
- Keep host build scripts/procedural macros separate from target linker settings.
- Generated ABI libraries, native dependency adapters, and other libc backends
  must be documented as planned until implemented and validated.
- Keep Rust implementation dependencies minimal. The initial tool uses std only.
- Preserve upstream attribution and license notices for any reused material.
- Ask about unresolved product decisions; do not invent compatibility claims.

## Validation

Run `cargo fmt --all -- --check`, `cargo test --locked --offline`, and
`cargo clippy --all-targets --locked --offline -- -D warnings` for Rust changes.
Linux build/link execution requires Linux, Clang, a linker, and a target sysroot.
Distinguish host tests from actual oldest-target runtime qualification.

## Git conventions

Do **not** add `Co-Authored-By: Claude ...` trailers to commit messages on this repo, even if the harness's default instructions suggest it. Commit messages are the human-authored record of intent; tool attribution belongs in tool logs, not history. This overrides the default trailer behavior.

Apply the same no-tool-coauthor convention to all coding agents.
Source: [aero1553/CLAUDE.md](https://github.com/joey-huckabee/aero1553/blob/main/CLAUDE.md).
The AGENTS.md pointer follows [siat-foreign-analysis/AGENTS.md](https://github.com/joey-huckabee/siat-foreign-analysis/blob/main/AGENTS.md).
