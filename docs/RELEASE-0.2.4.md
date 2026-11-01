# cargo-libc 0.2.4 release preparation

Status: authorized minimal documentation-only candidate, **not published**.
Registry/package/clean-install gates passed; independent review/merge and
publication remain pending. Latest verified published version is 0.2.3. Current states and dated records live
in the maintained [release index](RELEASING.md#release-records).

## Correction and scope

The 0.2.3 package fixed guide/banner paths but still called 0.2.2 evidence the
"Current release". Version 0.2.4 replaces exactly that README line with the
version-neutral **Release policy and records** link to maintained
`docs/RELEASING.md`. That target records dated versions and makes the latest
verified publication and unpublished candidate distinct. No pinned historical
version is labeled current in the new package README.

The root package version and its own lock entry change to 0.2.4. All other
README text and every Rust source/API, example, product script/test, dependency
version/checksum, ABI/data/cache/collector payload and direct dependency floor
remain identical to 0.2.3. Existing workflow version arguments/assertions target
0.2.4; its package comparison uses the official 0.2.3 archive and also requires
lock equality apart from the root entry. No new CI mode, automation or dependency
is added. Old release records, packages and tags stay immutable.

## README audit before candidate packaging

On 2026-11-01, the complete prospective packaged README was compared with
`v0.2.3:README.md`: its only text delta is the release-navigation line. Every
label was reviewed against unchanged source/manifest/CI scope; the generic 0.2
series and Rust 1.90 labels remain accurate and no patch-version label remains.

All 30 absolute destinations were checked, including guide/reference anchors;
all six image URLs return SVG. Relative LICENSE and NOTICE links name included
package files. Browser inspection confirms the actual crates.io cargo-libc page
(the simple HTTP client returned 404) and the actual Codeberg Zig/glibc design
README (its basic HTTP response required browser rendering). No failing product
check was bypassed and no existing valid public URL was changed to mask a client
limitation. The new policy/index Markdown and local record links are checked
before packaging; the [complete audit](assets/release-0.2.4/readme-prepackage-audit.json)
records the README hash and individual link/image results. Postpublication checks
must follow the exact docs.rs README's
release-index href and verify actual rendered content.

## Scoped release gates

- Exact Rust 1.90 candidate: fmt, MSRV/all-target check, unit/process/doctests,
  Clippy, docs and package validation; no feature or architecture investigation.
- Compare package paths and Rust/license/notice payload with official 0.2.3,
  original manifest allowing only the root version change, and lock allowing only
  its root version entry. Require the new stable release link and absence of the
  old current-release label in the packaged README.
- Clean unpacked locked installation with expected version and all five helpers.
  The existing manual registry flow supplies this gate without adding a mode or
  bypassing failures; its retained bounded native checks may also execute.
- [0.2.3 qualification](RELEASE-0.2.3.md) remains named unchanged-implementation
  evidence for providers/C/CMake/containers/scanners (registry 41274081431 and
  published-install 41274796646). This patch makes no broader runtime claim.
- Independent exact-candidate review, normal merge approval, immutable merged
  release identity, and publication confirmation before any tag.
- After publication: official checksum/VCS, docs.rs actual README/index link and
  banner render, exact tag, clean published locked install and final record.

Direct floors stay scanner/kernel 0.3.3, libc-abi-tools 0.3.2 and linux-targets
0.3.0. The lock keeps four tools 0.3.3, glibc-abilists 0.1.2 and
`glibc-provider-sources` 0.1.1, with unchanged registry checksums. No source/path/
git overrides, third-party dependency or new requirement ID is introduced.
Tier 3 remains deferred, not an ordinary-release blocker; prebuilt legal review
and all existing coverage limits remain unchanged.

## Exact-candidate qualification

[Run 41275479293](https://github.com/rustcommons/cargo-libc/actions/runs/41275479293)
**passed** at exact candidate
`896ac7b2c043cbae971e4de86756e5214a7327b7` on Ubuntu 24.04 x86_64, Rust/Cargo
1.90.0 and Clang 18. Registry mode used no source checkout/patch/override and
asserted the exact unchanged dependency versions, checksums and direct floors.

Passing required gates: fmt, MSRV/all-target check, 71 unit tests, seven process
regressions, doctests, all-target Clippy and docs with warnings denied, verified
package and clean unpacked locked installation with expected 0.2.4 version and
all five executables/helpers. The existing flow also retained and passed its
bounded provider/reuse/inspection/refusal, C/CMake, both plain-Cargo formats,
host-sysroot/host-target, stream/stdin/environment/status, scanner/real-ELF and
Docker/Podman checks. This does not expand the qualified implementation scope.

Downloaded [artifact 13070383933](https://github.com/rustcommons/cargo-libc/actions/runs/41275479293/artifacts/13070383933)
contains exact refs, metadata, manifest/lock, logs, tested package and baseline
payload proof. Its downloaded ZIP matches GitHub's SHA-256 digest:
`db61d16093b539b4f0002bb25ca8937b033ece6fdae003efed8fcad8096e62df`.
Tested native crate SHA-256:
`9ae29ad2e7d4619c9bc775c286cd9299cfaf76ebf1f426ffef27c17d93cad491`.
This disposable crate is candidate evidence, not an official published archive.

The package proof compares with official 0.2.3 archive SHA-256
`cec36f14679064031545748420e97b3fbadea811be7b504177f6b25c1dfd585a`.
All package paths (apart from optional VCS metadata), Rust/license/notice bytes
and original manifest fields match, allowing only the package version in the
manifest. Parsed lock equality allows only the root 0.2.3-to-0.2.4 entry; every
dependency entry/checksum is identical. The packaged README matches the complete
prepackaging label/link/image audit hash and expected neutral release link, with
no old current-release label. Its sole text delta is the requested navigation
line. Candidate README, manifest/lock and Rust payload were independently
compared with the downloaded tested package.

The final evidence update changes only excluded release/index/trace/roadmap
documents and the audit JSON. All source, README, manifest/lock, workflow and
package inputs remain identical to tested `896ac7b`. No full suite was repeated
for these records. Independent final approval and normal merge remain pending;
no publication or tag is claimed. Prior 0.2.3 snapshots and evidence stay intact.
