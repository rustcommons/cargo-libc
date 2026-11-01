# cargo-libc 0.2.4 release preparation

Status: authorized minimal documentation-only candidate, **not published**.
Latest verified published version is 0.2.3. Current states and dated records live
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
before packaging; postpublication checks must follow the exact docs.rs README's
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
