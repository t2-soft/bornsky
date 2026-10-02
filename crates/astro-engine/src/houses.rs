//! Ascendant, Midheaven, Whole Sign, Equal, Porphyry, Regiomontanus and
//! Placidus houses, with flagged Porphyry fallbacks at extreme latitudes.
//! Inputs are ARMC (degrees), latitude (degrees) and true obliquity.

use crate::angle::normalize_deg;

/// A computed house frame: angles + twelve cusps (cusp 1 = index 0).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Houses {
    pub system: HouseSystem,
    pub ascendant: f64,
    pub midheaven: f64,
    pub cusps: [f64; 12],
    /// True when the requested system was replaced by the polar fallback.
    pub fell_back_to_porphyry: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HouseSystem {
    Placidus,
    WholeSign,
    Equal,
    Porphyry,
    /// Regiomontanus: the celestial equator divided into twelve equal
    /// 30° arcs by great circles through the north and south points of
    /// the horizon — the classical horary system (Lilly, *Christian
    /// Astrology*). Defined at every non-polar latitude; at |φ| ≥ 89.9°
    /// the pole heights degenerate and the frame falls back to Porphyry.
    Regiomontanus,
}

/// Ecliptic longitude of the Midheaven for a given ARMC and obliquity
/// (Meeus-standard formula, quadrant-safe).
#[must_use]
pub fn midheaven(armc_deg: f64, obliquity_deg: f64) -> f64 {
    let armc = armc_deg.to_radians();
    let eps = obliquity_deg.to_radians();
    normalize_deg(armc.sin().atan2(armc.cos() * eps.cos()).to_degrees())
}

/// Ecliptic longitude of the Ascendant (standard rising-point formula).
#[must_use]
pub fn ascendant(armc_deg: f64, lat_deg: f64, obliquity_deg: f64) -> f64 {
    let armc = armc_deg.to_radians();
    let lat = lat_deg.to_radians();
    let eps = obliquity_deg.to_radians();
    let lon = armc
        .cos()
        .atan2(-(armc.sin() * eps.cos() + lat.tan() * eps.sin()));
    normalize_deg(lon.to_degrees())
}

/// Ecliptic longitude of the point on the ecliptic with right ascension
/// `ra_deg` (used by the Placidus iteration; quadrant-safe).
fn ecliptic_lon_of_ra(ra_deg: f64, obliquity_deg: f64) -> f64 {
    let ra = ra_deg.to_radians();
    let eps = obliquity_deg.to_radians();
    normalize_deg(ra.sin().atan2(ra.cos() * eps.cos()).to_degrees())
}

/// Computes the house frame. `armc_deg` = local apparent sidereal time
/// in degrees; latitude north-positive.
#[must_use]
pub fn houses(system: HouseSystem, armc_deg: f64, lat_deg: f64, obliquity_deg: f64) -> Houses {
    let asc = ascendant(armc_deg, lat_deg, obliquity_deg);
    let mc = midheaven(armc_deg, obliquity_deg);

    match system {
        HouseSystem::WholeSign => {
            let start = (asc / 30.0).floor() * 30.0;
            let cusps = core::array::from_fn(|i| normalize_deg(start + 30.0 * f64::from(i as u8)));
            Houses {
                system,
                ascendant: asc,
                midheaven: mc,
                cusps,
                fell_back_to_porphyry: false,
            }
        }
        HouseSystem::Equal => {
            let cusps = core::array::from_fn(|i| normalize_deg(asc + 30.0 * f64::from(i as u8)));
            Houses {
                system,
                ascendant: asc,
                midheaven: mc,
                cusps,
                fell_back_to_porphyry: false,
            }
        }
        HouseSystem::Porphyry => Houses {
            system,
            ascendant: asc,
            midheaven: mc,
            cusps: porphyry_cusps(asc, mc),
            fell_back_to_porphyry: false,
        },
        HouseSystem::Regiomontanus => {
            match regiomontanus_cusps(armc_deg, lat_deg, obliquity_deg, asc, mc) {
                Some(cusps) => Houses {
                    system,
                    ascendant: asc,
                    midheaven: mc,
                    cusps,
                    fell_back_to_porphyry: false,
                },
                None => Houses {
                    system,
                    ascendant: asc,
                    midheaven: mc,
                    cusps: porphyry_cusps(asc, mc),
                    fell_back_to_porphyry: true,
                },
            }
        }
        HouseSystem::Placidus => match placidus_cusps(armc_deg, lat_deg, obliquity_deg, asc, mc) {
            Some(cusps) => Houses {
                system,
                ascendant: asc,
                midheaven: mc,
                cusps,
                fell_back_to_porphyry: false,
            },
            // Polar latitudes: Placidus semi-arcs degenerate — fall back
            // to Porphyry (the documented policy).
            None => Houses {
                system,
                ascendant: asc,
                midheaven: mc,
                cusps: porphyry_cusps(asc, mc),
                fell_back_to_porphyry: true,
            },
        },
    }
}

