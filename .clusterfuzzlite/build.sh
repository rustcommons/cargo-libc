#!/bin/bash -eu
# Builds the fuzz/ harnesses with the builder image's pinned nightly and
# packages each target's committed seeds as its seed corpus.
test "$RUSTUP_TOOLCHAIN" = nightly-2025-09-05
cd "$SRC/cargo-libc"
# Resolve exactly fuzz/Cargo.lock; fail instead of updating it.
cargo fetch --locked --manifest-path fuzz/Cargo.toml
cargo fuzz build -O --debug-assertions
release=fuzz/target/x86_64-unknown-linux-gnu/release
for target in $(cargo fuzz list); do
  cp "$release/$target" "$OUT/"
  (cd "fuzz/seeds/$target" && zip -q -r "$OUT/${target}_seed_corpus.zip" .)
done
