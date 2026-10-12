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
metadata design, and its glibc compatibility work, as of commit
`9217cf899bf12bbb24a8a3de4cc2de682de0379d` ("update to Zig 0.17.0",
2026-10-02). The companion repository preserves the Zig tooling license for the
code it adapted from that commit; its
[NOTICE](https://github.com/rustcommons/linux-abi-tools/blob/main/NOTICE.md)
holds the full attribution record.

This repository contains no copied Zig source or glibc data of its own.

## glibc data in the built tool

Since 2026-10-12 cargo-libc depends on `libc-abi-tools` for providers, with its
default features, so the `cargo-libc` executable compiles in:

- glibc's ABI lists, from glibc's release tags through
  [glibc-abilists](https://github.com/rustcommons/glibc-abilists), under the GNU
  Lesser General Public License, version 2.1 or later;
- glibc's recorded source files and headers, and the Linux UAPI headers of the
  builders, through
  [glibc-provider-sources](https://github.com/rustcommons/glibc-provider): glibc's
  files under LGPL-2.1-or-later, each UAPI header under its own license (mostly
  GPL-2.0 WITH Linux-syscall-note).

Providers prepared from them on the user's machine keep those licenses. Those
crates' NOTICE files hold the full provenance.
It is an independent project, with no implied endorsement by Zig or Rust.
