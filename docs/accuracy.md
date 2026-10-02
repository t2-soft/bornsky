# Numerical methods and limits

## What the library computes

The calculation pipeline uses the full VSOP87D series through the `vsop87` crate,
then geocentric conversion, light-time iteration, FK5 correction, aberration and
nutation. The Moon uses truncated ELP2000-82 terms; Pluto uses the Goffin series
with an explicit date gate. Nutation is IAU 2000B and precession/obliquity IAU 2006.
The ERFA and astro-rust adaptations are credited in THIRD_PARTY_NOTICES.md.

Positions are apparent geocentric ecliptic-of-date, in degrees. Signs use the
tropical zodiac. Distances are AU. Longitude speeds are degrees/day. House
coordinates use latitude north-positive and longitude east-positive.

## Time is an explicit input

`BirthData.jd_ut` means **UT1**. The third `natal_chart` argument is **TT - UT1 in
seconds**; the output retains it and the resulting TT Julian Day. The library
does not estimate this correction. Do not substitute local wall-clock time, or
silently treat an unknown birth time as noon.

For modern civil timestamps: resolve local time and DST to UTC externally;
apply DUT1 to obtain UT1; obtain TT using the appropriate time-scale data. UTC
and UT1 differ, and TT-UTC is not identical to TT-UT1. These policies belong to
the caller. Future leap seconds and historical Earth rotation are uncertain.

The conversion helper adds the supplied seconds divided by 86,400. An automatic
Delta-T implementation was deliberately not carried into this repository because
its source provenance needed a separate license review. No coefficients from
that implementation are included.

## Independent and regression checks

`tests/horizons_golden.rs` compares Sun, Moon and eight planets at four frozen
epochs against JPL Horizons apparent right ascension and declination. The RA/Dec
fixtures were collected on 2026-08-27. Request parameters:

```text
EPHEM_TYPE=OBSERVER
CENTER=500@399
TLIST=2440400.5,2451545.0,2461041.5,2466520.5
QUANTITIES=2
ANG_FORMAT=DEG
EXTRA_PREC=YES
```

These correspond to 1969-06-28, 2000-01-01 12:00, 2026-01-01 and 2041-01-01.
Command IDs are Sun 10, Moon 301, Mercury 199, Venus 299, Mars 499, Jupiter 599,
Saturn 699, Uranus 799, Neptune 899 and Pluto 999.

Quantity 30 was queried separately on 2026-10-02 for the same epochs: TDB-UTC
offsets 39.699542, 64.183903, 69.183920 and 69.183952 seconds. The test adds those
offsets to its UTC Julian Days and uses TDB as an approximation to TT (within
0.002 seconds). This is only a geocentric ephemeris test; it does not compute
houses from UTC or use these four constants as a general Delta-T model.
The future offset is a frozen Horizons assumption, not a known future leap-second
schedule. See the [Horizons time-scale documentation](https://ssd.jpl.nasa.gov/horizons/manual.html#time).

Test tolerances: Sun 15 arcseconds, other VSOP87 planets 30 arcseconds, Moon and
Pluto 60 arcseconds. These are test thresholds for the sampled epochs, not global
error guarantees or endorsements from JPL. Lower-level tests use Meeus worked
examples and ERFA reference vectors; property tests check round trips, coordinate
ranges, cusp ordering and motion. Sidereal JSON fixtures are regression checks,
not an independent ephemeris oracle. No network is required to run tests.

## Known limits

- Input dates are bounded by `time::JD_SUPPORTED` (approximately 1550–2650 with
  a margin), but historical/future accuracy depends on the theory and time input.
- Pluto has a narrower `ephemeris::pluto::supported_jd` gate. Unavailable bodies
  are listed in the result, never replaced with invented positions.
- The Moon is a truncated series. This is not a DE440/DE441 numerical ephemeris.
- Mean apogee is provisional. The separate true-node helper is an approximation;
  natal charts use the mean node.
- Placidus at polar latitudes and degenerate Regiomontanus geometry fall back
  to Porphyry with `fell_back_to_porphyry`. Exact geographic poles are rejected
  by the checked chart API.
- Atmospheric refraction, topocentric parallax, timezone/DST resolution and
  geocoding are not part of natal-chart assembly.
- Numerical agreement cannot validate astrological interpretations or predictions.
