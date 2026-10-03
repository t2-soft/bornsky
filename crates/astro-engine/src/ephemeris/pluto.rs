//! Pluto — Goffin, Meeus & Steyaert 1986 trigonometric series (Meeus
//! ch. 37), heliocentric ecliptic **J2000**, then reduced to geocentric
//! apparent of-date via the equatorial precession path.
//!
//! Series error vs the numerical integration it was fitted to: <0.07″
//! in longitude within its validity span. **Valid 1885–2099 only** —
//! outside, [`apparent`] returns `None`. No alternative ephemeris is bundled.
//! Coefficient table adapted to Rust tuples from MIT-licensed `astro-rust`.
//! Copyright (c) 2015, 2016 Saurav Sachidanand; retain `licenses/astro-rust.txt`.

use super::{aberration, EclipticPos, LIGHT_TIME_PER_AU};
use crate::angle::normalize_deg;
use crate::coords;
use crate::nutation;
use crate::time::{julian_day, J2000};

/// Validity span of the Goffin series (JDs of 1885-01-01 and 2099-12-31).
#[must_use]
pub fn supported_jd() -> (f64, f64) {
    (julian_day(1885, 1, 1.0), julian_day(2099, 12, 31.0))
}

/// Mean obliquity at J2000, degrees (for the J2000 ecl↔equ step).
const OBLIQUITY_J2000: f64 = 23.439_291_1;

/// (J, S, P multipliers; longitude sin/cos [deg]; latitude sin/cos
/// [deg]; radius sin/cos [AU]) — Goffin 1986 / Meeus table 37.A.
type SeriesRow = (f64, f64, f64, f64, f64, f64, f64, f64, f64);
#[rustfmt::skip]
const TERMS: &[SeriesRow] = &[
    (0.0, 0.0, 1.0, -19.799_805, 19.850_055, -5.452_852, -14.974_862, 6.686_543_9, 6.895_181_2),
    (0.0, 0.0, 2.0, 0.897_144, -4.954_829, 3.527_812, 1.672_790, -1.182_753_5, -0.033_253_8),
    (0.0, 0.0, 3.0, 0.611_149, 1.211_027, -1.050_748, 0.327_647, 0.159_317_9, -0.143_889_0),
    (0.0, 0.0, 4.0, -0.341_243, -0.189_585, 0.178_690, -0.292_153, -0.001_844_4, 0.048_322_0),
    (0.0, 0.0, 5.0, 0.129_287, -0.034_992, 0.018_650, 0.100_340, -0.006_597_7, -0.008_543_1),
    (0.0, 0.0, 6.0, -0.038_164, 0.030_893, -0.030_697, -0.025_823, 0.003_117_4, -0.000_603_2),
    (0.0, 1.0, -1.0, 0.020_442, -0.009_987, 0.004_878, 0.011_248, -0.000_579_4, 0.002_216_1),
    (0.0, 1.0, 0.0, -0.004_063, -0.005_071, 0.000_226, -0.000_064, 0.000_460_1, 0.000_403_2),
    (0.0, 1.0, 1.0, -0.006_016, -0.003_336, 0.002_030, -0.000_836, -0.000_172_9, 0.000_023_4),
    (0.0, 1.0, 2.0, -0.003_956, 0.003_039, 0.000_069, -0.000_604, -0.000_041_5, 0.000_070_2),
    (0.0, 1.0, 3.0, -0.000_667, 0.003_572, -0.000_247, -0.000_567, 0.000_023_9, 0.000_072_3),
    (0.0, 2.0, -2.0, 0.001_276, 0.000_501, -0.000_057, 0.000_001, 0.000_006_7, -0.000_006_7),
    (0.0, 2.0, -1.0, 0.001_152, -0.000_917, -0.000_122, 0.000_175, 0.000_103_4, -0.000_045_1),
    (0.0, 2.0, 0.0, 0.000_630, -0.001_277, -0.000_049, -0.000_164, -0.000_012_9, 0.000_050_4),
    (1.0, -1.0, 0.0, 0.002_571, -0.000_459, -0.000_197, 0.000_199, 0.000_048_0, -0.000_023_1),
    (1.0, -1.0, 1.0, 0.000_899, -0.001_449, -0.000_025, 0.000_217, 0.000_000_2, -0.000_044_1),
    (1.0, 0.0, -3.0, -0.001_016, 0.001_043, 0.000_589, -0.000_248, -0.000_335_9, 0.000_026_5),
    (1.0, 0.0, -2.0, -0.002_343, -0.001_012, -0.000_269, 0.000_711, 0.000_785_6, -0.000_783_2),
    (1.0, 0.0, -1.0, 0.007_042, 0.000_788, 0.000_185, 0.000_193, 0.000_003_6, 0.004_576_3),
    (1.0, 0.0, 0.0, 0.001_199, -0.000_338, 0.000_315, 0.000_807, 0.000_866_3, 0.000_854_7),
    (1.0, 0.0, 1.0, 0.000_418, -0.000_067, -0.000_130, -0.000_043, -0.000_080_9, -0.000_076_9),
    (1.0, 0.0, 2.0, 0.000_120, -0.000_274, 0.000_005, 0.000_003, 0.000_026_3, -0.000_014_4),
    (1.0, 0.0, 3.0, -0.000_060, -0.000_159, 0.000_002, 0.000_017, -0.000_012_6, 0.000_003_2),
    (1.0, 0.0, 4.0, -0.000_082, -0.000_029, 0.000_002, 0.000_005, -0.000_003_5, -0.000_001_6),
    (1.0, 1.0, -3.0, -0.000_036, -0.000_029, 0.000_002, 0.000_003, -0.000_001_9, -0.000_000_4),
    (1.0, 1.0, -2.0, -0.000_040, 0.000_007, 0.000_003, 0.000_001, -0.000_001_5, 0.000_000_8),
    (1.0, 1.0, -1.0, -0.000_014, 0.000_022, 0.000_002, -0.000_001, -0.000_000_4, 0.000_001_2),
    (1.0, 1.0, 0.0, 0.000_004, 0.000_013, 0.000_001, -0.000_001, 0.000_000_5, 0.000_000_6),
    (1.0, 1.0, 1.0, 0.000_005, 0.000_002, 0.0, -0.000_001, 0.000_000_3, 0.000_000_1),
    (1.0, 1.0, 3.0, -0.000_001, 0.0, 0.0, 0.0, 0.000_000_6, -0.000_000_2),
    (2.0, 0.0, -6.0, 0.000_002, 0.0, 0.0, -0.000_002, 0.000_000_2, 0.000_000_2),
    (2.0, 0.0, -5.0, -0.000_004, 0.000_005, 0.000_002, 0.000_002, -0.000_000_2, -0.000_000_2),
    (2.0, 0.0, -4.0, 0.000_004, -0.000_007, -0.000_007, 0.0, 0.000_001_4, 0.000_001_3),
    (2.0, 0.0, -3.0, 0.000_014, 0.000_024, 0.000_010, -0.000_008, -0.000_006_3, 0.000_001_3),
    (2.0, 0.0, -2.0, -0.000_049, -0.000_034, -0.000_003, 0.000_020, 0.000_013_6, -0.000_023_6),
    (2.0, 0.0, -1.0, 0.000_163, -0.000_048, 0.000_006, 0.000_005, 0.000_027_3, 0.000_106_5),
    (2.0, 0.0, 0.0, 0.000_009, -0.000_024, 0.000_014, 0.000_017, 0.000_025_1, 0.000_014_9),
    (2.0, 0.0, 1.0, -0.000_004, 0.000_001, -0.000_002, 0.0, -0.000_002_5, -0.000_000_9),
    (2.0, 0.0, 2.0, -0.000_003, 0.000_001, 0.0, 0.0, 0.000_000_9, -0.000_000_2),
    (2.0, 0.0, 3.0, 0.000_001, 0.000_003, 0.0, 0.0, -0.000_000_8, 0.000_000_7),
    (3.0, 0.0, -2.0, -0.000_003, -0.000_001, 0.0, 0.000_001, 0.000_000_2, -0.000_001_0),
    (3.0, 0.0, -1.0, 0.000_005, -0.000_003, 0.0, 0.0, 0.000_001_9, 0.000_003_5),
    (3.0, 0.0, 0.0, 0.0, 0.0, 0.000_001, 0.0, 0.000_001_0, 0.000_000_3),
];

