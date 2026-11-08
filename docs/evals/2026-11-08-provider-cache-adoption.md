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
unpacked-package and future published-install qualification use the same gate.
The workflow's historical published cargo-libc 0.6.0 install still uses producer
0.5.0, so it explicitly skips the new-layout gate. That test exception refuses
any other version: future release preparation must restore the default gate;
it cannot silently carry the historical exception into a new release.

The package audit is updated to compare against official cargo-libc 0.6.0:
same 28 files and source/README/legal bytes, only the intended dependency metadata
and future root-version metadata may differ. It checks all actual registry
dependency archive checksums and preserves unrelated lock entries. Registry-mode
CI expects the new graph and minimum. Local qualification is recorded below;
no hosted integration or new publication is implied.

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

## Local qualification results

Package-input commit `65f7a67f2b5053bab142315102be27954d76b457` uses only the
published registry dependency. Rust 1.90 on x86_64 Linux/WSL passed 105 library
and ten process tests, doctests (zero examples), package verification and audit.
A fresh Cargo home fetched the unpacked package's locked registry dependencies,
then `cargo install --path UNPACKED --locked --offline` installed all five
helpers. This is an unpatched candidate installation, not an installation of a
newly published Cargo version.

The first new fixture failed because it requested `--message-format=json`, which
Cargo-libc reserves for artifact checking. Production behavior was correct; that
failed attempt remains in the proof. Harness-only correction `fcb555d1966bdbabc602d2fdec03d9393bd779e4`
uses supported verbose Cargo diagnostics and artifact timestamps. It also fixes
one documentation-map row. Source, dependency and package inputs are identical;
no production rebuild or reinstall was needed for the resumed tests.

The installed-tool resume passed all sixteen generation-gate commands:
managed initial/reused/new-generation/reused states were respectively nonfresh,
fresh, nonfresh, fresh; five old files retained their hashes after forced refresh;
old saved plain-Cargo configuration and old explicit Rust/C/CMake inputs remained
runnable. Provider stdout, ordinary run/test stream contracts, capability13 and
singleton-option regressions passed against the same installed helpers. These
stream results do not claim a new Docker/Podman run.

An additional upgrade control used the actual previously published cargo-libc
0.6.0 installation to prepare a legacy producer-0.5.0 entry. The candidate refused
to treat that old version key as current (path exit 1), prepared a new 0.6.0 key,
retained the legacy descriptor bytes, and successfully ran a Rust fixture using
that explicit legacy descriptor. This checks an actual old-to-new layout/key
transition rather than only two new-format generations.

Windows Rust 1.99 owner checks passed: fmt, 100 library plus one process test,
all-target Clippy and rustdoc with warnings denied. The first offline attempt
found the new registry archive absent locally; `cargo fetch --locked` resolved
that setup prerequisite before the checks passed. Windows provider preparation
is not claimed.

Repackaging clean `fcb555d` verified 28 files, 85,711 bytes, SHA-256
`bd12e37fcfc4fe38d060caea79c446a078f6cc4e7d0376958b409729a860d685`.
Only VCS metadata differs from the installed candidate archive. Compared with
actual published Cargo 0.6.0, only manifests, lock and VCS metadata change; all
Rust source, README and legal bytes match. All six dependency archive checksums
were verified against crates.io; the unrelated graph is unchanged.

The proof packet contains both candidate archives, raw initial/resume/upgrade
logs, graph, audits, helper hashes and summaries: 297 hashed files, 390,959 bytes,
SHA-256 `c184d9c772f9f57449ae914b600fb00713ff14de48f3da49865786dfc149cec6`.
Six GitHub GFM renders, 244 local links/anchors, shell syntax, release-doc checker
and diff checks passed. Final evidence-only docs preserve package and harness
inputs. Applicable hosted checks and review remain separate; release version,
merge and publication are not performed by this record.
