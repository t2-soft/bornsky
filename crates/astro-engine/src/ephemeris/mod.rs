//! Geocentric apparent positions — the reduction pipeline.
//!
//! Planets: heliocentric VSOP87D (full series via the `vsop87` crate,
//! MIT/Apache) → geocentric with light-time iteration → FK5 frame
//! correction → annual aberration → nutation in longitude. Method:
//! Meeus, *Astronomical Algorithms* (2nd ed.), ch. 25/26/32/33.
//!
//! Moon: truncated ELP2000-82 (Meeus ch. 47) in [`moon`].
//! Pluto: Goffin 1986 series (Meeus ch. 37) in [`pluto`], J2000 →
//! of-date via equatorial precession.
//! Mean node / mean apogee: [`points`].

pub mod moon;
pub mod pluto;
pub mod points;

use crate::angle::normalize_deg;
use crate::bodies::Body;
use crate::nutation;
use crate::time::J2000;

/// Kilometres per astronomical unit (IAU 2012).
pub const KM_PER_AU: f64 = 149_597_870.7;

/// Light-time in days for a distance of 1 AU.
const LIGHT_TIME_PER_AU: f64 = 0.005_775_518_3;

/// Constant of aberration, arcseconds.
const ABERRATION_KAPPA: f64 = 20.495_52;

/// A geocentric apparent ecliptic position (equinox of date).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EclipticPos {
    /// Apparent ecliptic longitude, degrees `[0, 360)`.
    pub lon_deg: f64,
    /// Ecliptic latitude, degrees.
    pub lat_deg: f64,
    /// Geocentric distance, AU.
    pub dist_au: f64,
}

/// Geocentric apparent position of a body at a TT Julian Day.
///
/// Returns `None` only for [`Body::Pluto`] outside the Goffin series'
/// validity span (see [`pluto::supported_jd`]).
#[must_use]
pub fn apparent(body: Body, jd_tt: f64) -> Option<EclipticPos> {
    match body {
        Body::Sun => Some(sun_apparent(jd_tt)),
        Body::Moon => Some(moon::apparent(jd_tt)),
        Body::Pluto => pluto::apparent(jd_tt),
        Body::MeanNode => Some(points::mean_node(jd_tt)),
        Body::MeanApogee => Some(points::mean_apogee(jd_tt)),
        planet => Some(planet_apparent(planet, jd_tt)),
    }
}

/// Ecliptic-longitude speed in degrees/day (central difference, with a
/// one-sided fallback when a probe point falls outside a body's validity
/// span — e.g. within 0.05 days of Pluto's 1885/2099 gate — so a body
/// that is available at `jd_tt` always gets a real speed, boundary regression fix). Positive = direct, negative = retrograde.
#[must_use]
pub fn longitude_speed(body: Body, jd_tt: f64) -> Option<f64> {
    longitude_speed_from(body, jd_tt, apparent(body, jd_tt)?)
}

/// [`longitude_speed`] for a caller that already holds the body's position at
/// `jd_tt`, so the centre is not evaluated a second time. `center` must be
/// `apparent(body, jd_tt)`; the answer is then [`longitude_speed`]'s exactly.
pub(crate) fn longitude_speed_from(body: Body, jd_tt: f64, center: EclipticPos) -> Option<f64> {
    let step = 0.05;
    let before = apparent(body, jd_tt - step);
    let after = apparent(body, jd_tt + step);

    let span = |from: EclipticPos, to: EclipticPos, days: f64| {
        let delta = (to.lon_deg - from.lon_deg + 540.0).rem_euclid(360.0) - 180.0;
        delta / days
    };
    match (before, after) {
        (Some(b), Some(a)) => Some(span(b, a, 2.0 * step)),
        (Some(b), None) => Some(span(b, center, step)),
        (None, Some(a)) => Some(span(center, a, step)),
        (None, None) => None,
    }
}

/// Heliocentric VSOP87D spherical coordinates (ecliptic of date):
/// longitude and latitude in **degrees**, radius in AU.
fn vsop_heliocentric(body: Body, jd_tt: f64) -> (f64, f64, f64) {
    use vsop87::vsop87d;
    let coords = match body {
        Body::Mercury => vsop87d::mercury(jd_tt),
        Body::Venus => vsop87d::venus(jd_tt),
        Body::Mars => vsop87d::mars(jd_tt),
        Body::Jupiter => vsop87d::jupiter(jd_tt),
        Body::Saturn => vsop87d::saturn(jd_tt),
        Body::Uranus => vsop87d::uranus(jd_tt),
        Body::Neptune => vsop87d::neptune(jd_tt),
        // Earth is used internally for the geocentric conversion.
        _ => vsop87d::earth(jd_tt),
    };
    (
        normalize_deg(coords.longitude().to_degrees()),
        coords.latitude().to_degrees(),
        coords.distance(),
    )
}

