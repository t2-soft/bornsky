//! Geocentric position of the Moon — truncated ELP2000-82 as tabulated
//! in Meeus, *Astronomical Algorithms* (2nd ed.), ch. 47 (60 longitude/
//! distance terms + 60 latitude terms + planetary additives).
//!
//! Accuracy vs the full theory: ≈10″ in longitude, ≈4″ in latitude —
//! far below chart resolution. Tables ported from the MIT-licensed
//! `astro-rust` crate (Saurav Sachidanand), which transcribes the same
//! Meeus tables, adapted here to Rust tuples and degree-based calculations.
//! Copyright (c) 2015, 2016 Saurav Sachidanand; retain `licenses/astro-rust.txt`.
//! See `THIRD_PARTY_NOTICES.md` for the inherited table license.

use super::{EclipticPos, KM_PER_AU};
use crate::angle::normalize_deg;
use crate::nutation;
use crate::time::J2000;

/// (D, M, M′, F multipliers, sine coeff for longitude [1e-6 deg],
/// cosine coeff for distance [1e-3 km]) — Meeus table 47.A.
#[rustfmt::skip]
const LON_DIST_TERMS: &[(f64, f64, f64, f64, f64, f64)] = &[
    (0.0, 0.0, 1.0, 0.0, 6_288_774.0, -20_905_355.0),
    (2.0, 0.0, -1.0, 0.0, 1_274_027.0, -3_699_111.0),
    (2.0, 0.0, 0.0, 0.0, 658_314.0, -2_955_968.0),
    (0.0, 0.0, 2.0, 0.0, 213_618.0, -569_925.0),
    (0.0, 1.0, 0.0, 0.0, -185_116.0, 48_888.0),
    (0.0, 0.0, 0.0, 2.0, -114_332.0, -3_149.0),
    (2.0, 0.0, -2.0, 0.0, 58_793.0, 246_158.0),
    (2.0, -1.0, -1.0, 0.0, 57_066.0, -152_138.0),
    (2.0, 0.0, 1.0, 0.0, 53_322.0, -170_733.0),
    (2.0, -1.0, 0.0, 0.0, 45_758.0, -204_586.0),
    (0.0, 1.0, -1.0, 0.0, -40_923.0, -129_620.0),
    (1.0, 0.0, 0.0, 0.0, -34_720.0, 108_743.0),
    (0.0, 1.0, 1.0, 0.0, -30_383.0, 104_755.0),
    (2.0, 0.0, 0.0, -2.0, 15_327.0, 10_321.0),
    (0.0, 0.0, 1.0, 2.0, -12_528.0, 0.0),
    (0.0, 0.0, 1.0, -2.0, 10_980.0, 79_661.0),
    (4.0, 0.0, -1.0, 0.0, 10_675.0, -34_782.0),
    (0.0, 0.0, 3.0, 0.0, 10_034.0, -23_210.0),
    (4.0, 0.0, -2.0, 0.0, 8_548.0, -21_636.0),
    (2.0, 1.0, -1.0, 0.0, -7_888.0, 24_208.0),
    (2.0, 1.0, 0.0, 0.0, -6_766.0, 30_824.0),
    (1.0, 0.0, -1.0, 0.0, -5_163.0, -8_379.0),
    (1.0, 1.0, 0.0, 0.0, 4_987.0, -16_675.0),
    (2.0, -1.0, 1.0, 0.0, 4_036.0, -12_831.0),
    (2.0, 0.0, 2.0, 0.0, 3_994.0, -10_445.0),
    (4.0, 0.0, 0.0, 0.0, 3_861.0, -11_650.0),
    (2.0, 0.0, -3.0, 0.0, 3_665.0, 14_403.0),
    (0.0, 1.0, -2.0, 0.0, -2_689.0, -7_003.0),
    (2.0, 0.0, -1.0, 2.0, -2_602.0, 0.0),
    (2.0, -1.0, -2.0, 0.0, 2_390.0, 10_056.0),
    (1.0, 0.0, 1.0, 0.0, -2_348.0, 6_322.0),
    (2.0, -2.0, 0.0, 0.0, 2_236.0, -9_884.0),
    (0.0, 1.0, 2.0, 0.0, -2_120.0, 5_751.0),
    (0.0, 2.0, 0.0, 0.0, -2_069.0, 0.0),
    (2.0, -2.0, -1.0, 0.0, 2_048.0, -4_950.0),
    (2.0, 0.0, 1.0, -2.0, -1_773.0, 4_130.0),
    (2.0, 0.0, 0.0, 2.0, -1_595.0, 0.0),
    (4.0, -1.0, -1.0, 0.0, 1_215.0, -3_958.0),
    (0.0, 0.0, 2.0, 2.0, -1_110.0, 0.0),
    (3.0, 0.0, -1.0, 0.0, -892.0, 3_258.0),
    (2.0, 1.0, 1.0, 0.0, -810.0, 2_616.0),
    (4.0, -1.0, -2.0, 0.0, 759.0, -1_897.0),
    (0.0, 2.0, -1.0, 0.0, -713.0, -2_117.0),
    (2.0, 2.0, -1.0, 0.0, -700.0, 2_354.0),
    (2.0, 1.0, -2.0, 0.0, 691.0, 0.0),
    (2.0, -1.0, 0.0, -2.0, 596.0, 0.0),
    (4.0, 0.0, 1.0, 0.0, 549.0, -1_423.0),
    (0.0, 0.0, 4.0, 0.0, 537.0, -1_117.0),
    (4.0, -1.0, 0.0, 0.0, 520.0, -1_571.0),
    (1.0, 0.0, -2.0, 0.0, -487.0, -1_739.0),
    (2.0, 1.0, 0.0, -2.0, -399.0, 0.0),
    (0.0, 0.0, 2.0, -2.0, -381.0, -4_421.0),
    (1.0, 1.0, 1.0, 0.0, 351.0, 0.0),
    (3.0, 0.0, -2.0, 0.0, -340.0, 0.0),
    (4.0, 0.0, -3.0, 0.0, 330.0, 0.0),
    (2.0, -1.0, 2.0, 0.0, 327.0, 0.0),
    (0.0, 2.0, 1.0, 0.0, -323.0, 1_165.0),
    (1.0, 1.0, -1.0, 0.0, 299.0, 0.0),
    (2.0, 0.0, 3.0, 0.0, 294.0, 0.0),
    (2.0, 0.0, -1.0, -2.0, 0.0, 8_752.0),
];

