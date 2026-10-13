# Cargo flags and workspace builds

Pass ordinary Cargo build options after the coordinator's separator. Options
before it select the provider, target, compiler/linker, and executable to inspect.

    cargo libc build --target x86_64-unknown-linux-gnu.2.17 --provider FILE --artifact target/x86_64-unknown-linux-gnu/release/my-app -- --manifest-path Cargo.toml -p my-package --bin my-app --release --locked --offline

The current coordinator forwards Cargo arguments as separate OS strings, keeping
paths with spaces intact. Cargo decides package, feature, profile and target
selection. The coordinator adds the ordinary Rust target and JSON artifact output,
then requires the selected executable to be reported by that invocation. Cargo
may report an unchanged executable as fresh; a file merely existing on disk is
insufficient.

`cargo libc check` takes the same options before the separator, minus the
artifact, and forwards the arguments after it to `cargo check` with the same
restrictions. It shares the managed target flags with `build`, so host build
scripts and procedural macros stay native; it does not link and makes no
claim about the final executable.

`clippy`, `doc` and `rustc` work like `check`. `clippy` and `rustc` accept
Cargo's own `--` for tool arguments, which must not name a target, linker, link
arguments or relocation model. `install` works like `build` without
`--artifact` and checks every executable it built. `env` prints the same
configuration for plain Cargo (README, "Plain Cargo"). `scripts/smoke-commands.sh`
covers all five with automatic providers in CI.

## Qualified workflow fixtures

The hello fixture has a second workspace package, a feature-gated example, and a
custom profile inheriting release behavior. The provider integration exercises:

- Workspace build with an excluded member, all binaries/examples, a namespaced
  feature, no default features, a custom profile, and locked/offline operation.
- Repeated builds into the same target directory, accepting Cargo's fresh output.
- Package/example selection with an unqualified feature name and release mode.
- Package/bin selection that leaves another executable on disk, rejecting that
  unreported executable as the selected artifact.
- Real-loader execution of the main binary, second package, and example; the
  example asserts feature activation and release/custom profile behavior.
- `check` of the same workspace with each provider and versioned target
  (2026-10-11), and rejection of a `--target` suffix that conflicts with
  `--glibc`.

Custom profile outputs use their profile directory, while release outputs use
release. Examples are below an examples directory. All use the ordinary Rust
triple, without the glibc suffix.

## Preserved restrictions and scope

Cargo target overrides, configuration overrides, message-format changes, an
additional argument separator, and toolchain selectors after the separator are
rejected. These restrictions protect the controlled target and artifact reporting;
general safe configuration/flag merging is separate work (F13). Existing global
Rustflags overrides remain rejected.

The wrapper validates the one executable named by its artifact option.
Building several workspace outputs does not imply compatibility validation of
every output. Workspace-wide checks and library output policies remain separate
features (F10/F11). C dependencies built with `cc` or CMake are configured for the
target (README, "C dependencies"); C++ and `pkg-config` dependencies are not.
Build flags not exercised by these fixtures are forwarded, but this document
does not claim qualification of every Cargo flag or compiler combination.

Qualified with producer c442310 and consumer 6b07260: [all nine provider jobs passed](https://github.com/rustcommons/linux-abi-tools/actions/runs/38077381465). See [VALIDATION.md](../VALIDATION.md) and [the roadmap](ROADMAP.md).
