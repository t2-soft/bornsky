# Bornsky calculation foundation

A small, standalone Rust library for astronomical positions and natal-chart
geometry. It is the calculation-only portion of Bornsky, prepared for open-source
use under **Apache-2.0**. No account, API key, database or hosted service is needed.

## Included

- Sun, Moon and eight planets, plus mean lunar node and mean apogee.
- VSOP87D planetary positions, light-time and apparent-position reductions.
- Lunar/Pluto series, IAU 2000B nutation and IAU 2006 precession/obliquity.
- Julian Day arithmetic, sidereal time and coordinate transformations.
- UT1-to-TT arithmetic with an **explicit caller-supplied time correction**.
- Twelve tropical zodiac signs; Placidus, Whole Sign, Equal, Porphyry and
  Regiomontanus houses; major/minor aspects and natal-chart assembly.
- Unit, property and numerical-reference tests, plus a JSON example.

## Kept outside this repository

Interpretation catalogs and translations, authored readings, forecast synthesis,
ranking and relevance rules, matching services, horary judgments, rectification,
astrocartography products, premium workflows, HTTP servers, authentication,
billing, databases, deployment configuration, websites and mobile apps.

The library supplies numerical results, not personalized advice or a replacement
for Bornsky's hosted products. It has no outbound requests or telemetry.

## Run it

Install Rust through [rustup](https://rustup.rs/). The repository pins its tested
toolchain; the minimum supported Rust version is 1.94.

```sh
cargo test --workspace --locked
cargo run -p astro-engine --example natal --locked
cargo doc --workspace --no-deps --locked
```

The example prints a synthetic natal chart as JSON. Use the crate from this
workspace or a pinned Git revision; it is not published on crates.io.

```rust
use astro_engine::chart::{natal_chart, BirthData};
use astro_engine::houses::HouseSystem;
use astro_engine::time::julian_day;

// Synthetic example in UT1, not local time. 64 seconds is illustrative only.
let birth = BirthData {
    jd_ut: julian_day(2000, 1, 1.5),
    lat_deg: 51.5,
    lon_deg: -0.12,
};
let chart = natal_chart(birth, HouseSystem::Placidus, 64.0)
    .expect("valid synthetic example");
```

Callers must resolve local time/timezone and obtain TT - UT1 for the date from
their chosen data source. There is no automatic Delta-T model, leap-second table,
geocoding, or timezone database here. Do not use the example's correction for
other dates. Returned charts retain the correction and TT instant for reproducibility.

## Accuracy and limits

Read [accuracy and time scales](docs/accuracy.md). The test fixtures check
astronomical positions; they do not establish predictive validity for astrology.
Pluto is available only within the implemented 1885–2099 date gate. Extreme
latitudes may use a flagged Porphyry fallback. Mean apogee remains provisional.
The supported input date range is not a claim of uniform accuracy across it.

The checked chart entry point rejects invalid inputs. Lower-level mathematical
functions expect valid finite inputs in their documented units.

## Contributing and licensing

See [CONTRIBUTING.md](CONTRIBUTING.md), the [export boundary](docs/export-boundary.md)
and [third-party notices](THIRD_PARTY_NOTICES.md). Original code is Apache-2.0;
third-party portions retain their own notices. The license allows commercial
reuse of this calculation code and does not grant rights to excluded private
software, catalogs or Bornsky trademarks.

This repository is being prepared privately. Making it public is a separate
owner-controlled release step; see the [release checklist](docs/release-checklist.md).