/// (D, M, M′, F multipliers, sine coeff for latitude [1e-6 deg]) —
/// Meeus table 47.B.
#[rustfmt::skip]
const LAT_TERMS: &[(f64, f64, f64, f64, f64)] = &[
    (0.0, 0.0, 0.0, 1.0, 5_128_122.0),
    (0.0, 0.0, 1.0, 1.0, 280_602.0),
    (0.0, 0.0, 1.0, -1.0, 277_693.0),
    (2.0, 0.0, 0.0, -1.0, 173_237.0),
    (2.0, 0.0, -1.0, 1.0, 55_413.0),
    (2.0, 0.0, -1.0, -1.0, 46_271.0),
    (2.0, 0.0, 0.0, 1.0, 32_573.0),
    (0.0, 0.0, 2.0, 1.0, 17_198.0),
    (2.0, 0.0, 1.0, -1.0, 9_266.0),
    (0.0, 0.0, 2.0, -1.0, 8_822.0),
    (2.0, -1.0, 0.0, -1.0, 8_216.0),
    (2.0, 0.0, -2.0, -1.0, 4_324.0),
    (2.0, 0.0, 1.0, 1.0, 4_200.0),
    (2.0, 1.0, 0.0, -1.0, -3_359.0),
    (2.0, -1.0, -1.0, 1.0, 2_463.0),
    (2.0, -1.0, 0.0, 1.0, 2_211.0),
    (2.0, -1.0, -1.0, -1.0, 2_065.0),
    (0.0, 1.0, -1.0, -1.0, -1_870.0),
    (4.0, 0.0, -1.0, -1.0, 1_828.0),
    (0.0, 1.0, 0.0, 1.0, -1_794.0),
    (0.0, 0.0, 0.0, 3.0, -1_749.0),
    (0.0, 1.0, -1.0, 1.0, -1_565.0),
    (1.0, 0.0, 0.0, 1.0, -1_491.0),
    (0.0, 1.0, 1.0, 1.0, -1_475.0),
    (0.0, 1.0, 1.0, -1.0, -1_410.0),
    (0.0, 1.0, 0.0, -1.0, -1_344.0),
    (1.0, 0.0, 0.0, -1.0, -1_335.0),
    (0.0, 0.0, 3.0, 1.0, 1_107.0),
    (4.0, 0.0, 0.0, -1.0, 1_021.0),
    (4.0, 0.0, -1.0, 1.0, 833.0),
    (0.0, 0.0, 1.0, -3.0, 777.0),
    (4.0, 0.0, -2.0, 1.0, 671.0),
    (2.0, 0.0, 0.0, -3.0, 607.0),
    (2.0, 0.0, 2.0, -1.0, 596.0),
    (2.0, -1.0, 1.0, -1.0, 491.0),
    (2.0, 0.0, -2.0, 1.0, -451.0),
    (0.0, 0.0, 3.0, -1.0, 439.0),
    (2.0, 0.0, 2.0, 1.0, 422.0),
    (2.0, 0.0, -3.0, -1.0, 421.0),
    (2.0, 1.0, -1.0, 1.0, -366.0),
    (2.0, 1.0, 0.0, 1.0, -351.0),
    (4.0, 0.0, 0.0, 1.0, 331.0),
    (2.0, -1.0, 1.0, 1.0, 315.0),
    (2.0, -2.0, 0.0, -1.0, 302.0),
    (0.0, 0.0, 1.0, 3.0, -283.0),
    (2.0, 1.0, 1.0, -1.0, -229.0),
    (1.0, 1.0, 0.0, -1.0, 223.0),
    (1.0, 1.0, 0.0, 1.0, 223.0),
    (0.0, 1.0, -2.0, -1.0, -220.0),
    (2.0, 1.0, -1.0, -1.0, -220.0),
    (1.0, 0.0, 1.0, 1.0, -185.0),
    (2.0, -1.0, -2.0, -1.0, 181.0),
    (0.0, 1.0, 2.0, 1.0, -177.0),
    (4.0, 0.0, -2.0, -1.0, 176.0),
    (4.0, -1.0, -1.0, -1.0, 166.0),
    (1.0, 0.0, 1.0, -1.0, -164.0),
    (4.0, 0.0, 1.0, -1.0, 132.0),
    (1.0, 0.0, -1.0, -1.0, -119.0),
    (4.0, -1.0, 0.0, -1.0, 115.0),
    (2.0, -2.0, 0.0, 1.0, 107.0),
];

