# Roadmap

Open work and adopted policy decisions. What cargo-libc does today is in the [user guide](USER-GUIDE.md), and
what was tested is in [VALIDATION.md](../VALIDATION.md). Candidate features are
inventoried in the [feature review](FEATURE-REVIEW.md); listing one there is not
approval.

[cargo-libc 0.2.1](RELEASE-0.2.1.md) is published and its registry installation,
release tag and docs.rs documentation are verified.

## Release policy adopted

- [x] [Release and versioning policy](RELEASING.md), adopted 2026-11-01.
  Cargo-libc versions remain independent of dependency versions.

## Maintainer decisions

- [ ] The [feature review](FEATURE-REVIEW.md): approve, defer, reject or ask for
  evidence on each item, starting with core delivery and plain Cargo, then native
  dependencies, Cargo conveniences and link modes, then platform expansion; record
  ownership, prerequisites, maintenance cost and acceptance evidence for approved
  items.
- [ ] The legal review, which gates `prebuilt` mode and `glibc-provider-binaries`.

Revisit the name before adopting general toolchain/SDK management or platform
packaging as a primary role.

## Deferred qualification and infrastructure

- [ ] **Tier 3 on real old systems.** As decided on 2026-11-01, these environments
  are unavailable and the maintainer cannot provide or run them. This work is
  deferred, not complete and not an ordinary-release blocker. Releases may ship
  with applicable automated gates passing and an explicit tier 3 coverage gap,
  under the [release policy](RELEASING.md#validation-and-deferred-qualification).
  Any new real-old-system compatibility claim still needs actual evidence.

  Potential qualification includes old distribution/kernel combinations,
  executables and Python extensions with native C dependencies, and relevant
  build hosts. Tier 1 scans, tier 2 simulation and containers on modern kernels
  do not supply that evidence. A full-VM approach is an investigation only; no
  new CI platform, user hardware, funding or delivery is committed.

## Providers and artifact checks

- [ ] Resolve providers through an authenticated inventory (relocatable SDKs, or the
  `prebuilt` binaries after the legal review), with immutable package identity; the
  build fingerprint does not yet hash every provider file.
- [ ] Check every executable a workspace build produces, not only the one named with
  `--artifact`.
- [ ] Validate the dynamic-library closure and runtime search paths.
- [ ] Merge flags: today `RUSTFLAGS` and `CARGO_ENCODED_RUSTFLAGS` replace cargo-libc's
  settings instead of combining with them.

## Later features

After the above:

- C++ dependencies (a target C++ runtime; `libc-cxx` refuses them now);
- pkg-config and host system libraries;
- finding imported providers regardless of the importing machine's GCC and binutils;
- distribution-specific providers (planned in
  [linux-abi-tools](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/ROADMAP.md#future-release-distribution-specific-providers),
  with [measured differences](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/DISTRIBUTION-DIFFERENCES.md));
- PIE executables (F31);
- other library outputs (F11: `staticlib`, `dylib` policy);
- build hosts other than Linux x86_64;
- other architectures (aarch64 first) and C libraries (musl), each once its baseline
  and static/dynamic linking behavior are defined.

## Presentation

- [ ] A raster social preview image for GitHub, if wanted; the README uses the SVG
  banner directly.
