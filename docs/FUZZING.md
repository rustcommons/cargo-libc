# Fuzzing

cargo-libc's own parsers of untrusted or external input are fuzzed with
[cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html) (libFuzzer) locally
and with [ClusterFuzzLite](https://google.github.io/clusterfuzzlite/) in CI. This is
parser robustness testing: it looks for panics, overflows, hangs and memory
errors in the parsing code. It is not x86-64 runtime qualification and does not
prove whole-process safety.

## Layout

- [`fuzz/`](../fuzz) is a separate, unpublished Cargo workspace with its own
  lockfile. Its `libfuzzer-sys` dependency (owner-approved for fuzzing only)
  never enters cargo-libc's dependency graph, `Cargo.lock` or published package.
- [`src/fuzzing.rs`](../src/fuzzing.rs) exposes the harness entry points only
  under `--cfg fuzzing`, which cargo-fuzz sets. Normal builds do not contain it.
  The entry points call pure parsers only: no SDK resolution, provider file
  access, tool probing, process execution or execution of parsed input.
- `fuzz/seeds/<target>/` holds small committed seed corpora taken from the
  existing unit-test fixtures. The `elf` seeds are not committed, so the
  repository holds no binary artifacts:
  [`fuzz/gen-elf-seeds.py`](../fuzz/gen-elf-seeds.py) (Python standard library
  only) writes them byte-for-byte as the `src/elf.rs` and `src/elf/exact.rs`
  test fixtures and refuses any output whose SHA-256 differs from the pinned
  value. Generated seeds, corpora and crash artifacts are ignored.

## Targets

| Target | Input | Parser |
| --- | --- | --- |
| `version` | glibc versions and `TARGET.GLIBC` suffixes | `Version` parsing, display round-trip, 2.17+ target validation |
| `cli_args` | NUL-separated `cargo libc` arguments, including `--target` specs | The command-line parser (parsing only; nothing runs) |
| `provider_descriptor` | `libc-abi-provider-v1` descriptor text (provider/sysroot metadata) | `Provider::parse`; paths are never opened |
| `sdk_header` | glibc `features.h` text from an SDK sysroot | `__GLIBC__`/`__GLIBC_MINOR__` extraction |
| `cargo_message` | One line of Cargo `--message-format=json` output | The bounded JSON reader that finds executables |
| `elf` | Arbitrary, usually malformed, ELF files | `inspect`'s dynamic metadata, symbol-version (Verneed/Verdef) reader and exact dynsym inventory |
| `linker_syntax` | Clang `-###` traces and GNU linker scripts | Trace quoting and linker-script tokenization |

The `elf` target asserts that the exact dynsym reader never succeeds where the
metadata reader fails. `version` asserts that every accepted version displays
and parses back to itself. Scans delegated to `linux-abi-scan` and provider
preparation in `libc-abi-tools` belong to those crates and are not fuzzed here.

## Running locally

Fuzzing needs Linux, a nightly toolchain and cargo-fuzz. CI uses the toolchain of
the pinned OSS-Fuzz builder image, `nightly-2025-09-05`; the shipped crate's MSRV
stays 1.90.

```sh
rustup toolchain install nightly-2025-09-05
cargo +nightly-2025-09-05 install cargo-fuzz --version 0.13.2 --locked
cargo +nightly-2025-09-05 fuzz list
# Generate the ELF seeds (also done by the ClusterFuzzLite build).
python3 -I fuzz/gen-elf-seeds.py fuzz/seeds/elf
# Fuzz one target for five minutes. New inputs go to fuzz/corpus/elf (ignored).
cargo +nightly-2025-09-05 fuzz run --debug-assertions elf fuzz/corpus/elf fuzz/seeds/elf -- -max_total_time=300
# Reproduce a crash artifact.
cargo +nightly-2025-09-05 fuzz run --debug-assertions elf fuzz/artifacts/elf/crash-<hash>
```

Run cargo-fuzz from the repository root. `--debug-assertions` turns on overflow
checks, as the ClusterFuzzLite build does.

## Continuous fuzzing

[`.clusterfuzzlite/`](../.clusterfuzzlite) builds the targets in the OSS-Fuzz Rust
builder, pinned by image digest, generates the ELF seeds and packages each
target's seeds.

- [`cflite_pr.yml`](../.github/workflows/cflite_pr.yml) fuzzes code changes for
  300 s on pull requests that touch `src/`, `fuzz/`, the fuzzing build or the
  manifest/lock. It is skipped for Dependabot PRs; a newer push cancels the PR's
  earlier run.
- [`cflite_batch.yml`](../.github/workflows/cflite_batch.yml) runs every target
  for 1800 s weekly (and on manual dispatch) and keeps the corpus as workflow
  artifacts.

Both upload crash reports as SARIF to code scanning (category `clusterfuzzlite`).
A crash is a real bug: fix it with a regression unit test that reproduces it,
or report it to the maintainers if the fix needs a product decision.