/// Rectangular ecliptic-of-date coordinates of a heliocentric spherical
/// position.
fn to_rect(lon_deg: f64, lat_deg: f64, radius: f64) -> (f64, f64, f64) {
    let (lon, lat) = (lon_deg.to_radians(), lat_deg.to_radians());
    (
        radius * lat.cos() * lon.cos(),
        radius * lat.cos() * lon.sin(),
        radius * lat.sin(),
    )
}

/// Geometric geocentric position with light-time iteration (Meeus ch. 33):
/// the planet is evaluated at `t − τ`, the Earth at `t`.
fn geometric_geocentric(body: Body, jd_tt: f64) -> (f64, f64, f64) {
    let (earth_lon, earth_lat, earth_r) = vsop_heliocentric(Body::Sun, jd_tt);
    let (ex, ey, ez) = to_rect(earth_lon, earth_lat, earth_r);

    let mut tau = 0.0;
    let mut geo = (0.0, 0.0, 0.0);
    for _ in 0..3 {
        let (lon, lat, radius) = vsop_heliocentric(body, jd_tt - tau);
        let (px, py, pz) = to_rect(lon, lat, radius);
        geo = (px - ex, py - ey, pz - ez);
        let dist = (geo.0 * geo.0 + geo.1 * geo.1 + geo.2 * geo.2).sqrt();
        tau = LIGHT_TIME_PER_AU * dist;
    }
    geo
}

/// FK5 frame correction (Meeus eq. 32.3), returns (Δλ, Δβ) in degrees.
fn fk5_correction(lon_deg: f64, lat_deg: f64, jd_tt: f64) -> (f64, f64) {
    let big_t = (jd_tt - J2000) / 36_525.0;
    let lon_prime = (lon_deg - 1.397 * big_t - 0.000_31 * big_t * big_t).to_radians();
    let dlon = (-0.090_33
        + 0.039_16 * (lon_prime.cos() + lon_prime.sin()) * lat_deg.to_radians().tan())
        / 3600.0;
    let dlat = 0.039_16 * (lon_prime.cos() - lon_prime.sin()) / 3600.0;
    (dlon, dlat)
}

/// Annual aberration in ecliptic coordinates (Meeus eq. 23.2, low
/// precision form), returns (Δλ, Δβ) in degrees.
pub(crate) fn aberration(lon_deg: f64, lat_deg: f64, jd_tt: f64) -> (f64, f64) {
    let big_t = (jd_tt - J2000) / 36_525.0;
    // Sun true longitude (low precision suffices here, Meeus ch. 25).
    let sun_lon = sun_true_longitude_low(jd_tt).to_radians();
    let ecc = 0.016_708_634 - 0.000_042_037 * big_t - 0.000_000_126_7 * big_t * big_t;
    let perihelion = (102.937_35 + 1.719_46 * big_t + 0.000_46 * big_t * big_t).to_radians();

    let lon = lon_deg.to_radians();
    let lat = lat_deg.to_radians();
    let kappa = ABERRATION_KAPPA / 3600.0;

    let dlon =
        (-kappa * (sun_lon - lon).cos() + ecc * kappa * (perihelion - lon).cos()) / lat.cos();
    let dlat = -kappa * lat.sin() * ((sun_lon - lon).sin() - ecc * (perihelion - lon).sin());
    (dlon, dlat)
}

/// Low-precision Sun true longitude in degrees (Meeus ch. 25, eq. 25.2
/// class — used only inside the aberration term where 0.01° suffices).
fn sun_true_longitude_low(jd_tt: f64) -> f64 {
    let big_t = (jd_tt - J2000) / 36_525.0;
    let mean_lon = 280.466_46 + 36_000.769_83 * big_t + 0.000_303_2 * big_t * big_t;
    let mean_anom = (357.529_11 + 35_999.050_29 * big_t - 0.000_153_7 * big_t * big_t).to_radians();
    let center = (1.914_602 - 0.004_817 * big_t - 0.000_014 * big_t * big_t) * mean_anom.sin()
        + (0.019_993 - 0.000_101 * big_t) * (2.0 * mean_anom).sin()
        + 0.000_289 * (3.0 * mean_anom).sin();
    normalize_deg(mean_lon + center)
}

