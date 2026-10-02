//! Coordinate transformations: ecliptic ↔ equatorial, equatorial →
//! horizontal, and equatorial precession.
//!
//! All angles in degrees at the public boundary (the engine's convention).
//! Spherical transforms from Meeus, *Astronomical Algorithms* (2nd ed.),
//! ch. 13, with the book's worked examples as golden tests; precession is
//! the IAU 2006 model (see below) with ERFA test vectors as goldens.

use crate::angle::normalize_deg;

/// Ecliptic (λ, β) → equatorial (α, δ), all degrees, for obliquity ε.
/// (Meeus eq. 13.3/13.4.)
#[must_use]
pub fn ecliptic_to_equatorial(lon_deg: f64, lat_deg: f64, obliquity_deg: f64) -> (f64, f64) {
    let (lon, lat, eps) = (
        lon_deg.to_radians(),
        lat_deg.to_radians(),
        obliquity_deg.to_radians(),
    );
    let ra = (lon.sin() * eps.cos() - lat.tan() * eps.sin()).atan2(lon.cos());
    let dec = (lat.sin() * eps.cos() + lat.cos() * eps.sin() * lon.sin()).asin();
    (normalize_deg(ra.to_degrees()), dec.to_degrees())
}

/// Equatorial (α, δ) → ecliptic (λ, β), all degrees, for obliquity ε.
/// (Meeus eq. 13.1/13.2.)
#[must_use]
pub fn equatorial_to_ecliptic(ra_deg: f64, dec_deg: f64, obliquity_deg: f64) -> (f64, f64) {
    let (ra, dec, eps) = (
        ra_deg.to_radians(),
        dec_deg.to_radians(),
        obliquity_deg.to_radians(),
    );
    let lon = (ra.sin() * eps.cos() + dec.tan() * eps.sin()).atan2(ra.cos());
    let lat = (dec.sin() * eps.cos() - dec.cos() * eps.sin() * ra.sin()).asin();
    (normalize_deg(lon.to_degrees()), lat.to_degrees())
}

/// Equatorial (α, δ) → horizontal (azimuth, altitude) for an observer.
///
/// `sidereal_deg` is the local apparent sidereal time in degrees;
/// `lat_deg` the geographic latitude (north positive). Returns
/// `(azimuth_deg, altitude_deg)` with azimuth measured **from North,
/// through East** (the navigation convention; Meeus's from-South value
/// is `azimuth − 180°`). (Meeus eq. 13.5/13.6.)
#[must_use]
pub fn equatorial_to_horizontal(
    ra_deg: f64,
    dec_deg: f64,
    lat_deg: f64,
    sidereal_deg: f64,
) -> (f64, f64) {
    let hour_angle = (sidereal_deg - ra_deg).to_radians();
    let (dec, lat) = (dec_deg.to_radians(), lat_deg.to_radians());

    let az_from_south = hour_angle
        .sin()
        .atan2(hour_angle.cos() * lat.sin() - dec.tan() * lat.cos());
    let altitude = (lat.sin() * dec.sin() + lat.cos() * dec.cos() * hour_angle.cos()).asin();
    (
        normalize_deg(az_from_south.to_degrees() + 180.0),
        altitude.to_degrees(),
    )
}

// ── IAU 2006 precession  ───────────────────
// Fukushima–Williams bias+precession angles (Hilton et al. 2006,
// Cel. Mech. Dyn. Astron. 94, 351; the P03 solution of Capitaine,
// Wallace & Chapront 2003 as adopted by the IAU), ported from ERFA
// 2.0.x `eraPfw06`/`eraFw2m`/`eraPmat06` and CI-checked against the
// ERFA test vectors. The matrix carries frame bias (GCRS → J2000,
// ≈23 mas) with it — correct for the ICRS star catalog and far below
// chart resolution everywhere else.

/// The P03 bias+precession angles (γ̄, φ̄, ψ̄, `ε_A`) in radians at a TT JD.
#[allow(clippy::similar_names)] // gamb/phib/psib ARE the literature's names (ERFA/Hilton 2006)
fn pfw06_angles(jd_tt: f64) -> (f64, f64, f64, f64) {
    let t = (jd_tt - crate::time::J2000) / 36_525.0;
    let as2r = |seconds: f64| (seconds / 3600.0).to_radians();
    let gamb = as2r(
        -0.052_928
            + t * (10.556_378
                + t * (0.493_204_4
                    + t * (-0.000_312_38 + t * (-0.000_002_788 + t * 0.000_000_026_0)))),
    );
    let phib = as2r(
        84_381.412_819
            + t * (-46.811_016
                + t * (0.051_126_8
                    + t * (0.000_532_89 + t * (-0.000_000_440 + t * (-0.000_000_017_6))))),
    );
    let psib = as2r(
        -0.041_775
            + t * (5_038.481_484
                + t * (1.558_417_5
                    + t * (-0.000_185_22 + t * (-0.000_026_452 + t * (-0.000_000_014_8))))),
    );
    let epsa = crate::nutation::mean_obliquity(jd_tt).to_radians();
    (gamb, phib, psib, epsa)
}

