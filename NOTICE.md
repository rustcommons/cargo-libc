# Attribution

`cargo-libc` is based on the MIT-licensed `cargo-glibc` prototype.
Its original copyright notice is preserved in [LICENSE](LICENSE), and the
unmodified import is retained in commit `be7dee4` before the command migration.

The future generated ABI provider is being developed in
[rustcommons/libc-abi-tools](https://github.com/rustcommons/libc-abi-tools) as a
Rust port of the glibc portions of
[Zig's libc ABI tools](https://codeberg.org/ziglang/libc-abi-tools).
**Credit belongs to the Zig contributors** for their ABI collection tools,
metadata design, and glibc compatibility work. The companion repository records
the exact upstream commit and preserves both the Zig tooling license and the
separate license supplied for glibc ABI data.

This repository currently contains no copied Zig source or glibc ABI dataset.
It uses the caller's real sysroot; the generated provider is not integrated yet.
It is an independent project, with no implied endorsement by Zig or Rust.