/// Geocentric **mean-equinox-of-date** ecliptic position (Meeus ch. 47);
/// the public [`apparent`] adds nutation in longitude.
#[must_use]
/// The Delaunay arguments (D, M, M′, F) in degrees at a TT Julian
/// century (Meeus 47.2–47.5) — shared by the position series here and
/// the true-node correction in [`super::points`].
pub(crate) fn delaunay_args(jc: f64) -> (f64, f64, f64, f64) {
    let elong = normalize_deg(
        297.850_192_1
            + 445_267.111_403_4 * jc
            + jc * jc * (-0.001_881_9 + jc * (1.0 / 545_868.0 - jc / 113_065_000.0)),
    );
    let sun_anom = normalize_deg(
        357.529_109_2 + 35_999.050_290_9 * jc + jc * jc * (-0.000_153_6 + jc / 24_490_000.0),
    );
    let moon_anom = normalize_deg(
        134.963_396_4
            + 477_198.867_505_5 * jc
            + jc * jc * (0.008_741_4 + jc * (1.0 / 69_699.0 - jc / 14_712_000.0)),
    );
    let arg_lat = normalize_deg(
        93.272_095_0
            + 483_202.017_523_3 * jc
            + jc * jc * (-0.003_653_9 + jc * (-1.0 / 3_526_000.0 + jc / 863_310_000.0)),
    );
    (elong, sun_anom, moon_anom, arg_lat)
}

