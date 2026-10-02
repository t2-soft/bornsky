# Third-party attribution

The Apache-2.0 license covers Bornsky's original code. It does not replace the
licenses of third-party portions or dependencies.

| Component | Where used | Source and terms |
| --- | --- | --- |
| ERFA 2.0.x | IAU 2000B nutation, IAU 2006 obliquity and precession in `nutation.rs` and `coords.rs` | [ERFA v2.0.1](https://github.com/liberfa/erfa/tree/v2.0.1); full BSD-style notice in [licenses/ERFA.txt](licenses/ERFA.txt) |
| astro-rust | Lunar and Pluto coefficient tables in `ephemeris/moon.rs` and `ephemeris/pluto.rs` | [saurvs/astro-rust](https://github.com/saurvs/astro-rust); MIT notice in [licenses/astro-rust.txt](licenses/astro-rust.txt) |
| VSOP87 | Planetary series, supplied as a Cargo dependency, not vendored | [vsop87](https://crates.io/crates/vsop87); MIT OR Apache-2.0 |
| Serde | Serialization derives, supplied as a Cargo dependency | [Serde](https://serde.rs/); MIT OR Apache-2.0 |
| JPL Horizons | Independent numerical test observations | [Horizons](https://ssd.jpl.nasa.gov/horizons/); request details and limitations in `docs/accuracy.md` |

The ERFA-derived routines are Rust adaptations, not the unmodified SOFA reference
implementation. No endorsement by IAU, SOFA, JPL or NASA is implied.

Mathematical references include Jean Meeus, *Astronomical Algorithms*, second
edition; VSOP87D; IAU 2000B; and IAU 2006. These references are not claims to
ownership of the underlying theories.

No automatic Delta-T model is distributed. In particular, no code or coefficient
table from `ytliu0/DeltaT` is included. UT1-to-TT conversion here only adds a
caller-supplied offset; the caller is responsible for the source and license of
that offset. Cargo.lock records the resolved external dependencies; their own
license files apply when distributing binaries.
