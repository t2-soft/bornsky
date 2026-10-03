//! Nutation and obliquity of the ecliptic.
//!
//! Nutation: the **IAU 2000B** model (`McCarthy & Luzum 2003`, Cel. Mech.
//! Dyn. Astron. 85, 37) — the 77-term luni-solar series plus the fixed
//! planetary-bias offsets, ported from ERFA 2.0.x `eraNut00b` (BSD-style
//! license; ERFA is derived from SOFA, not the SOFA reference implementation)
//! and CI-checked against the ERFA test vector. Pole accurate
//! to 1 mas over 1900–2100 (the full 2000A
//! model's remaining ~1 mas would be invisible at chart scale).
//!
//! Mean obliquity: the **IAU 2006** polynomial (Capitaine, Wallace &
//! Chapront 2003 "P03"; adopted IAU 2006), ported from ERFA `eraObl06`
//! and CI-checked against its vector.
//!
//! Meeus example 22.a is kept as a golden with a tolerance wide enough
//! for the difference from the older IAU 1980 model (approximately 10 mas).
//! ERFA attribution and redistribution terms are in `THIRD_PARTY_NOTICES.md`.
//! Adapted to Rust and degree-based inputs/outputs from ERFA v2.0.1.
//! Copyright (C) 2013-2021 `NumFOCUS` Foundation; retain `licenses/ERFA.txt`.

use crate::time::J2000;

/// Arcseconds in a full turn.
const TURNAS: f64 = 1_296_000.0;
/// Arcseconds → radians.
const AS2R: f64 = core::f64::consts::PI / (180.0 * 3600.0);
/// Units of 0.1 microarcsecond → radians.
const U2R: f64 = AS2R / 1e7;
/// Milliarcseconds → radians.
const MAS2R: f64 = AS2R / 1e3;