#[must_use]
pub fn geometric(jd_tt: f64) -> EclipticPos {
    let jc = (jd_tt - J2000) / 36_525.0;

    // Mean longitude L′ and the Delaunay arguments (Meeus 47.1–47.5).
    let mean_lon = normalize_deg(
        218.316_447_7
            + 481_267.881_234_21 * jc
            + jc * jc * (-0.001_578_6 + jc * (1.0 / 538_841.0 - jc / 65_194_000.0)),
    );
    let (elong, sun_anom, moon_anom, arg_lat) = delaunay_args(jc);

    // Eccentricity damping for terms involving M (Meeus 47.6).
    let ecc = 1.0 - jc * (0.002_516 + jc * 0.000_007_4);

    // Planetary additives (Meeus ch. 47).
    let add1 = normalize_deg(119.75 + 131.849 * jc).to_radians();
    let add2 = normalize_deg(53.09 + 479_264.29 * jc).to_radians();
    let add3 = normalize_deg(313.45 + 481_266.484 * jc).to_radians();

    let (elong_r, sun_r, moon_r, lat_r, mean_r) = (
        elong.to_radians(),
        sun_anom.to_radians(),
        moon_anom.to_radians(),
        arg_lat.to_radians(),
        mean_lon.to_radians(),
    );

    let mut sum_lon = 0.0; // 1e-6 degrees
    let mut sum_dist = 0.0; // 1e-3 km
    for &(mul_d, mul_m, mul_mp, mul_f, coeff_lon, coeff_dist) in LON_DIST_TERMS {
        let arg = mul_d * elong_r + mul_m * sun_r + mul_mp * moon_r + mul_f * lat_r;
        let damp = match mul_m.abs() {
            m if (m - 1.0).abs() < 0.5 => ecc,
            m if (m - 2.0).abs() < 0.5 => ecc * ecc,
            _ => 1.0,
        };
        sum_lon += coeff_lon * damp * arg.sin();
        sum_dist += coeff_dist * damp * arg.cos();
    }

    let mut sum_lat = 0.0; // 1e-6 degrees
    for &(mul_d, mul_m, mul_mp, mul_f, coeff) in LAT_TERMS {
        let arg = mul_d * elong_r + mul_m * sun_r + mul_mp * moon_r + mul_f * lat_r;
        let damp = match mul_m.abs() {
            m if (m - 1.0).abs() < 0.5 => ecc,
            m if (m - 2.0).abs() < 0.5 => ecc * ecc,
            _ => 1.0,
        };
        sum_lat += coeff * damp * arg.sin();
    }

    // Additive corrections (Meeus ch. 47).
    sum_lon += 3958.0 * add1.sin() + 1962.0 * (mean_r - lat_r).sin() + 318.0 * add2.sin();
    sum_lat += -2235.0 * mean_r.sin()
        + 382.0 * add3.sin()
        + 175.0 * ((add1 - lat_r).sin() + (add1 + lat_r).sin())
        + 127.0 * (mean_r - moon_r).sin()
        - 115.0 * (mean_r + moon_r).sin();

    let dist_km = 385_000.56 + sum_dist / 1000.0;

    EclipticPos {
        lon_deg: normalize_deg(mean_lon + sum_lon / 1_000_000.0),
        lat_deg: sum_lat / 1_000_000.0,
        dist_au: dist_km / KM_PER_AU,
    }
}

/// Apparent geocentric ecliptic position (adds nutation in longitude).
#[must_use]
pub fn apparent(jd_tt: f64) -> EclipticPos {
    let mut pos = geometric(jd_tt);
    let (dpsi, _) = nutation::nutation(jd_tt);
    pos.lon_deg = normalize_deg(pos.lon_deg + dpsi);
    pos
}

#[cfg(test)]
mod tests {
    use super::{apparent, geometric};
    use crate::ephemeris::KM_PER_AU;

    /// Meeus example 47.a — 1992 April 12.0 TD (JD 2448724.5):
    /// λ = 133.162655°, β = −3.229126°, Δ = 368409.7 km;
    /// apparent λ = 133.167265°.
    #[test]
    fn meeus_example_47a() {
        let jd = 2_448_724.5;
        let geo = geometric(jd);
        assert!(
            (geo.lon_deg - 133.162_655).abs() < 2e-5,
            "λ = {}",
            geo.lon_deg
        );
        assert!(
            (geo.lat_deg - -3.229_126).abs() < 2e-5,
            "β = {}",
            geo.lat_deg
        );
        assert!(
            (geo.dist_au * KM_PER_AU - 368_409.7).abs() < 5.0,
            "Δ = {} km",
            geo.dist_au * KM_PER_AU
        );
        let app = apparent(jd);
        assert!(
            (app.lon_deg - 133.167_265).abs() < 5e-5,
            "apparent λ = {}",
            app.lon_deg
        );
    }

    /// The Moon's distance stays within its physical envelope.
    #[test]
    fn distance_envelope() {
        for step in 0..60 {
            let jd = crate::time::J2000 + f64::from(step) * 13.7;
            let km = geometric(jd).dist_au * KM_PER_AU;
            assert!((330_000.0..410_000.0).contains(&km), "{km}");
        }
    }
}
