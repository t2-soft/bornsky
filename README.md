# Bornsky — Astrology calculation engine in Rust

A standalone **Rust astrology library** for planetary positions, natal charts
(birth charts), tropical zodiac signs, house systems and astrological aspects.
It is Bornsky's calculation foundation, licensed under **Apache-2.0**, with
inherited third-party notices. No account, API key, database or hosted service
is needed to run the library.

[Quick start](#run-it) · [Accuracy and limits](docs/accuracy.md) ·
[Licensing](docs/licensing.md) · [Contributing](CONTRIBUTING.md)

## Choose how to use Bornsky

| Rust calculation library | Hosted astrology API |
| --- | --- |
| Run calculations in your own application or infrastructure. | Integrate through Bornsky's commercial partner service. |
| Numerical positions, houses, angles and aspects. | Hosted calculations and separately offered paid capabilities. |
| You supply time conversion and geographic coordinates. | Explore available endpoints and access in the Developer Studio. |
| [Run the example](#run-it). | [Open Developer Studio (preview)](https://bornsky-api-staging.bornsky-cloud.workers.dev/). |

## Hosted API and partnerships

**A calculation engine written in Rust, delivered through our Cloudflare-based
partner API.**

Bring astrology into your app or website through Bornsky's commercial hosted
service. Our partner API provides access to the Rust engine without operating
your own calculation infrastructure.

- **For developers:** explore the [Developer Studio (preview)](https://bornsky-api-staging.bornsky-cloud.workers.dev/).
- **For businesses and partners:** [discuss API access and integration](https://bornsky.app/en#api-partnership).
- **For personal charts and readings:** [visit Bornsky](https://bornsky.app/).

The hosted API, paid services and interpretation catalogs are separate offerings
and are not included in this Apache-2.0 library.

## Included

- Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto,
  plus mean lunar node and mean apogee.
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

The result contains body longitudes, latitudes, distances, zodiac signs,
retrograde flags and house numbers; twelve house cusps, Ascendant and Midheaven;
detected aspects; and the supplied time correction. Unavailable bodies and house
system fallbacks are explicit. This is numerical chart data, not a written reading.

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
third-party portions retain their own notices. **Commercial use, modification
and redistribution of the public calculation code are permitted**, including
use in proprietary applications and competing hosted services, subject to the
applicable licence conditions. You do not need a Bornsky subscription to use it.

The licence does not grant access to Bornsky's separate service code, catalogs,
accounts or paid API, or trademark rights beyond the licence's attribution
allowance. See [licensing and commercial boundaries](docs/licensing.md).

## Support

For reproducible calculation bugs or library questions, [open a GitHub issue](https://github.com/t2-soft/bornsky/issues)
with synthetic inputs, the library revision and expected numerical results.
For hosted API access and partnerships, [contact Bornsky](https://bornsky.app/en#api-partnership).
See [SECURITY.md](SECURITY.md) for sensitive reports. Community support carries
no response-time or service-level commitment.

Maintainers: follow the [release checklist](docs/release-checklist.md) before
changing visibility or publishing a package. GitHub source availability and a
crates.io package release are separate steps.