/// One luni-solar term: (l, l′, F, D, Ω multipliers; Δψ sin, Δψ·T sin,
/// Δψ cos; Δε cos, Δε·T cos, Δε sin coefficients in 0.1 µas).
/// Table: ERFA `eraNut00b` (machine-extracted, not hand-copied).
type SeriesRow = (i8, i8, i8, i8, i8, f64, f64, f64, f64, f64, f64);
#[rustfmt::skip]
const TERMS: &[SeriesRow] = &[
    (0, 0, 0, 0, 1, -172_064_161.0, -174_666.0, 33_386.0, 92_052_331.0, 9086.0, 15_377.0),
    (0, 0, 2, -2, 2, -13_170_906.0, -1675.0, -13_696.0, 5_730_336.0, -3015.0, -4587.0),
    (0, 0, 2, 0, 2, -2_276_413.0, -234.0, 2796.0, 978_459.0, -485.0, 1374.0),
    (0, 0, 0, 0, 2, 2_074_554.0, 207.0, -698.0, -897_492.0, 470.0, -291.0),
    (0, 1, 0, 0, 0, 1_475_877.0, -3633.0, 11_817.0, 73_871.0, -184.0, -1924.0),
    (0, 1, 2, -2, 2, -516_821.0, 1226.0, -524.0, 224_386.0, -677.0, -174.0),
    (1, 0, 0, 0, 0, 711_159.0, 73.0, -872.0, -6750.0, 0.0, 358.0),
    (0, 0, 2, 0, 1, -387_298.0, -367.0, 380.0, 200_728.0, 18.0, 318.0),
    (1, 0, 2, 0, 2, -301_461.0, -36.0, 816.0, 129_025.0, -63.0, 367.0),
    (0, -1, 2, -2, 2, 215_829.0, -494.0, 111.0, -95_929.0, 299.0, 132.0),
    (0, 0, 2, -2, 1, 128_227.0, 137.0, 181.0, -68_982.0, -9.0, 39.0),
    (-1, 0, 2, 0, 2, 123_457.0, 11.0, 19.0, -53_311.0, 32.0, -4.0),
    (-1, 0, 0, 2, 0, 156_994.0, 10.0, -168.0, -1235.0, 0.0, 82.0),
    (1, 0, 0, 0, 1, 63_110.0, 63.0, 27.0, -33_228.0, 0.0, -9.0),
    (-1, 0, 0, 0, 1, -57_976.0, -63.0, -189.0, 31_429.0, 0.0, -75.0),
    (-1, 0, 2, 2, 2, -59_641.0, -11.0, 149.0, 25_543.0, -11.0, 66.0),
    (1, 0, 2, 0, 1, -51_613.0, -42.0, 129.0, 26_366.0, 0.0, 78.0),
    (-2, 0, 2, 0, 1, 45_893.0, 50.0, 31.0, -24_236.0, -10.0, 20.0),
    (0, 0, 0, 2, 0, 63_384.0, 11.0, -150.0, -1220.0, 0.0, 29.0),
    (0, 0, 2, 2, 2, -38_571.0, -1.0, 158.0, 16_452.0, -11.0, 68.0),
    (0, -2, 2, -2, 2, 32_481.0, 0.0, 0.0, -13_870.0, 0.0, 0.0),
    (-2, 0, 0, 2, 0, -47_722.0, 0.0, -18.0, 477.0, 0.0, -25.0),
    (2, 0, 2, 0, 2, -31_046.0, -1.0, 131.0, 13_238.0, -11.0, 59.0),
    (1, 0, 2, -2, 2, 28_593.0, 0.0, -1.0, -12_338.0, 10.0, -3.0),
    (-1, 0, 2, 0, 1, 20_441.0, 21.0, 10.0, -10_758.0, 0.0, -3.0),
    (2, 0, 0, 0, 0, 29_243.0, 0.0, -74.0, -609.0, 0.0, 13.0),
    (0, 0, 2, 0, 0, 25_887.0, 0.0, -66.0, -550.0, 0.0, 11.0),
    (0, 1, 0, 0, 1, -14_053.0, -25.0, 79.0, 8551.0, -2.0, -45.0),
    (-1, 0, 0, 2, 1, 15_164.0, 10.0, 11.0, -8001.0, 0.0, -1.0),
    (0, 2, 2, -2, 2, -15_794.0, 72.0, -16.0, 6850.0, -42.0, -5.0),
    (0, 0, -2, 2, 0, 21_783.0, 0.0, 13.0, -167.0, 0.0, 13.0),
    (1, 0, 0, -2, 1, -12_873.0, -10.0, -37.0, 6953.0, 0.0, -14.0),
    (0, -1, 0, 0, 1, -12_654.0, 11.0, 63.0, 6415.0, 0.0, 26.0),
    (-1, 0, 2, 2, 1, -10_204.0, 0.0, 25.0, 5222.0, 0.0, 15.0),
    (0, 2, 0, 0, 0, 16_707.0, -85.0, -10.0, 168.0, -1.0, 10.0),
    (1, 0, 2, 2, 2, -7691.0, 0.0, 44.0, 3268.0, 0.0, 19.0),
    (-2, 0, 2, 0, 0, -11_024.0, 0.0, -14.0, 104.0, 0.0, 2.0),
    (0, 1, 2, 0, 2, 7566.0, -21.0, -11.0, -3250.0, 0.0, -5.0),
    (0, 0, 2, 2, 1, -6637.0, -11.0, 25.0, 3353.0, 0.0, 14.0),
    (0, -1, 2, 0, 2, -7141.0, 21.0, 8.0, 3070.0, 0.0, 4.0),
    (0, 0, 0, 2, 1, -6302.0, -11.0, 2.0, 3272.0, 0.0, 4.0),
    (1, 0, 2, -2, 1, 5800.0, 10.0, 2.0, -3045.0, 0.0, -1.0),
    (2, 0, 2, -2, 2, 6443.0, 0.0, -7.0, -2768.0, 0.0, -4.0),
    (-2, 0, 0, 2, 1, -5774.0, -11.0, -15.0, 3041.0, 0.0, -5.0),
    (2, 0, 2, 0, 1, -5350.0, 0.0, 21.0, 2695.0, 0.0, 12.0),
    (0, -1, 2, -2, 1, -4752.0, -11.0, -3.0, 2719.0, 0.0, -3.0),
    (0, 0, 0, -2, 1, -4940.0, -11.0, -21.0, 2720.0, 0.0, -9.0),
    (-1, -1, 0, 2, 0, 7350.0, 0.0, -8.0, -51.0, 0.0, 4.0),
    (2, 0, 0, -2, 1, 4065.0, 0.0, 6.0, -2206.0, 0.0, 1.0),
    (1, 0, 0, 2, 0, 6579.0, 0.0, -24.0, -199.0, 0.0, 2.0),
    (0, 1, 2, -2, 1, 3579.0, 0.0, 5.0, -1900.0, 0.0, 1.0),
    (1, -1, 0, 0, 0, 4725.0, 0.0, -6.0, -41.0, 0.0, 3.0),
    (-2, 0, 2, 0, 2, -3075.0, 0.0, -2.0, 1313.0, 0.0, -1.0),
    (3, 0, 2, 0, 2, -2904.0, 0.0, 15.0, 1233.0, 0.0, 7.0),
    (0, -1, 0, 2, 0, 4348.0, 0.0, -10.0, -81.0, 0.0, 2.0),
    (1, -1, 2, 0, 2, -2878.0, 0.0, 8.0, 1232.0, 0.0, 4.0),
    (0, 0, 0, 1, 0, -4230.0, 0.0, 5.0, -20.0, 0.0, -2.0),
    (-1, -1, 2, 2, 2, -2819.0, 0.0, 7.0, 1207.0, 0.0, 3.0),
    (-1, 0, 2, 0, 0, -4056.0, 0.0, 5.0, 40.0, 0.0, -2.0),
    (0, -1, 2, 2, 2, -2647.0, 0.0, 11.0, 1129.0, 0.0, 5.0),
    (-2, 0, 0, 0, 1, -2294.0, 0.0, -10.0, 1266.0, 0.0, -4.0),
    (1, 1, 2, 0, 2, 2481.0, 0.0, -7.0, -1062.0, 0.0, -3.0),
    (2, 0, 0, 0, 1, 2179.0, 0.0, -2.0, -1129.0, 0.0, -2.0),
    (-1, 1, 0, 1, 0, 3276.0, 0.0, 1.0, -9.0, 0.0, 0.0),
    (1, 1, 0, 0, 0, -3389.0, 0.0, 5.0, 35.0, 0.0, -2.0),
    (1, 0, 2, 0, 0, 3339.0, 0.0, -13.0, -107.0, 0.0, 1.0),
    (-1, 0, 2, -2, 1, -1987.0, 0.0, -6.0, 1073.0, 0.0, -2.0),
    (1, 0, 0, 0, 2, -1981.0, 0.0, 0.0, 854.0, 0.0, 0.0),
    (-1, 0, 0, 1, 0, 4026.0, 0.0, -353.0, -553.0, 0.0, -139.0),
    (0, 0, 2, 1, 2, 1660.0, 0.0, -5.0, -710.0, 0.0, -2.0),
    (-1, 0, 2, 4, 2, -1521.0, 0.0, 9.0, 647.0, 0.0, 4.0),
    (-1, 1, 0, 1, 1, 1314.0, 0.0, 0.0, -700.0, 0.0, 0.0),
    (0, -2, 2, -2, 1, -1283.0, 0.0, 0.0, 672.0, 0.0, 0.0),
    (1, 0, 2, 2, 1, -1331.0, 0.0, 8.0, 663.0, 0.0, 4.0),
    (-2, 0, 2, 2, 2, 1383.0, 0.0, -2.0, -594.0, 0.0, -2.0),
    (-1, 0, 0, 0, 2, 1405.0, 0.0, 4.0, -610.0, 0.0, 2.0),
    (1, 1, 2, -2, 2, 1290.0, 0.0, 0.0, -556.0, 0.0, 0.0),
];

