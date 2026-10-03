# Locked dependency licence inventory

Reviewed 2026-10-03 against `Cargo.lock` using `cargo metadata --locked`.
This includes runtime, build, target-specific and test dependencies; it is not
a claim that every package is linked into each distributed binary.

All listed external packages declare an MIT or Apache-2.0 option.
`unicode-ident` additionally requires Unicode-3.0. The historical slash notation
in some manifests denotes a licence choice. No dependency requires a copyleft
option for this locked graph when its permissive alternative is selected.

Verify the actual source licence files and include the required notices when
redistributing binaries or vendored dependencies. Regenerate and review this
inventory whenever the lockfile changes. It records upstream declarations,
not an independent guarantee of copyright ownership.

| Package | Version | Declared licence expression |
| --- | --- | --- |
| autocfg | 1.5.1 | `Apache-2.0 OR MIT` |
| bitflags | 2.13.2 | `MIT OR Apache-2.0` |
| bit-set | 0.8.0 | `Apache-2.0 OR MIT` |
| bit-vec | 0.8.0 | `Apache-2.0 OR MIT` |
| cfg-if | 1.0.5 | `MIT OR Apache-2.0` |
| errno | 0.3.14 | `MIT OR Apache-2.0` |
| fastrand | 2.5.0 | `Apache-2.0 OR MIT` |
| fnv | 1.0.7 | `Apache-2.0 / MIT` |
| getrandom | 0.3.4 | `MIT OR Apache-2.0` |
| getrandom | 0.4.3 | `MIT OR Apache-2.0` |
| itoa | 1.0.18 | `MIT OR Apache-2.0` |
| libc | 0.2.190 | `MIT OR Apache-2.0` |
| linux-raw-sys | 0.12.1 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| memchr | 2.8.3 | `Unlicense OR MIT` |
| num-traits | 0.2.19 | `MIT OR Apache-2.0` |
| once_cell | 1.21.4 | `MIT OR Apache-2.0` |
| ppv-lite86 | 0.2.21 | `MIT OR Apache-2.0` |
| proc-macro2 | 1.0.107 | `MIT OR Apache-2.0` |
| proptest | 1.11.0 | `MIT OR Apache-2.0` |
| quick-error | 1.2.3 | `MIT/Apache-2.0` |
| quote | 1.0.47 | `MIT OR Apache-2.0` |
| rand | 0.9.5 | `MIT OR Apache-2.0` |
| rand_chacha | 0.9.0 | `MIT OR Apache-2.0` |
| rand_core | 0.9.5 | `MIT OR Apache-2.0` |
| rand_xorshift | 0.4.0 | `MIT OR Apache-2.0` |
| r-efi | 5.3.0 | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` |
| r-efi | 6.0.0 | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` |
| regex-syntax | 0.8.11 | `MIT OR Apache-2.0` |
| rustix | 1.1.5 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| rusty-fork | 0.3.1 | `MIT/Apache-2.0` |
| serde | 1.0.229 | `MIT OR Apache-2.0` |
| serde_core | 1.0.229 | `MIT OR Apache-2.0` |
| serde_derive | 1.0.229 | `MIT OR Apache-2.0` |
| serde_json | 1.0.151 | `MIT OR Apache-2.0` |
| syn | 2.0.119 | `MIT OR Apache-2.0` |
| syn | 3.0.6 | `MIT OR Apache-2.0` |
| tempfile | 3.27.0 | `MIT OR Apache-2.0` |
| unarray | 0.1.4 | `MIT OR Apache-2.0` |
| unicode-ident | 1.0.26 | `(MIT OR Apache-2.0) AND Unicode-3.0` |
| vsop87 | 3.0.0 | `MIT/Apache-2.0` |
| wait-timeout | 0.2.1 | `MIT/Apache-2.0` |
| wasip2 | 1.0.4+wasi-0.2.12 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| windows-link | 0.2.1 | `MIT OR Apache-2.0` |
| windows-sys | 0.61.2 | `MIT OR Apache-2.0` |
| wit-bindgen | 0.57.1 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| zerocopy | 0.8.59 | `BSD-2-Clause OR Apache-2.0 OR MIT` |
| zerocopy-derive | 0.8.59 | `BSD-2-Clause OR Apache-2.0 OR MIT` |
| zmij | 1.0.23 | `MIT` |
