//! Computed lunar points: mean ascending node and mean apogee
//! ("Black Moon Lilith").
//!
//! Both are **mean elements** (smooth polynomials of time), quoted per
//! astrological convention without nutation. The mean-node polynomial is
//! Meeus ch. 47; the mean-apogee polynomial is the mean-perigee
//! development + 180°. Mean apogee remains provisional: the tests check
//! its range and motion, not independent positional accuracy.

use super::EclipticPos;
use crate::angle::normalize_deg;
use crate::time::J2000;

/// Mean lunar geocentric distance used for the point pseudo-positions.
const MEAN_DIST_AU: f64 = 385_000.56 / super::KM_PER_AU;

/// Mean ascending node Ω of the lunar orbit (Meeus ch. 47).
#[must_use]
pub fn mean_node(jd_tt: f64) -> EclipticPos {
    let jc = (jd_tt - J2000) / 36_525.0;
    let lon = 125.044_547_9 - 1_934.136_289_1 * jc
        + jc * jc * (0.002_075_4 + jc * (1.0 / 467_441.0 - jc / 60_616_000.0));
    EclipticPos {
        lon_deg: normalize_deg(lon),
        lat_deg: 0.0,
        dist_au: MEAN_DIST_AU,
    }
}

/// TRUE (osculating) ascending node: the mean node plus the periodic
/// correction series (Meeus ch. 47, the node passage) driven by the
/// same Delaunay arguments as the position series. The correction
/// stays within ±1.6°, but astrologers notice the difference (up to
/// 1.75° vs the mean node over a cycle) — the "true node" most modern
/// software displays.
///
/// Validated two ways (see the tests): golden values from Swiss
/// Ephemeris (the independent oracle), and self-consistently against
/// the engine's own lunar theory (the node is where the Moon's
/// latitude crosses zero ascending).
#[must_use]
pub fn true_node(jd_tt: f64) -> EclipticPos {
    let jc = (jd_tt - J2000) / 36_525.0;
    let (elong, sun_anom, moon_anom, arg_lat) = super::moon::delaunay_args(jc);
    let d = elong.to_radians();
    let m = sun_anom.to_radians();
    let mp = moon_anom.to_radians();
    let f = arg_lat.to_radians();
    // Coefficients: Meeus, Astronomical Algorithms (2nd ed.), ch. 47,
    // the true-node correction series at the chapter's end — the five
    // largest periodic terms in D, M, M′, F (degrees).
    let correction = -1.4979 * (2.0 * (d - f)).sin() - 0.1500 * m.sin() - 0.1226 * (2.0 * d).sin()
        + 0.1176 * (2.0 * f).sin()
        - 0.0801 * (2.0 * (mp - f)).sin();
    EclipticPos {
        lon_deg: normalize_deg(mean_node(jd_tt).lon_deg + correction),
        lat_deg: 0.0,
        dist_au: MEAN_DIST_AU,
    }
}

/// Mean lunar apogee = mean perigee + 180° (the astrological
/// "Black Moon Lilith" point).
#[must_use]
pub fn mean_apogee(jd_tt: f64) -> EclipticPos {
    let jc = (jd_tt - J2000) / 36_525.0;
    // Mean perigee development (Meeus ch. 50 lunar apsides mean args).
    let perigee = 83.353_246_5
        + 4_069.013_728_7 * jc
        + jc * jc * (-0.010_320_0 + jc * (-1.0 / 80_053.0 + jc / 18_999_000.0));
    EclipticPos {
        lon_deg: normalize_deg(perigee + 180.0),
        lat_deg: 0.0,
        dist_au: MEAN_DIST_AU,
    }
}

#[cfg(test)]
mod tests {
    use super::{mean_apogee, mean_node, true_node};
    use crate::ephemeris::moon;
    use crate::time::J2000;

    /// The node regresses ~0.0529°/day; the apogee advances ~0.1114°/day.
    #[test]
    fn characteristic_rates() {
        let node_rate = (mean_node(J2000 + 0.5).lon_deg - mean_node(J2000 - 0.5).lon_deg + 540.0)
            .rem_euclid(360.0)
            - 180.0;
        assert!((node_rate - -0.0529).abs() < 1e-3, "node rate {node_rate}");

        let apogee_rate = (mean_apogee(J2000 + 0.5).lon_deg - mean_apogee(J2000 - 0.5).lon_deg
            + 540.0)
            .rem_euclid(360.0)
            - 180.0;
        assert!(
            (apogee_rate - 0.1114).abs() < 1e-3,
            "apogee rate {apogee_rate}"
        );
    }

    /// Node longitude at J2000 is near its well-known value ~125.04°.
    #[test]
    fn node_at_j2000() {
        assert!((mean_node(J2000).lon_deg - 125.044_547_9).abs() < 1e-9);
    }