/// Nutation in longitude and obliquity, `(Δψ, Δε)` in **degrees**, at a
/// TT Julian Day. IAU 2000B: luni-solar series + fixed planetary offsets.
#[must_use]
pub fn nutation(jd_tt: f64) -> (f64, f64) {
    let t = (jd_tt - J2000) / 36_525.0;

    // Fundamental (Delaunay) arguments, Simon et al. (1994), linear
    // rates only — exactly the abridged-model convention (ERFA nut00b).
    // Arcseconds → radians after reduction mod one turn.
    let el = (485_868.249_036 + 1_717_915_923.217_8 * t).rem_euclid(TURNAS) * AS2R;
    let elp = (1_287_104.793_05 + 129_596_581.048_1 * t).rem_euclid(TURNAS) * AS2R;
    let f = (335_779.526_232 + 1_739_527_262.847_8 * t).rem_euclid(TURNAS) * AS2R;
    let d = (1_072_260.703_69 + 1_602_961_601.209 * t).rem_euclid(TURNAS) * AS2R;
    let om = (450_160.398_036 - 6_962_890.543_1 * t).rem_euclid(TURNAS) * AS2R;

    let mut dpsi = 0.0; // 0.1 µas
    let mut deps = 0.0;

    // Smallest terms first (the ERFA summation order).
    for &(nl, nlp, nf, nd, nom, ps, pst, pc, ec, ect, es) in TERMS.iter().rev() {
        let arg = f64::from(nl) * el
            + f64::from(nlp) * elp
            + f64::from(nf) * f
            + f64::from(nd) * d
            + f64::from(nom) * om;
        let (sarg, carg) = arg.sin_cos();
        dpsi += (ps + pst * t) * sarg + pc * carg;
        deps += (ec + ect * t) * carg + es * sarg;
    }

    // Fixed offsets in lieu of the planetary nutation terms (Luzum 2001
    // values, optimized for the rigorous bias/precession/nutation chain).
    let dpsi_rad = dpsi * U2R + (-0.135 * MAS2R);
    let deps_rad = deps * U2R + (0.388 * MAS2R);

    (dpsi_rad.to_degrees(), deps_rad.to_degrees())
}

