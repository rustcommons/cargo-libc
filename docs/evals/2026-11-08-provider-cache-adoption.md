# Provider-cache 0.6.0 adoption (2026-11-08)

## Scope and status

Unreleased consumer update, not a new Cargo publication. The root version remains
0.6.0 until separate release preparation; the official cargo-libc 0.6.0 archive
is unchanged and still uses the older producer. The producer's published
[0.6.0 record](https://github.com/rustcommons/linux-abi-tools/blob/01676c8eec800d9339058d31f4c3f9e53cdd0fa6/docs/RELEASE-0.6.0.md)
and [source tag](https://github.com/rustcommons/linux-abi-tools/tree/v0.6.0)
are independently verified. This adoption raises the actual dependency minimum,
not just a development lock or source override.

## Dependency and behavior contract

Only the `libc-abi-tools` requirement changes, 0.5.0 to 0.6.0; the lock changes its
version/checksum to registry SHA-256
`368579c7f6401ead63390c8c273da9d9840a905438b62e603917bb169e084d6e`.
Kernel/scanner/targets remain locked at 0.3.3, with existing minima. Glibc lists
stay at 0.1.2 and provider sources at 0.1.1, with identical checksums. No path,
git, source replacement or patch override is used. Rust/native floors, selected
clock profile, system unwinder and target/host separation are unchanged.

Cargo already uses `Prepared.entry.descriptor()` and fingerprints the actual
provider descriptor/sysroot paths. No producer cache algorithm is copied into
Cargo. New automatic selection therefore sees a new immutable generation and
changes the target configuration; old explicit/saved inputs keep their generation.
Malformed current selection metadata refuses without legacy fallback. The
delegated layout/refusal contract and release version need review under
[Cargo's version policy](../RELEASING.md#choosing-a-version); producer and consumer
numbers are independent. No release version is assigned here.

## Permanent qualification

[check-provider-cache.py](../../scripts/check-provider-cache.py) runs against
already-installed tools: initial managed build/reuse; saved plain-Cargo config;
forced generation refresh; held descriptor/manifest/header/startup/stub hashes;
new managed fingerprint rebuild and reuse; old saved config execution; explicit
old-descriptor Rust and native C/CMake; current provider verification. It writes
bounded command logs and a JSON summary. `smoke-installed.sh` includes it, so
unpacked-package and actual published-install qualification use the same gate.

The package audit is updated to compare against official cargo-libc 0.6.0:
same 28 files and source/README/legal bytes, only the intended dependency metadata
and future root-version metadata may differ. It checks all actual registry
dependency archive checksums and preserves unrelated lock entries. Registry-mode
CI expects the new graph and minimum. Candidate source/native/package evidence
is pending; none of the proposed checks is yet claimed passed here.

## Migration and limits

The producer version participates in preparation keys, so upgrading can prepare
providers again. Old/imported version-keyed entries are not silently reused.
Source mode still needs host GCC/binutils/startup files; container mode additionally
needs its image/engine and possibly archives/network. External GCC startup files
are not immutable snapshots of a generation. Saved settings do not switch until
regenerated. Retained paths survive preparation, not explicit removal or edits.
Stop readers/writers before destructive removal; no automatic reclamation,
authenticated cache, crash durability or network-filesystem transaction is promised.
Native modern-host tests do not establish real old-system, musl/static-host or
new container compatibility. LLVM research remains held and unwired.
