# Roadmap

Open work only. What cargo-libc does today is in the [user guide](USER-GUIDE.md), and
what was tested is in [VALIDATION.md](../VALIDATION.md). Candidate features are
inventoried in the [feature review](FEATURE-REVIEW.md); listing one there is not
approval.

## Next

In this order; each item says what it waits on.

1. [ ] **Broken pipe, then 0.2.1.** Piped into a reader that exits early
   (`cargo libc inspect ... | head`), `cargo-libc` panics with "failed printing to
   stdout: Broken pipe" and exits with status 101: Rust ignores SIGPIPE, so the write
   fails, and `print!` panics on a failed write. Fix it as linux-abi-tools'
   `kernel-seccomp` was (2026-10-14): write stdout through a helper that stops quietly
   on `BrokenPipe`, tested against a pipe whose read end is closed. Then release
   0.2.1 with that fix and the one already on `main` (`test` and `run` pass arguments
   after a second `--` to the test harness or program, as the user guide always
   said; 0.2.0 refused them). Releasing needs the maintainer's go-ahead.
2. [ ] **Tier 3 on real old systems.** For example CentOS 7 (glibc 2.17, Linux 3.10)
   and SLES 12 SP5 (glibc 2.22, Linux 4.12), for executables and Python extensions,
   including native C dependencies; and a real Rocky Linux 9 build host. This is the
   main missing evidence behind the releases. There are no hosts; a possible route is
   virtual machines in CI (GitHub's Linux runners can usually run KVM): CentOS 7's
   cloud image from its archive, and openSUSE Leap 42.x (also glibc 2.22) as a free
   stand-in for SLES 12 SP5, which needs a SUSE subscription. Check first that the
   runners allow it.
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