/// Mean obliquity of the ecliptic ε₀ in degrees — IAU 2006 (P03).
#[must_use]
pub fn mean_obliquity(jd_tt: f64) -> f64 {
    let t = (jd_tt - J2000) / 36_525.0;
    let seconds = 84_381.406
        + t * (-46.836_769
            + t * (-0.000_183_1
                + t * (0.002_003_40 + t * (-0.000_000_576 + t * (-0.000_000_043_4)))));
    seconds / 3600.0
}

/// True obliquity ε = ε₀ + Δε in degrees.
#[must_use]
pub fn true_obliquity(jd_tt: f64) -> f64 {
    mean_obliquity(jd_tt) + nutation(jd_tt).1
}

#[cfg(test)]
mod tests {
    use super::{mean_obliquity, nutation, true_obliquity, AS2R};

    /// ERFA test vector for `eraNut00b` at JD 2400000.5+53736.0,
    /// tolerance 1e-13 rad (`t_erfa_c.c`). Expected values keep ERFA's
    /// digits verbatim (beyond f64 precision; the compiler rounds them
    /// exactly as C does).
    #[test]
    #[allow(clippy::excessive_precision)]
    fn erfa_nut00b_vector() {
        let (dpsi_deg, deps_deg) = nutation(2_400_000.5 + 53_736.0);
        let dpsi = dpsi_deg.to_radians();
        let deps = deps_deg.to_radians();
        assert!(
            (dpsi - -0.963_255_229_114_836_278_3e-5).abs() < 1e-13,
            "Δψ = {dpsi:e}"
        );
        assert!(
            (deps - 0.406_319_710_662_115_936_7e-4).abs() < 1e-13,
            "Δε = {deps:e}"
        );
    }

    /// ERFA test vector for `eraObl06` at JD 2400000.5+54388.0,
    /// tolerance 1e-14 rad.
    #[test]
    #[allow(clippy::excessive_precision)]
    fn erfa_obl06_vector() {
        let eps = mean_obliquity(2_400_000.5 + 54_388.0).to_radians();
        assert!(
            (eps - 0.409_074_922_938_725_820_4).abs() < 1e-14,
            "ε₀ = {eps}"
        );
    }

    /// Meeus example 22.a — 1987 April 10.0 TD (JD 2446895.5). The book
    /// tabulates the IAU 1980 answer (Δψ = −3.788″, Δε = +9.443″, from
    /// which ε = 23°26′36″.850); IAU 2000B agrees within ~10 mas, so the
    /// 50 mas tolerance still pins the result.
    #[test]
    fn meeus_example_22a() {
        let jd = 2_446_895.5;
        let (dpsi, deps) = nutation(jd);
        assert!(
            (dpsi * 3600.0 - -3.788).abs() < 0.05,
            "Δψ = {}″, expected −3.788″",
            dpsi * 3600.0
        );
        assert!(
            (deps * 3600.0 - 9.443).abs() < 0.05,
            "Δε = {}″, expected 9.443″",
            deps * 3600.0
        );
        let eps = true_obliquity(jd);
        let expected_eps = 23.0 + 26.0 / 60.0 + 36.850 / 3600.0;
        assert!((eps - expected_eps).abs() < 0.06 / 3600.0);
    }

    /// J2000 mean obliquity is the IAU 2006 value 84381.406″
    /// (23.439279444°) — NOT the older Lieske 84381.448″.
    #[test]
    fn j2000_obliquity() {
        let eps = mean_obliquity(crate::time::J2000);
        assert!((eps - 84_381.406 / 3600.0).abs() < 1e-9, "got {eps}");
    }

    /// Nutation in longitude stays within ±20″ (its physical envelope).
    #[test]
    fn nutation_envelope() {
        for step in 0..200 {
            let jd = 2_415_020.5 + f64::from(step) * 200.0; // 1900–2009
            let (dpsi, deps) = nutation(jd);
            assert!(dpsi.abs() * 3600.0 < 20.0);
            assert!(deps.abs() * 3600.0 < 12.0);
        }
    }

    /// The unit ladder itself: 1″ in radians.
    #[test]
    fn arcsec_constant() {
        assert!((AS2R - 4.848_136_811_095_36e-6).abs() < 1e-18);
    }
}