/// Porphyry: each quadrant between the angles trisected.
fn porphyry_cusps(asc: f64, mc: f64) -> [f64; 12] {
    let ic = normalize_deg(mc + 180.0);
    let desc = normalize_deg(asc + 180.0);

    // Arc from ASC forward (zodiacal order) to IC, trisected → cusps 2, 3;
    // arc from IC to DESC → cusps 5, 6; opposite cusps mirror.
    let arc_asc_ic = normalize_deg(ic - asc);
    let arc_ic_desc = normalize_deg(desc - ic);

    let mut cusps = [0.0; 12];
    cusps[0] = asc;
    cusps[1] = normalize_deg(asc + arc_asc_ic / 3.0);
    cusps[2] = normalize_deg(asc + 2.0 * arc_asc_ic / 3.0);
    cusps[3] = ic;
    cusps[4] = normalize_deg(ic + arc_ic_desc / 3.0);
    cusps[5] = normalize_deg(ic + 2.0 * arc_ic_desc / 3.0);
    for i in 6..12 {
        cusps[i] = normalize_deg(cusps[i - 6] + 180.0);
    }
    cusps
}

/// Regiomontanus cusps. Each intermediate cusp is the ecliptic point
/// on the house circle through the horizon's north and south points
/// that meets the equator `F` degrees of right ascension past the
/// meridian (F = 30°, 60° for cusps 11, 12; 120°, 150° for cusps 2, 3).
/// That circle's pole height above the horizon is
/// `tan P = tan φ · sin F`, and the cusp longitude follows the rising-
/// point formula with `P` in place of `φ` (for F = 90° it is the
/// Ascendant itself). Returns `None` at the geographic poles.
fn regiomontanus_cusps(
    armc_deg: f64,
    lat_deg: f64,
    obliquity_deg: f64,
    asc: f64,
    mc: f64,
) -> Option<[f64; 12]> {
    if lat_deg.abs() >= 89.9 {
        return None;
    }
    let lat = lat_deg.to_radians();
    let eps = obliquity_deg.to_radians();
    let cusp = |offset_deg: f64| {
        let ra = (armc_deg + offset_deg).to_radians();
        let tan_pole = lat.tan() * offset_deg.to_radians().sin();
        normalize_deg(
            ra.sin()
                .atan2(ra.cos() * eps.cos() - tan_pole * eps.sin())
                .to_degrees(),
        )
    };
    let mut cusps = [0.0; 12];
    cusps[0] = asc;
    cusps[1] = cusp(120.0);
    cusps[2] = cusp(150.0);
    cusps[3] = normalize_deg(mc + 180.0);
    cusps[9] = mc;
    cusps[10] = cusp(30.0);
    cusps[11] = cusp(60.0);
    for i in [4usize, 5, 6, 7, 8] {
        cusps[i] = normalize_deg(cusps[(i + 6) % 12] + 180.0);
    }
    Some(cusps)
}

/// Placidus semi-arc iteration. Returns `None` when the ascensional
/// difference is undefined (|tan φ · tan δ| > 1 — polar latitudes).
fn placidus_cusps(
    armc_deg: f64,
    lat_deg: f64,
    obliquity_deg: f64,
    asc: f64,
    mc: f64,
) -> Option<[f64; 12]> {
    // Intermediate cusps: (RA offset from ARMC at AD = 0, semi-arc
    // fraction, diurnal?) — cusp 11 sits 1/3 of the diurnal semi-arc
    // past the meridian, cusp 12 at 2/3; cusps 2 and 3 divide the
    // nocturnal semi-arc from the IC.
    let eleventh = placidus_iterate(armc_deg, lat_deg, obliquity_deg, 1.0 / 3.0, true)?;
    let twelfth = placidus_iterate(armc_deg, lat_deg, obliquity_deg, 2.0 / 3.0, true)?;
    let second = placidus_iterate(armc_deg, lat_deg, obliquity_deg, 2.0 / 3.0, false)?;
    let third = placidus_iterate(armc_deg, lat_deg, obliquity_deg, 1.0 / 3.0, false)?;

    let mut cusps = [0.0; 12];
    cusps[0] = asc;
    cusps[1] = second;
    cusps[2] = third;
    cusps[3] = normalize_deg(mc + 180.0);
    cusps[9] = mc;
    cusps[10] = eleventh;
    cusps[11] = twelfth;
    for i in [4usize, 5, 6, 7, 8] {
        cusps[i] = normalize_deg(cusps[(i + 6) % 12] + 180.0);
    }
    Some(cusps)
}

