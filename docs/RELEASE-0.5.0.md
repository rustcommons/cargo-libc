# cargo-libc 0.5.0 candidate release record

Prepared 2026-11-03, not published. Latest published cargo-libc remains 0.4.0.
Only release metadata/docs are being prepared around the reviewed source; no
checker or runtime behavior is changed. [User-facing notes](releases/v0.5.0.md)
and [contract](TARGET-LIBGCC.md) describe the option/status/JSON and memory bounds.

## Reviewed source evidence

Implementation `972783dfd1571fb8e23385b88d147ef42c7c6dfe`, code-equivalent
PR20 documentation head `cf5e49efd09a1a61f498f7602ada059eb777e683`.
Producer hash API source `2c70e47954f328e55950d738d5a9295de7c44d67`.
The [dated source record](evals/2026-11-03-target-libgcc.md) preserves independent
review, 102 library/nine process/36 native GNU static cases, Rust 1.90 tests/build/
docs and pinned 1.99 fmt/Clippy, plus byte-equal legacy inspect against released 0.4.0.

The accepted final source packet is 16,028,453 bytes, 150 member hashes,SHA256
`0b7cd40fbf354141bde0d10205f53358685be26b25dd4e2e7880dab6fecc7ef8`.
Those tests use a disposable first-party source patch and are not real-registry
qualification. The source package contents audit used `--no-verify`; it is not
unpatched package verification. LLD/Windows were not qualified in that native run.

## Registry and final candidate gates

Producer `ALL_FIVE_AVAILABLE` 0.5.0 was confirmed at exact merge
`bc7fcb35ea86409a2a9f543b8d25b65ed321cf47`. Independent official libc-abi-tools
archive/API verification matched SHA256
`c598bccdec99a399cc8ea639c69ffd59a58dd3dc7904ea5cc586afd01dcea469`,
non-yanked state, clean exact VCS and public `content_sha256`. Only its direct
minimum and registry lock entry change to 0.5.0. Metadata resolves exactly seven
packages: root0.5.0, libc-abi-tools0.5.0, kernel/scan/targets0.3.3,
glibc-abilists0.1.2 and glibc-provider-sources0.1.1. All six dependencies are registry
sources; the other five dependency lock entries/checksums are unchanged. There
are no production path/git patches or source replacements.

Next gates: actual unpatched registry tests/MSRV/fmt/Clippy/docs, permanent static
checker fixtures and legacy comparison, official 0.4.0→0.5.0 payload audit, clean
exact VCS package verification, clean-home unpacked locked installation, all five
helpers and affected installed checker/C/CMake/stream/host-target smokes. These
are not claimed passed yet. The package auditor now uses official 0.4.0 checksum
`7c6177bbd11ddc43cf2e800ea2b30050eeaf52942eebbec4e531dcbc60ba6e0b`
and allows exactly the reviewed new checker modules/common source changes, root
version and root producer version/checksum. README/legal/linker/unwinder inputs
and unrelated dependencies must remain equal.

The producer version change refreshes cache identity; fresh provider preparation
may be needed. No automatic import/offline migration is promised. No source/data,
clock, linker or unwinder policy change follows. Read limits are not whole-process
memory bounds; no safe-to-run, target-loader selection or full dependency/old-OS/
LLVM interoperability qualification is claimed.

Normal merge requires final candidate review and passing applicable gates. Sole
publisher is the local coordinator. No publication, tag or GitHub Release is
performed by release preparation; official archive/docs.rs/published-install and
release-documentation completion remain later steps after publication signal.