type Mat3 = [[f64; 3]; 3];

/// GCRS → mean equator/equinox of date: `R₁(−ε_A)·R₃(−ψ̄)·R₁(φ̄)·R₃(γ̄)`
/// (ERFA `eraFw2m` composition for `eraPmat06`).
#[allow(clippy::similar_names)] // same literature names as pfw06_angles
fn pmat06(jd_tt: f64) -> Mat3 {
    let (gamb, phib, psib, epsa) = pfw06_angles(jd_tt);
    let m = rot_x(-epsa);
    let m = mat_mul(&m, &rot_z(-psib));
    let m = mat_mul(&m, &rot_x(phib));
    mat_mul(&m, &rot_z(gamb))
}

/// Rotation about the x-axis, ERFA's `eraRx` sign convention.
fn rot_x(angle: f64) -> Mat3 {
    let (s, c) = angle.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]]
}

/// Rotation about the z-axis, ERFA's `eraRz` sign convention.
fn rot_z(angle: f64) -> Mat3 {
    let (s, c) = angle.sin_cos();
    [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]]
}

/// Matrix product `a · b`.
fn mat_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// Precesses equatorial coordinates (degrees) from epoch `jd_from` to
/// epoch `jd_to` (both TT) with the IAU 2006 model: rotate the epoch
/// frame back to GCRS (transposed matrix), then forward to the target —
/// exact for any epoch pair, since the matrices are orthogonal.
#[must_use]
pub fn precess_equatorial(ra_deg: f64, dec_deg: f64, jd_from: f64, jd_to: f64) -> (f64, f64) {
    let (ra0, dec0) = (ra_deg.to_radians(), dec_deg.to_radians());
    let v = [dec0.cos() * ra0.cos(), dec0.cos() * ra0.sin(), dec0.sin()];

    let from = pmat06(jd_from);
    let to = pmat06(jd_to);
    // GCRS vector: fromᵀ · v (orthogonal ⇒ transpose = inverse).
    let g = [
        from[0][0] * v[0] + from[1][0] * v[1] + from[2][0] * v[2],
        from[0][1] * v[0] + from[1][1] * v[1] + from[2][1] * v[2],
        from[0][2] * v[0] + from[1][2] * v[1] + from[2][2] * v[2],
    ];
    let w = [
        to[0][0] * g[0] + to[0][1] * g[1] + to[0][2] * g[2],
        to[1][0] * g[0] + to[1][1] * g[1] + to[1][2] * g[2],
        to[2][0] * g[0] + to[2][1] * g[1] + to[2][2] * g[2],
    ];

    let ra = w[1].atan2(w[0]);
    // Rounding through two matrix products can push |w[2]| past 1.0 at
    // the poles; unclamped, asin would return NaN (regression fix).
    let dec = w[2].clamp(-1.0, 1.0).asin();
    (normalize_deg(ra.to_degrees()), dec.to_degrees())
}

#[cfg(test)]
mod tests {
    use super::{
        ecliptic_to_equatorial, equatorial_to_ecliptic, equatorial_to_horizontal,
        precess_equatorial,
    };
    use proptest::prelude::*;

    /// Meeus example 13.a — Pollux at J2000 mean obliquity 23.4392911°:
    /// α = 116.328942°, δ = 28.026183° ↔ λ = 113.215630°, β = 6.684170°.
    #[test]
    fn meeus_example_13a() {
        let eps = 23.439_291_1;
        let (lon, lat) = equatorial_to_ecliptic(116.328_942, 28.026_183, eps);
        assert!((lon - 113.215_630).abs() < 1e-5, "λ = {lon}");
        assert!((lat - 6.684_170).abs() < 1e-5, "β = {lat}");
        let (ra, dec) = ecliptic_to_equatorial(lon, lat, eps);
        assert!((ra - 116.328_942).abs() < 1e-9);
        assert!((dec - 28.026_183).abs() < 1e-9);
    }

    /// Meeus example 13.b — Venus from the US Naval Observatory
    /// (φ = 38.9213889°, longitude 77.0655556° W), 1987 April 10
    /// 19:21:00 UT: apparent α = 347.3193°, δ = −6.71989°;
    /// result A = 68.0337° (from South), h = 15.1249°.
    #[test]
    fn meeus_example_13b() {
        // Apparent Greenwich sidereal time at that instant is 8h34m57.0896s
        // (Meeus); local sidereal = GAST − west longitude.
        let gast_deg = (8.0 + 34.0 / 60.0 + 57.0896 / 3600.0) * 15.0;
        let lst = gast_deg - 77.065_555_6;
        let (az, alt) = equatorial_to_horizontal(347.3193, -6.719_89, 38.921_388_9, lst);
        // Our azimuth is from North: Meeus's 68.0337 + 180. Inputs are
        // quoted to 4 decimals (≈0.4″ quantum), so allow a few arcsec.
        assert!((az - (68.0337 + 180.0)).abs() < 1e-3, "A = {az}");
        assert!((alt - 15.1249).abs() < 1e-3, "h = {alt}");
    }