/// Iterates one Placidus cusp. `fraction` is the portion of the relevant
/// semi-arc; `diurnal` selects the arc above (cusps 11/12) or below
/// (cusps 2/3) the horizon on the eastern side.
fn placidus_iterate(
    armc_deg: f64,
    lat_deg: f64,
    obliquity_deg: f64,
    fraction: f64,
    diurnal: bool,
) -> Option<f64> {
    let lat = lat_deg.to_radians();
    let eps = obliquity_deg.to_radians();

    // Initial guess: AD = 0 (equatorial semi-arcs of 90°).
    let mut ra = if diurnal {
        armc_deg + 90.0 * fraction
    } else {
        armc_deg + 180.0 - 90.0 * fraction
    };

    for _ in 0..100 {
        // Declination of the ecliptic point with this RA:
        // tan δ = tan ε · sin RA.
        let dec = (eps.tan() * ra.to_radians().sin()).atan();
        let tan_product = dec.tan() * lat.tan();
        if tan_product.abs() > 1.0 {
            return None; // circumpolar — Placidus undefined here
        }
        let ascensional_diff = tan_product.asin().to_degrees();
        let next = if diurnal {
            armc_deg + (90.0 + ascensional_diff) * fraction
        } else {
            armc_deg + 180.0 - (90.0 - ascensional_diff) * fraction
        };
        let delta = (next - ra + 540.0).rem_euclid(360.0) - 180.0;
        ra = next;
        if delta.abs() < 1e-9 {
            break;
        }
    }
    Some(ecliptic_lon_of_ra(normalize_deg(ra), obliquity_deg))
}

#[cfg(test)]
mod tests {
    use super::{ascendant, houses, midheaven, HouseSystem};
    use crate::angle::normalize_deg;
    use proptest::prelude::*;

    const EPS: f64 = 23.439_291_1;

    /// At ARMC = 0 the MC is 0° Aries; at ARMC = 90 it is 0° Cancer.
    #[test]
    fn midheaven_anchors() {
        assert!(midheaven(0.0, EPS).abs() < 1e-9);
        assert!((midheaven(90.0, EPS) - 90.0).abs() < 1e-9);
        assert!((midheaven(180.0, EPS) - 180.0).abs() < 1e-9);
    }

    /// On the equator the Ascendant is exactly 90° of RA east of the
    /// meridian, mapped through the ecliptic.
    #[test]
    fn equator_ascendant() {
        let asc = ascendant(0.0, 0.0, EPS);
        // RA 90 on the ecliptic → λ = 90 (solstitial colure).
        assert!((asc - 90.0).abs() < 1e-9, "asc = {asc}");
    }

    /// Placidus at the equator: intermediate cusps collapse to equal RA
    /// divisions (AD = 0), and every opposite cusp pair is 180° apart.
    #[test]
    fn placidus_equator_and_opposites() {
        let h = houses(HouseSystem::Placidus, 40.0, 0.0, EPS);
        assert!(!h.fell_back_to_porphyry);
        for i in 0..6 {
            let diff = normalize_deg(h.cusps[i + 6] - h.cusps[i]);
            assert!(
                (diff - 180.0).abs() < 1e-6,
                "cusp {} vs {}: {diff}",
                i + 1,
                i + 7
            );
        }
        assert!((h.cusps[9] - h.midheaven).abs() < 1e-9);
        assert!((h.cusps[0] - h.ascendant).abs() < 1e-9);
    }

    /// Above the polar circle Placidus falls back to Porphyry, flagged.
    #[test]
    fn polar_fallback() {
        let h = houses(HouseSystem::Placidus, 100.0, 78.0, EPS);
        assert!(h.fell_back_to_porphyry);
        // Porphyry cusps still well-formed: cusp 10 = MC, cusp 1 = ASC.
        assert!((h.cusps[9] - h.midheaven).abs() < 1e-9);
        assert!((h.cusps[0] - h.ascendant).abs() < 1e-9);
    }

    /// Regiomontanus at the equator: pole heights are zero, so the house
    /// circles are hour circles and the cusps coincide with Placidus
    /// (both reduce to equal right-ascension divisions there).
    #[test]
    fn regiomontanus_equals_placidus_on_equator() {
        for armc in [0.0, 40.0, 133.0, 271.5] {
            let r = houses(HouseSystem::Regiomontanus, armc, 0.0, EPS);
            let p = houses(HouseSystem::Placidus, armc, 0.0, EPS);
            for i in 0..12 {
                let d = normalize_deg(r.cusps[i] - p.cusps[i] + 180.0) - 180.0;
                assert!(d.abs() < 1e-6, "armc {armc} cusp {}: {d}", i + 1);
            }
        }
    }