/// Apparent geocentric position of a VSOP planet (Meeus ch. 33).
fn planet_apparent(body: Body, jd_tt: f64) -> EclipticPos {
    let (gx, gy, gz) = geometric_geocentric(body, jd_tt);
    let dist = (gx * gx + gy * gy + gz * gz).sqrt();
    let mut lon = gy.atan2(gx).to_degrees();
    let mut lat = (gz / (gx * gx + gy * gy).sqrt()).atan().to_degrees();

    let (dlon_fk5, dlat_fk5) = fk5_correction(lon, lat, jd_tt);
    lon += dlon_fk5;
    lat += dlat_fk5;

    let (dlon_ab, dlat_ab) = aberration(lon, lat, jd_tt);
    lon += dlon_ab;
    lat += dlat_ab;

    let (dpsi, _) = nutation::nutation(jd_tt);
    lon += dpsi;

    EclipticPos {
        lon_deg: normalize_deg(lon),
        lat_deg: lat,
        dist_au: dist,
    }
}

/// Apparent geocentric Sun (Meeus ch. 25, VSOP-based high accuracy).
fn sun_apparent(jd_tt: f64) -> EclipticPos {
    let (earth_lon, earth_lat, earth_r) = vsop_heliocentric(Body::Sun, jd_tt);
    let mut lon = normalize_deg(earth_lon + 180.0);
    let mut lat = -earth_lat;

    let (dlon_fk5, dlat_fk5) = fk5_correction(lon, lat, jd_tt);
    lon += dlon_fk5;
    lat += dlat_fk5;

    // Aberration for the Sun: −20″.4898 / R (Meeus eq. 25.10).
    lon -= ABERRATION_KAPPA / 3600.0 / earth_r;

    let (dpsi, _) = nutation::nutation(jd_tt);
    lon += dpsi;

    EclipticPos {
        lon_deg: normalize_deg(lon),
        lat_deg: lat,
        dist_au: earth_r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bodies::Body;

    /// The Sun's instantaneous speed stays in its physical envelope
    /// (0.953°/day at aphelion, 1.019°/day at perihelion) and its
    /// year-average is the mean motion 0.9856°/day.
    #[test]
    fn sun_daily_motion() {
        let mut sum = 0.0;
        for day in 0..366 {
            let speed = longitude_speed(Body::Sun, J2000 + f64::from(day)).unwrap();
            assert!(
                (0.94..1.03).contains(&speed),
                "sun speed {speed} on day {day}"
            );
            sum += speed;
        }
        let mean = sum / 366.0;
        assert!((mean - 0.9856).abs() < 0.002, "mean sun speed {mean}");
    }

    /// Venus geocentric distance stays within its physical envelope
    /// (0.25–1.75 AU).
    #[test]
    fn venus_distance_envelope() {
        for step in 0..40 {
            let jd = J2000 + f64::from(step) * 100.0;
            let pos = apparent(Body::Venus, jd).unwrap();
            assert!(pos.dist_au > 0.25 && pos.dist_au < 1.75, "{}", pos.dist_au);
        }
    }

    /// Mars exhibits retrograde motion within any ~2.2-year window.
    #[test]
    fn mars_goes_retrograde() {
        let mut saw_retro = false;
        let mut saw_direct = false;
        for day in 0..800 {
            let jd = J2000 + f64::from(day);
            let speed = longitude_speed(Body::Mars, jd).unwrap();
            if speed < 0.0 {
                saw_retro = true;
            } else {
                saw_direct = true;
            }
        }
        assert!(saw_retro && saw_direct);
    }

    /// A speed taken from a centre already evaluated is the speed.
    #[test]
    fn a_speed_from_a_held_centre_is_the_same_speed() {
        for body in Body::ALL {
            for jd in [J2000, J2000 + 1234.567, 2_460_000.25] {
                let Some(center) = apparent(body, jd) else {
                    continue;
                };
                assert_eq!(
                    longitude_speed_from(body, jd, center).map(f64::to_bits),
                    longitude_speed(body, jd).map(f64::to_bits),
                    "{body:?} at {jd}"
                );
            }
        }
    }
}
