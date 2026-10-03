//! Golden tests against JPL Horizons — the engine's independent oracle
//! See docs/accuracy.md for scope and reproducibility.
//!
//! Fixtures: apparent right ascension / declination (airless, equinox of
//! date — Horizons OBSERVER table, `QUANTITIES='2'`, `ANG_FORMAT='DEG'`,
//! `EXTRA_PREC='YES'`), geocentric (`CENTER='500@399'`), fetched
//! 2026-08-27 from <https://ssd.jpl.nasa.gov/api/horizons.api>. Epochs
//! are UTC; the fixture-specific offsets below approximate TT for positions.
//!
//! The engine computes apparent ecliptic-of-date positions; the test
//! converts them to RA/Dec with the true obliquity (that conversion is
//! itself golden-tested against Meeus 13.a) and compares the angular
//! separation from Horizons.

use astro_engine::bodies::Body;
use astro_engine::coords::ecliptic_to_equatorial;
use astro_engine::ephemeris::apparent;
use astro_engine::nutation::true_obliquity;
use astro_engine::time::tt_from_ut1;

// JPL Horizons quantity 30, queried 2026-10-02 for the four fixture epochs.
// All epochs are after 1962: these are TDB-UTC offsets, not TT-UT1.
// For this geocentric ephemeris test ONLY, TDB approximates TT within 0.002 s.
// We add the offset directly to the fixture UTC JD; no houses use this instant.
// Future leap seconds are unknown. This is a frozen test input, not a time model.
const TIME_OFFSETS: [f64; 4] = [39.699_542, 64.183_903, 69.183_920, 69.183_952];

/// `(body, tolerance_arcsec, [(jd_ut, ra_deg, dec_deg); 4])`
///
/// Tolerances reflect each theory's documented truncation level plus
/// time-scale rounding (TDB approximates TT within 0.002 seconds):
/// Sun/planets (full VSOP87D + Meeus reductions) 15″/30″; Moon
/// (truncated ELP, ~10″ stated) 60″; Pluto (Goffin + frame hops) 60″.
type BodyFixture = (Body, f64, [(f64, f64, f64); 4]);
const FIXTURES: &[BodyFixture] = &[
    (
        Body::Sun,
        15.0,
        [
            (2_440_400.5, 96.668_786_016, 23.303_706_967),
            (2_451_545.0, 281.278_375_239, -23.032_430_130),
            (2_461_041.5, 281.494_699_195, -23.017_248_343),
            (2_466_520.5, 281.885_510_638, -22.985_725_202),
        ],
    ),
    (
        Body::Moon,
        60.0,
        [
            (2_440_400.5, 247.480_523_536, -26.741_907_886),
            (2_451_545.0, 222.452_200_851, -10.900_653_780),
            (2_461_041.5, 63.920_306_258, 26.403_701_421),
            (2_466_520.5, 254.678_771_611, -24.654_498_534),
        ],
    ),
    (
        Body::Mercury,
        30.0,
        [
            (2_440_400.5, 73.430_269_521, 19.921_287_606),
            (2_451_545.0, 272.074_637_466, -24.418_901_716),
            (2_461_041.5, 268.524_035_973, -24.001_291_126),
            (2_466_520.5, 293.178_259_979, -23.916_142_988),
        ],
    ),
    (
        Body::Venus,
        30.0,
        [
            (2_440_400.5, 49.125_381_003, 15.096_820_481),
            (2_451_545.0, 239.892_759_465, -18.448_919_196),
            (2_461_041.5, 280.056_455_581, -23.622_404_581),
            (2_466_520.5, 330.699_725_643, -13.381_157_189),
        ],
    ),
    (
        Body::Mars,
        30.0,
        [
            (2_440_400.5, 239.679_389_757, -23.718_618_671),
            (2_451_545.0, 330.516_799_463, -13.182_481_236),
            (2_461_041.5, 283.879_610_807, -23.720_007_274),
            (2_466_520.5, 277.749_469_677, -24.019_920_424),
        ],
    ),
    (
        Body::Jupiter,
        30.0,
        [
            (2_440_400.5, 178.613_688_940, 2.010_818_258),
            (2_451_545.0, 23.867_836_618, 8.594_261_973),
            (2_461_041.5, 113.124_332_352, 21.979_135_798),
            (2_466_520.5, 207.191_764_509, -9.879_081_051),
        ],
    ),
    (
        Body::Saturn,
        30.0,
        [
            (2_440_400.5, 34.973_236_741, 11.477_604_767),
            (2_451_545.0, 38.765_386_155, 12.614_763_512),
            (2_461_041.5, 357.380_959_207, -3.596_394_732),
            (2_466_520.5, 202.717_855_253, -6.893_897_416),
        ],
    ),
    (
        Body::Uranus,
        30.0,
        [
            (2_440_400.5, 180.343_626_086, 0.642_364_363),
            (2_451_545.0, 317.474_810_891, -17.020_330_403),
            (2_461_041.5, 55.737_099_009, 19.509_648_526),
            (2_466_520.5, 129.622_595_436, 19.120_718_580),
        ],
    ),
    (
        Body::Neptune,
        30.0,
        [
            (2_440_400.5, 234.469_281_618, -17.642_509_169),
            (2_451_545.0, 305.432_832_031, -19.213_240_976),
            (2_461_041.5, 0.077_610_938, -1.418_611_036),
            (2_466_520.5, 31.494_191_140, 10.872_947_784),
        ],
    ),
    (
        Body::Pluto,
        60.0,
        [
            (2_440_400.5, 179.508_244_032, 17.097_485_752),
            (2_451_545.0, 251.419_169_870, -11.394_274_163),
            (2_461_041.5, 305.936_045_877, -23.219_923_591),
            (2_466_520.5, 331.353_176_224, -22.431_016_644),
        ],
    ),
];

/// Angular separation between two RA/Dec points, arcseconds.
fn separation_arcsec(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let (ra1, dec1, ra2, dec2) = (
        ra1.to_radians(),
        dec1.to_radians(),
        ra2.to_radians(),
        dec2.to_radians(),
    );
    let cos_sep = dec1.sin() * dec2.sin() + dec1.cos() * dec2.cos() * (ra1 - ra2).cos();
    cos_sep.clamp(-1.0, 1.0).acos().to_degrees() * 3600.0
}

#[test]
fn engine_matches_jpl_horizons() {
    let mut worst: (f64, String) = (0.0, String::new());
    for &(body, tolerance, ref epochs) in FIXTURES {
        for (epoch_index, &(jd_ut, ra_expected, dec_expected)) in epochs.iter().enumerate() {
            let jd_tt = tt_from_ut1(jd_ut, TIME_OFFSETS[epoch_index]);
            let pos =
                apparent(body, jd_tt).unwrap_or_else(|| panic!("{body:?} unavailable at {jd_ut}"));
            let eps = true_obliquity(jd_tt);
            let (ra, dec) = ecliptic_to_equatorial(pos.lon_deg, pos.lat_deg, eps);
            let sep = separation_arcsec(ra, dec, ra_expected, dec_expected);
            assert!(
                sep < tolerance,
                "{body:?} at JD {jd_ut}: {sep:.2}\" from Horizons (tolerance {tolerance}\"), \
                 engine RA {ra:.6} Dec {dec:.6}, Horizons RA {ra_expected:.6} Dec {dec_expected:.6}"
            );
            if sep > worst.0 {
                worst = (sep, format!("{body:?} @ {jd_ut}"));
            }
        }
    }
    println!("worst separation: {:.2}\" ({})", worst.0, worst.1);
}
