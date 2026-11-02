# cargo-libc 0.3.0 — source candidate

Not released. Producer source qualification is in progress; publication and
registry-only validation remain pending. The tracked lock is still the last
published release's registry lock, not a fabricated unpublished 0.4.0 entry.
Only disposable source consumers patch `libc-abi-tools` during this stage.

The approved default selects libc's GLIBC_2.17 definitions of `clock_gettime`,
`clock_getres`, `clock_settime`, `clock_nanosleep` and `clock_getcpuclockid`.
There is no clock option, environment policy switch or opt-out. Provider/sysroot
selection still controls the target inputs; there is no host-library fallback.

`libc-abi-tools` 0.4.0 supplies the versioned `LibcClocksV1` profile. Its builder
validates actual selected libc/librt ELFs and caches a read-only RT copy, changing
only the five version-default bits. Old explicit versions remain available.
Original ABI/source data, user files, code, unwinder/libgcc_s selection, linker
ordering, groups and as-needed state are not rewritten. A no-as-needed link may
retain librt in DT_NEEDED while ordinary clocks' version-needed owner is libc.
There is no new static runtime support.

The overlay applies to automatic source/container providers, explicit providers
and safely discovered full-sysroot inputs through the same target helper path.
Controlled matching full-path RT inputs are substituted at the same argument
position. Opaque linker-input scripts and response files are currently refused,
not silently accepted as a policy bypass. Version scripts that control exports
are distinct from linker-input scripts. These boundaries need native review;
source code presence is not qualification of every provider mode or input form.

Profile identity and both original input hashes enter generated shell/TOML
configuration and the Cargo fingerprint. Updating an existing environment
requires regenerating it with `cargo libc env`. The same-target-directory upgrade
regression must demonstrate a relink without manual cleaning. This derived cache
does not alter descriptor v1 or replace provider-cache semantics.

Only the direct `libc-abi-tools` floor rises to 0.4.0. Scanner/kernel minima stay
0.3.3 and linux-targets stays 0.3.0, with their lock resolutions held at 0.3.3.
Glibc inputs remain 0.1.2/0.1.1 with their existing registry checksums.

The [dated investigation](evals/2026-11-02-clock-linking.md) preserves rejected
ordering prototypes and their real mixed-version alias failure. The source gate
initially integrated producer `1b3b51c8afad88e05b0dc238c15f5ee879f94184` in a
disposable root-only source consumer. That producer is independently review-blocked
for partial PT_LOAD overlap validation and an empty-destination rename race;
no results on it count as final release gates. A corrected reviewed ref is pending,
with the same agreed API. The standalone source driver requires that exact ref as
its fifth argument and checks it against a clean producer checkout. It validates
the appended disposable patch with `tomllib` before Cargo runs. Native cases save objects,
before/after ELFs, complete readelf evidence and command results before assertions.
This first gate covers source/explicit providers, GNU ld/LLD, C and Cargo mixed
references, RT APIs, linker state, full paths, environment roundtrips and a
same-directory upgrade, plus existing C/plain-Cargo/host-sysroot smoke checks.
Container-provider preparation is not part of that first bounded gate.

Remaining release gates: source/native results and producer independent approval;
producer publication first; real registry lock adoption; final portable/native
release checks, package and clean unpacked locked install; independent Cargo
review and merge; sole-coordinator publication; official archive, docs.rs, exact
tag and published installation verification. The separate 182-case consumer
comparison preserves its original strict comparator. Clock changes do not satisfy
that comparator's glibc-only dependency condition while libgcc_s remains.
Tier 3 qualification stays deferred and prebuilt-provider legal gates remain.
