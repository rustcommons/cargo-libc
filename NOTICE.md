# Attribution

`cargo-libc` is based on the MIT-licensed `cargo-glibc` prototype.
Its original copyright notice is preserved in [LICENSE](LICENSE), and the
unmodified import is retained in commit `be7dee4` before the command migration.

The generated ABI provider is developed in the `libc-abi-tools` crate of
[rustcommons/linux-abi-tools](https://github.com/rustcommons/linux-abi-tools).
Its glibc ABI data comes directly from the **GNU C Library**: the ABI list
files at glibc's signed release tags 2.17-2.44 in
https://sourceware.org/git/glibc.git, collected with per-release and per-file
provenance by [rustcommons/glibc-abilists](https://github.com/rustcommons/glibc-abilists)
and licensed, like glibc, under LGPL-2.1-or-later. `libc-abi-tools` depends
on that crate by default through its `bundled-glibc-data` Cargo feature; it
can opt out (`default-features = false`) and read a data directory instead
(`--data DIR`).

**Credit belongs to the Zig contributors** for the idea and approach:
[Zig's libc ABI tools](https://codeberg.org/ziglang/libc-abi-tools), its ABI
metadata design, and its glibc compatibility work. The companion repository
preserves the Zig tooling license for the code it adapted from that design.

This repository currently contains no copied Zig source or glibc ABI dataset.
It uses the caller's real sysroot; the generated provider is not integrated yet.
It is an independent project, with no implied endorsement by Zig or Rust.
