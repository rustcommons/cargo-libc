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
features (F10/F11). Native C/C++ dependencies still need their own configuration.
Build flags not exercised by these fixtures are forwarded, but this document
does not claim qualification of every Cargo flag or compiler combination.

Qualified with producer c442310 and consumer 6b07260: [all nine provider jobs passed](https://github.com/rustcommons/libc-abi-tools/actions/runs/38077381465). See [VALIDATION.md](../VALIDATION.md) and [the roadmap](ROADMAP.md).