    /// The independent oracle: Swiss Ephemeris `TRUE_NODE` longitudes
    /// (pyswisseph 2.10.03, `swe.calc(jd_tt, TRUE_NODE, FLG_MOSEPH)` —
    /// the Moshier analytic ephemeris, no data files, deterministic).
    /// Epochs span 1900–2050: Meeus 47.a's date, J2000, 2025-01-01,
    /// 2050-01-01 and 1900-01-01 (all 0h TT). The budget is the
    /// truncation gap between Meeus' 5-term correction series and
    /// Swiss' osculating node — observed worst 11.9′ at 2448724.5, so
    /// 0.25° with margin. The mean node must NOT pass at this bound at
    /// the two epochs where the correction is large, so the test can
    /// tell the true node from the mean node it corrects.
    #[test]
    fn true_node_matches_swiss_ephemeris_goldens() {
        const SWISS: [(f64, f64); 5] = [
            (2_448_724.5, 273.539_748),
            (J2000, 123.953_329),
            (2_460_676.5, 0.874_694),
            (2_469_807.5, 239.499_085),
            (2_415_020.0, 260.272_612),
        ];
        let gap = |a: f64, b: f64| {
            let raw = (a - b).abs() % 360.0;
            raw.min(360.0 - raw)
        };
        for (jd_tt, swiss_lon) in SWISS {
            let ours = true_node(jd_tt).lon_deg;
            assert!(
                gap(ours, swiss_lon) < 0.25,
                "true node {ours}° vs Swiss {swiss_lon}° at {jd_tt}"
            );
        }
        // Discriminating power: where the correction is substantial,
        // the MEAN node fails this same bound.
        for (jd_tt, swiss_lon) in [SWISS[0], SWISS[3]] {
            assert!(
                gap(mean_node(jd_tt).lon_deg, swiss_lon) > 0.25,
                "mean node would pass at {jd_tt}; golden has no teeth"
            );
        }
    }

    /// The node is DEFINED as the ascending zero of the Moon's
    /// ecliptic latitude. Find that crossing with the engine's own
    /// lunar theory near several epochs; the TRUE node's longitude
    /// must sit far closer to the Moon's longitude at that instant
    /// than the mean node does, and within arcminutes absolutely.
    #[test]
    fn true_node_matches_the_latitude_zero_crossing() {
        for start in [
            J2000,
            J2000 + 4_000.5,
            J2000 - 7_300.5,
            2_461_289.5, // 2026-09-06
        ] {
            // Scan for an ascending latitude crossing (β < 0 → β ≥ 0)
            // within one node month, then bisect it tight.
            let mut lo = start;
            let mut hi = start;
            let mut found = false;
            for day in 0..28 {
                let a = moon::geometric(start + f64::from(day)).lat_deg;
                let b = moon::geometric(start + f64::from(day + 1)).lat_deg;
                if a < 0.0 && b >= 0.0 {
                    lo = start + f64::from(day);
                    hi = start + f64::from(day + 1);
                    found = true;
                    break;
                }
            }
            assert!(found, "no ascending crossing within a month of {start}");
            for _ in 0..50 {
                let mid = f64::midpoint(lo, hi);
                if moon::geometric(mid).lat_deg < 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let crossing = f64::midpoint(lo, hi);
            let moon_lon = moon::geometric(crossing).lon_deg;
            let gap = |node_lon: f64| {
                let raw = (moon_lon - node_lon).abs() % 360.0;
                raw.min(360.0 - raw)
            };
            let true_gap = gap(true_node(crossing).lon_deg);
            let mean_gap = gap(mean_node(crossing).lon_deg);
            // Absolute: the osculating node lands within arcminutes of
            // where the orbit actually crosses. Both sides are
            // truncated — the 5-term correction series AND the 60-term
            // latitude series — so ~10′ is the honest budget (observed
            // worst across epochs: ~5′).
            assert!(
                true_gap < 0.17,
                "true node off by {true_gap}° at {crossing}"
            );
            // Relative: it must beat the mean node whenever the mean
            // node is meaningfully off (the correction can pass through
            // zero, where both agree).
            assert!(
                true_gap <= mean_gap + 1e-9,
                "true {true_gap}° vs mean {mean_gap}° at {crossing}"
            );
        }
    }

    /// The correction is bounded by its series total (≈1.6°) and
    /// actually exercises both signs across a node month.
    #[test]
    fn true_node_correction_is_bounded_and_two_sided() {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for day in 0..28 {
            let jd = J2000 + f64::from(day);
            let raw = true_node(jd).lon_deg - mean_node(jd).lon_deg;
            let delta = if raw > 180.0 {
                raw - 360.0
            } else if raw < -180.0 {
                raw + 360.0
            } else {
                raw
            };
            assert!(delta.abs() <= 1.9, "correction {delta}° on day {day}");
            min = min.min(delta);
            max = max.max(delta);
        }
        assert!(min < 0.0 && max > 0.0, "one-sided: [{min}, {max}]");
    }
}