/// Heliocentric ecliptic J2000 position: (longitude °, latitude °,
/// radius AU). `None` outside the series' validity span.
#[must_use]
pub fn heliocentric_j2000(jd_tt: f64) -> Option<(f64, f64, f64)> {
    let (lo, hi) = supported_jd();
    if !(lo..=hi).contains(&jd_tt) {
        return None;
    }
    let jc = (jd_tt - J2000) / 36_525.0;

    // Mean arguments (degrees): Jupiter, Saturn, Pluto.
    let jup = 34.35 + 3034.9057 * jc;
    let sat = 50.08 + 1222.1138 * jc;
    let plu = 238.96 + 144.96 * jc;

    let mut lon = 238.958_116 + 144.96 * jc;
    let mut lat = -3.908_239;
    let mut radius = 40.724_134_6;

    for &(mul_j, mul_s, mul_p, lon_s, lon_c, lat_s, lat_c, rad_s, rad_c) in TERMS {
        let arg = (mul_j * jup + mul_s * sat + mul_p * plu).to_radians();
        let (sin_a, cos_a) = arg.sin_cos();
        lon += lon_s * sin_a + lon_c * cos_a;
        lat += lat_s * sin_a + lat_c * cos_a;
        radius += rad_s * sin_a + rad_c * cos_a;
    }

    Some((normalize_deg(lon), lat, radius))
}

