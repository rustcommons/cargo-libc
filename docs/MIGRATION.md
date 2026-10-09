# Migration from cargo-glibc

The source directory contained 22 files. Each was copied and compared by SHA-256
before the unmodified prototype was committed as `be7dee4`. Original source,
documentation, examples, lockfiles, and license can be recovered from that commit.
The conventions commit preceded this import.

| Original | Current |
| --- | --- |
| Package/binary `cargo-glibc` | `cargo-libc` |
| Library `cargo_glibc` | `cargo_libc` |
| Cargo command `cargo glibc` | `cargo libc` |
| Helper `glibc-linker` | `libc-linker` |
| Internal `CARGO_GLIBC_*` variables | `CARGO_LIBC_*` |
| Fingerprint cfg `cargo_glibc_profile` | `cargo_libc_profile` |
| Original validation report | `docs/PROTOTYPE-VALIDATION.md`, unchanged |

The glibc-specific `--glibc` and `--max-glibc` options retain their names. Selected
target versions below 2.17 are now rejected; historical ELF symbol versions remain
valid. The `glibc-hello` example keeps its name because it tests the glibc backend.

There is no compatibility alias for `cargo glibc`. Install from this repository
with `cargo install --path . --locked --offline`, then use `cargo libc --help`.
Both binaries must be installed together. Removing the old source folder does not
uninstall any old binaries already present in a user's Cargo bin directory.

The original source folder is removed only after the migration is tested and
committed, with a final comparison against the preserved import to ensure no new
source changes would be discarded. See [validation](../VALIDATION.md).
