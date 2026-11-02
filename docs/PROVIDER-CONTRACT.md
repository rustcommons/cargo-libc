# Experimental glibc provider contract

Maintained v1 consumer reference. For normal automatic provider/cache workflows,
use the [user guide](USER-GUIDE.md#providers); the manual exporter examples below
are producer development fixtures. Published releases and candidate status are in
the [release index](RELEASING.md#release-records) and
[requirements index](REQUIREMENTS.md).

Version 1 connects generated link-only DSOs to Cargo through a trusted local
descriptor. It supports x86_64 Linux dynamically linked non-PIE Rust executables,
Rust shared libraries (`cdylib`, linked without `crt1.o` and with GCC's
`crtbeginS.o`/`crtendS.o`; Rust code is position-independent in both cases), and exactly one glibc release from 2.17 through 2.44, every release in the pinned ABI lists (the `RELEASES` list in `src/provider.rs`). It is not an authenticated or relocatable release SDK.
The six-field v1 format is unchanged; existing 2.17 descriptors retain that format.
The 0.3.0 candidate additionally validates the selected libc/RT ELFs for the
[clock profile](USER-GUIDE.md#clock-linking), so a descriptor passing the six-field
parser alone does not guarantee acceptance of its library contents.

The descriptor is UTF-8, at most 16 KiB, with LF endings and a final newline.
The first line is `libc-abi-provider-v1`; six tab-separated key/value lines follow.
Unknown, duplicate, missing, empty, and control-character fields fail. Field order
is immaterial to parsing; the exporter uses this order:

```text
libc-abi-provider-v1
target<TAB>x86_64-unknown-linux-gnu
glibc<TAB>2.17
sysroot<TAB>/work/prebuilt
stubs<TAB>/work/full-libraries-prebuilt
gcc-startup<TAB>/usr/lib/gcc/x86_64-linux-gnu/13
compiler-runtime<TAB>/work/compiler-runtime
```

`<TAB>` represents a literal tab. All paths must be absolute, without parent
components. Required files must resolve inside their declared directories;
relative symlinks inside those directories are allowed.

| Directory | Required files and purpose |
| --- | --- |
| sysroot | Exact selected-release `usr/include/features.h`; `usr/lib64/{crt1.o,crti.o,crtn.o,libc_nonshared.a}`, plus `usr/lib64/libpthread_nonshared.a` before 2.28 (glibc 2.28 merged it into `libc_nonshared.a`); `lib64/ld-linux-x86-64.so.2`, which cargo-libc requires but never runs: the real loader, or in a `source`-mode provider the loader's link stub (that mode builds no glibc libraries) |
| stubs | Eight generated DSOs with exact SONAME filenames and `lib{name}.so` symlinks resolving to them; loader uses `libld.so` |
| gcc-startup | Explicit GCC `crtbegin.o`/`crtend.o`, providing compiler startup support |
| compiler-runtime | `libgcc_s.so.1` and `libgcc_s.so` to link against: a generated link-only stub with GCC 4.8.5's interface (`libc-abi libgcc stub`), the same for every release, so programs need nothing newer than CentOS 7's compiler runtime. Its functions trap; programs run with the target system's real `libgcc_s.so.1`, never the stub |

From the `rustcommons/linux-abi-tools` checkout (package `libc-abi-tools`), export after generating and independently
checking the complete stubs:

```bash
cargo run --locked --offline --example provider_2_17 -- \
  data/glibc /work/prebuilt /work/full-libraries-prebuilt \
  /usr/lib/gcc/x86_64-linux-gnu/13 /work/compiler-runtime /work/provider.txt
```

The exporter validates all eight DSOs against the pinned ABI, including SONAME,
versions, kinds, and object sizes, before creating a new descriptor. Existing
files are never overwritten. Real inputs retain their separate provenance/licenses.

Consume from the cargo-libc checkout:

```bash
cargo libc build --provider /work/provider.txt --glibc 2.17 \
  --cc /usr/bin/clang-24 --ld /usr/bin/ld.bfd \
  --artifact /work/app/x86_64-unknown-linux-gnu/debug/glibc-hello -- \
  --manifest-path examples/hello/Cargo.toml --target-dir /work/app --locked --offline
```

`--provider` conflicts with `--sysroot`/`--gcc-runtime`. Unsupported or mismatched releases fail without
fallback. Original sysroot mode remains available. The consumer validates paths,
required files, and exact headers; explicitly selects CRT/nonshared files; searches
stubs and the compatible unwinder first; and checks direct executable GLIBC requirements.
It does not revalidate full ABI membership or hash every provider file on every
build. For the clock profile, the producer validates and hashes the selected
`libc.so.6`/`librt.so.1` pair and owns the read-only derived RT cache.

### Derived RT selection in the 0.3.0 candidate

The descriptor continues to name the original, faithful provider stubs. Do not
change its `stubs` field or overwrite the original RT DSO to opt into the clock
default. Cargo derives the private overlay from those selected ELFs and puts it
before the provider directories for target links, including C's implicit default
libc. The profile keeps all explicit old clock versions as nondefault aliases;
ordinary calls use libc's GLIBC_2.17 definitions. Other RT exports, library order,
groups, as-needed state and GCC/unwinder inputs retain their roles.

Matching full-path RT arguments use the derived file at the same position.
The completed Clang command supplies effective named-library search paths and
static/dynamic state, including compiler-injected C libraries. Ordinary selected
ELF/archive dependencies are accepted; unproven linker-default fallbacks refuse.
Normal full-sysroot libc/libm/GCC wrapper scripts have bounded supported forms
whose leaves are checked without rewriting the script or its order. Alternate
RT DSOs and opaque custom input scripts/response files are refused. This same
producer API accepts safely discovered full-sysroot ELF pairs, with no host
fallback or mutation of user files. Cache corruption or unqualified inputs fail
without reverting to the original default bindings. A compliant pair creates no
overlay and does not force librt into the link. This adds no field to descriptor
v1 and no user clock option. See the [candidate record](RELEASE-0.3.0.md) for
named passing execution/registry gates and remaining scope limits rather than
inferring qualification from format support.

Explicit Cargo `--target` and encoded target Rust flags preserve host compilation
of build scripts/procedural macros. Descriptor contents enter Cargo's fingerprint;
changing input files in place still requires a fresh target directory or clean
rebuild for inputs outside the selected clock pair. That pair's raw hashes and
the profile ID enter Cargo's fingerprint; generated shell/TOML settings must be
regenerated after changes. C dependencies compiled by `cc` or CMake use `libc-cc`, which compiles
with the provider's `usr/include` and links with the same startup objects, stubs,
nonshared archives and compiler runtime as Rust (plus GCC's default `-lgcc`,
`-lgcc_s` and `-lc`). Arbitrary native dependency overrides remain outside the
prototype scope.

`examples/glibc-two-paths/provider-and-check.sh WORK CARGO_LIBC_REPO CLANG` tests
the selected release's input routes, host build units, a target thread, wrong-version rejection,
ELF requirements, all shared link-map inputs, and runtime dependency paths. Its
SHA-256 logs are evidence, not authenticated SDK manifests.

For 2.22, set `ABI_GLIBC_RELEASE=2.22` and use `--example provider_glibc -- DATA
2.22 SYSROOT STUBS GCC_STARTUP COMPILER_RUNTIME NEW_DESCRIPTOR`. Only the pinned
source-built target route is used. See the [2.22 experiment](https://github.com/rustcommons/linux-abi-tools/blob/main/docs/GLIBC-2.22.md).

Future release packaging requires relative bundle paths, complete inventories,
integrity validation, provenance/licenses, stable compiler-runtime policy, and
target distribution/kernel qualification in a future contract version.