    /// Independent check: each Regiomontanus cusp lies on the great
    /// circle through the horizon's north/south points and the equator
    /// point `F` degrees of RA past the meridian — verified with plain
    /// vector geometry in the local horizon frame, not the closed form.
    #[test]
    fn regiomontanus_cusps_lie_on_house_circles() {
        let cases = [(40.0, 51.5), (200.0, -33.9), (310.0, 41.0), (95.0, 60.0)];
        for (armc, lat) in cases {
            let frame = houses(HouseSystem::Regiomontanus, armc, lat, EPS);
            assert!(!frame.fell_back_to_porphyry);
            for (index, offset) in [(10usize, 30.0), (11, 60.0), (1, 120.0), (2, 150.0)] {
                // Equatorial unit vector of the cusp's ecliptic point.
                let lon = frame.cusps[index].to_radians();
                let obliquity = EPS.to_radians();
                let cusp = (
                    lon.cos(),
                    lon.sin() * obliquity.cos(),
                    lon.sin() * obliquity.sin(),
                );
                // Equator point at RA = ARMC + F and the horizon's north
                // point (on the meridian, altitude 0, toward the pole).
                let ra = (armc + offset).to_radians();
                let equator = (ra.cos(), ra.sin(), 0.0);
                let (phi, meridian) = (lat.to_radians(), armc.to_radians());
                let north = (
                    -phi.sin() * meridian.cos(),
                    -phi.sin() * meridian.sin(),
                    phi.cos(),
                );
                // Normal of the house circle = north × equator point; the
                // cusp must be perpendicular to it.
                let normal = (
                    north.1 * equator.2 - north.2 * equator.1,
                    north.2 * equator.0 - north.0 * equator.2,
                    north.0 * equator.1 - north.1 * equator.0,
                );
                let dot = normal.0 * cusp.0 + normal.1 * cusp.1 + normal.2 * cusp.2;
                assert!(
                    dot.abs() < 1e-9,
                    "armc {armc} lat {lat} cusp {}: {dot}",
                    index + 1
                );
            }
        }
    }

    /// Whole-sign cusps are the twelve sign boundaries starting at the
    /// Ascendant's sign.
    #[test]
    fn whole_sign_shape() {
        let h = houses(HouseSystem::WholeSign, 250.0, 51.5, EPS);
        assert!((h.cusps[0] % 30.0).abs() < 1e-9);
        for i in 0..12 {
            let expected = normalize_deg(h.cusps[0] + 30.0 * f64::from(i as u8));
            assert!((h.cusps[i] - expected).abs() < 1e-9);
        }
    }

    proptest! {
        /// Placidus cusps march in zodiacal order (each successive cusp
        /// lies within the forward 180° of its predecessor) at
        /// non-polar latitudes, and MC/ASC always anchor cusps 10 and 1.
        #[test]
        fn placidus_wellformed(armc in 0.0f64..360.0, lat in -60.0f64..60.0) {
            let h = houses(HouseSystem::Placidus, armc, lat, EPS);
            prop_assert!(!h.fell_back_to_porphyry);
            for i in 0..12 {
                let step = normalize_deg(h.cusps[(i + 1) % 12] - h.cusps[i]);
                prop_assert!(step > 0.0 && step < 180.0,
                    "cusp {} -> {}: step {step}", i + 1, (i + 1) % 12 + 1);
            }
        }

        /// Regiomontanus cusps march in zodiacal order at non-polar
        /// latitudes and anchor the angles.
        #[test]
        fn regiomontanus_wellformed(armc in 0.0f64..360.0, lat in -66.0f64..66.0) {
            let h = houses(HouseSystem::Regiomontanus, armc, lat, EPS);
            prop_assert!(!h.fell_back_to_porphyry);
            prop_assert!((h.cusps[0] - h.ascendant).abs() < 1e-9);
            prop_assert!((h.cusps[9] - h.midheaven).abs() < 1e-9);
            for i in 0..12 {
                let step = normalize_deg(h.cusps[(i + 1) % 12] - h.cusps[i]);
                prop_assert!(step > 0.0 && step < 180.0,
                    "cusp {} -> {}: step {step}", i + 1, (i + 1) % 12 + 1);
            }
        }

        /// The Ascendant always lies in the half of the zodiac that is
        /// rising: it is within 180° forward of the IC.
        #[test]
        fn asc_between_mc_and_ic(armc in 0.0f64..360.0, lat in -60.0f64..60.0) {
            let asc = ascendant(armc, lat, EPS);
            let mc = midheaven(armc, EPS);
            let from_mc = normalize_deg(asc - mc);
            prop_assert!(from_mc > 0.0 && from_mc < 180.0,
                "asc {asc} not in rising half (mc {mc})");
        }
    }
}