    /// Meeus example 21.b — θ Persei from J2000 to 2028 Nov 13.19 TT
    /// (JD 2462088.69). The book's starting values α = 41.054063°,
    /// δ = 49.227750° are ALREADY corrected for proper motion to the
    /// target epoch, so they feed straight into the precession.
    /// Expected (book, IAU 1976): α = 41.547214°, δ = 49.348483°.
    /// IAU 2006 + frame bias differs from the 1976 answer by ≲0.1″,
    /// hence the 3.6e-5° (0.13″) tolerance.
    #[test]
    fn meeus_example_21b() {
        let (ra, dec) =
            precess_equatorial(41.054_063, 49.227_750, crate::time::J2000, 2_462_088.69);
        assert!((ra - 41.547_214).abs() < 3.6e-5, "α = {ra}");
        assert!((dec - 49.348_483).abs() < 3.6e-5, "δ = {dec}");
    }

    /// ERFA test vector for `eraPmat06` at JD 2400000.5+50123.9999 — all
    /// nine matrix elements, tolerances 1e-12 (diagonal) / 1e-14.
    /// The expected values keep ERFA's digits verbatim (beyond f64
    /// precision — the compiler rounds them, exactly as C does).
    #[test]
    #[allow(clippy::excessive_precision)]
    fn erfa_pmat06_vector() {
        let m = super::pmat06(2_400_000.5 + 50_123.999_9);
        let expected = [
            [
                0.999_999_550_517_600_704_7,
                0.869_540_461_734_820_840_6e-3,
                0.377_973_520_186_558_910_4e-3,
            ],
            [
                -0.869_540_472_377_203_141_4e-3,
                0.999_999_621_949_602_716_1,
                -0.136_175_249_708_027_014_3e-6,
            ],
            [
                -0.377_973_495_703_408_949_0e-3,
                -0.192_488_084_789_445_711_3e-6,
                0.999_999_928_567_997_195_8,
            ],
        ];
        for i in 0..3 {
            for j in 0..3 {
                let tol = if i == j { 1e-12 } else { 1e-14 };
                assert!(
                    (m[i][j] - expected[i][j]).abs() < tol,
                    "m[{i}][{j}] = {:e}, expected {:e}",
                    m[i][j],
                    expected[i][j]
                );
            }
        }
    }

    /// ERFA test vector for `eraPfw06` at the same epoch.
    #[test]
    #[allow(clippy::similar_names, clippy::excessive_precision)]
    fn erfa_pfw06_vector() {
        let (gamb, phib, psib, epsa) = super::pfw06_angles(2_400_000.5 + 50_123.999_9);
        assert!(
            (gamb - -0.224_338_767_099_799_569_0e-5).abs() < 1e-16,
            "γ̄ = {gamb:e}"
        );
        assert!(
            (phib - 0.409_101_460_239_131_280_8).abs() < 1e-12,
            "φ̄ = {phib}"
        );
        assert!(
            (psib - -0.950_195_417_801_303_189_5e-3).abs() < 1e-14,
            "ψ̄ = {psib:e}"
        );
        assert!(
            (epsa - 0.409_101_431_658_736_749_1).abs() < 1e-12,
            "ε_A = {epsa}"
        );
    }

    proptest! {
        /// Ecliptic↔equatorial round-trips to numerical identity.
        #[test]
        fn ecl_equ_roundtrip(lon in 0.0f64..360.0, lat in -89.0f64..89.0, eps in 22.0f64..25.0) {
            let (ra, dec) = ecliptic_to_equatorial(lon, lat, eps);
            let (lon2, lat2) = equatorial_to_ecliptic(ra, dec, eps);
            let dlon = (lon2 - lon + 540.0).rem_euclid(360.0) - 180.0;
            prop_assert!(dlon.abs() < 1e-9 && (lat2 - lat).abs() < 1e-9);
        }

        /// Precessing forward then backward returns the start point.
        #[test]
        fn precession_roundtrip(ra in 0.0f64..360.0, dec in -85.0f64..85.0, dt in -300.0f64..300.0) {
            let jd_to = crate::time::J2000 + dt * 365.25;
            let (ra1, dec1) = precess_equatorial(ra, dec, crate::time::J2000, jd_to);
            let (ra2, dec2) = precess_equatorial(ra1, dec1, jd_to, crate::time::J2000);
            let dra = (ra2 - ra + 540.0).rem_euclid(360.0) - 180.0;
            prop_assert!(dra.abs() < 1e-6 && (dec2 - dec).abs() < 1e-6);
        }
    }
}