/// Earth heliocentric rectangular ecliptic-J2000 coordinates (AU) via
/// VSOP87A.
fn earth_rect_j2000(jd_tt: f64) -> (f64, f64, f64) {
    let earth = vsop87::vsop87a::earth(jd_tt);
    (earth.x, earth.y, earth.z)
}

/// Geocentric apparent ecliptic-of-date position of Pluto.
#[must_use]
pub fn apparent(jd_tt: f64) -> Option<EclipticPos> {
    let (ex, ey, ez) = earth_rect_j2000(jd_tt);

    // Light-time iteration in the J2000 frame.
    let mut tau = 0.0;
    let mut geo = (0.0, 0.0, 0.0);
    for _ in 0..3 {
        let (lon, lat, radius) = heliocentric_j2000(jd_tt - tau)?;
        let (lon_r, lat_r) = (lon.to_radians(), lat.to_radians());
        let px = radius * lat_r.cos() * lon_r.cos();
        let py = radius * lat_r.cos() * lon_r.sin();
        let pz = radius * lat_r.sin();
        geo = (px - ex, py - ey, pz - ez);
        let dist = (geo.0 * geo.0 + geo.1 * geo.1 + geo.2 * geo.2).sqrt();
        tau = LIGHT_TIME_PER_AU * dist;
    }
    let dist = (geo.0 * geo.0 + geo.1 * geo.1 + geo.2 * geo.2).sqrt();

    // Geocentric ecliptic J2000 → equatorial J2000 → precess →
    // ecliptic of date (reuses the golden-tested precession).
    let lon_j2000 = normalize_deg(geo.1.atan2(geo.0).to_degrees());
    let lat_j2000 = (geo.2 / (geo.0 * geo.0 + geo.1 * geo.1).sqrt())
        .atan()
        .to_degrees();
    let (ra_j2000, dec_j2000) =
        coords::ecliptic_to_equatorial(lon_j2000, lat_j2000, OBLIQUITY_J2000);
    let (ra_date, dec_date) = coords::precess_equatorial(ra_j2000, dec_j2000, J2000, jd_tt);
    let eps_date = nutation::mean_obliquity(jd_tt);
    let (mut lon, lat) = coords::equatorial_to_ecliptic(ra_date, dec_date, eps_date);

    let (dlon_ab, _) = aberration(lon, lat, jd_tt);
    lon += dlon_ab;

    let (dpsi, _) = nutation::nutation(jd_tt);
    lon += dpsi;

    Some(EclipticPos {
        lon_deg: normalize_deg(lon),
        lat_deg: lat,
        dist_au: dist,
    })
}

#[cfg(test)]
mod tests {
    use super::{apparent, heliocentric_j2000, supported_jd};

    /// Meeus example 37.a — 1992 October 13.0 TD (JD 2448908.5):
    /// heliocentric l = 232.74071°, b = 14.58782°, r = 29.711111 AU.
    #[test]
    fn meeus_example_37a() {
        let (lon, lat, radius) = heliocentric_j2000(2_448_908.5).unwrap();
        assert!((lon - 232.740_71).abs() < 2e-4, "l = {lon}");
        assert!((lat - 14.587_82).abs() < 2e-4, "b = {lat}");
        assert!((radius - 29.711_111).abs() < 1e-4, "r = {radius}");
    }

    /// The validity gate rejects out-of-span dates.
    #[test]
    fn hard_gate() {
        let (lo, hi) = supported_jd();
        assert!(apparent(lo - 10.0).is_none());
        assert!(apparent(hi + 10.0).is_none());
        assert!(apparent(crate::time::J2000).is_some());
    }

    /// Near the span edges the speed falls back to a one-sided
    /// difference instead of silently reporting zero (boundary regression
    /// finding). Note the light-time iteration (~0.18 days at Pluto's
    /// distance) trims the usable span inside the raw gate, so the
    /// low-edge probe sits past that margin; the high-edge probe at
    /// `hi − 0.01` genuinely exercises the one-sided fallback (its
    /// forward probe point is out-of-gate).
    #[test]
    fn speed_defined_at_span_edges() {
        use crate::bodies::Body;
        use crate::ephemeris::longitude_speed;
        let (lo, hi) = supported_jd();
        // Low-edge margin: Pluto was near aphelion (~49 AU) in 1885, so
        // the light-time retardation there is ~0.29 days — probe past it.
        for jd in [lo + 0.5, hi - 0.01] {
            let speed = longitude_speed(Body::Pluto, jd).expect("in-span speed");
            assert!(speed.abs() > 1e-4, "suspicious zero speed at {jd}");
        }
    }

    /// Pluto's heliocentric distance stays within its orbit's envelope.
    #[test]
    fn radius_envelope() {
        for step in 0..40 {
            let jd = 2_415_020.5 + f64::from(step) * 1900.0; // 1900–2108-ish, gated
            if let Some((_, _, radius)) = heliocentric_j2000(jd) {
                assert!((29.0..50.0).contains(&radius), "{radius}");
            }
        }
    }
}
