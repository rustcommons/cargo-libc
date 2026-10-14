# Roadmap

Open work only. What cargo-libc does today is in the [user guide](USER-GUIDE.md), and
what was tested is in [VALIDATION.md](../VALIDATION.md). Candidate features are
inventoried in the [feature review](FEATURE-REVIEW.md); listing one there is not
approval.

## Next

In this order; each item says what it waits on.

1. [ ] **Plain-Cargo refusal test.** CI shows that `cargo libc build` refuses a symbol
   newer than the target (`examples/too-new` calls `memfd_create`, glibc 2.27, and
   must fail to link for every release before 2.27). Add the same case for plain
   Cargo configured by `cargo libc env` to `scripts/smoke-commands.sh`. Both routes
   link through `libc-linker`, so it should pass; the test makes that evidence rather
   than an assumption. Needs no decisions.
2. [ ] **Tier 3 on real old systems.** For example CentOS 7 (glibc 2.17, Linux 3.10)
   and SLES 12 SP5 (glibc 2.22, Linux 4.12), for executables and Python extensions,
   including native C dependencies; and a real Rocky Linux 9 build host. This is the
   main missing evidence behind the releases. Waits on the maintainer providing or
   approving the hosts or VMs.
3. [ ] **Maintainer decisions:**
   - a release and versioning policy;
   - the [feature review](FEATURE-REVIEW.md): approve, defer, reject or ask for
     evidence on each item, starting with core delivery and plain Cargo, then native
     dependencies, Cargo conveniences and link modes, then platform expansion; record
     ownership, prerequisites, maintenance cost and acceptance evidence for approved
     items;
   - the legal review, which gates `prebuilt` mode and `glibc-provider-binaries`.

   Revisit the name before adopting general toolchain/SDK management or platform
   packaging as a primary role.

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
