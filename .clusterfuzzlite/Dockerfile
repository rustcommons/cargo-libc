# OSS-Fuzz Rust builder, pinned by digest (latest on 2026-11-18). It provides
# cargo-fuzz and the fuzzing toolchain nightly-2025-09-05 (RUSTUP_TOOLCHAIN);
# build.sh fails if that changes. The shipped crate's MSRV stays 1.90.
FROM gcr.io/oss-fuzz-base/base-builder-rust@sha256:3428d0a90b1e7efbe2ba947bd1d0c9a70594986b4b6553804547f1fb24cfc92f
COPY . $SRC/cargo-libc
WORKDIR $SRC/cargo-libc
COPY .clusterfuzzlite/build.sh $SRC/
