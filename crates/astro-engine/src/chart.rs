//! Natal chart assembly with explicit UT1 and TT - UT1 inputs.

use crate::angle::normalize_deg;
use crate::aspects::{self, Aspect};
use crate::bodies::{Body, Sign};
use crate::ephemeris;
use crate::houses::{self, HouseSystem, Houses};
use crate::nutation;
use crate::time;

/// Birth data: a UT1 instant plus geographic coordinates
/// (north / east positive).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BirthData {
    pub jd_ut: f64,
    pub lat_deg: f64,
    pub lon_deg: f64,
}

/// Invalid input to the checked natal-chart entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartError {
    UnsupportedJulianDay,
    InvalidLatitude,
    InvalidLongitude,
    InvalidDeltaT,
}

impl core::fmt::Display for ChartError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::UnsupportedJulianDay => {
                "UT1 and TT must be within the supported Julian Day range"
            }
            Self::InvalidLatitude => {
                "latitude must be finite and strictly between -90 and 90 degrees"
            }
            Self::InvalidLongitude => "longitude must be finite and between -180 and 180 degrees",
            Self::InvalidDeltaT => "TT - UT1 must be finite and no more than one day in magnitude",
        })
    }
}

impl std::error::Error for ChartError {}

fn validate_birth(birth: BirthData, delta_t_seconds: f64) -> Result<(), ChartError> {
    if !delta_t_seconds.is_finite() || delta_t_seconds.abs() > 86_400.0 {
        return Err(ChartError::InvalidDeltaT);
    }
    if !time::jd_supported(birth.jd_ut)
        || !time::jd_supported(time::tt_from_ut1(birth.jd_ut, delta_t_seconds))
    {
        return Err(ChartError::UnsupportedJulianDay);
    }
    if !birth.lat_deg.is_finite() || birth.lat_deg.abs() >= 90.0 {
        return Err(ChartError::InvalidLatitude);
    }
    if !birth.lon_deg.is_finite() || birth.lon_deg.abs() > 180.0 {
        return Err(ChartError::InvalidLongitude);
    }
    Ok(())
}

/// One placed body in a chart.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BodyPosition {
    pub body: Body,
    /// Apparent ecliptic longitude of date, degrees `[0, 360)`.
    pub lon_deg: f64,
    pub lat_deg: f64,
    pub dist_au: f64,
    /// Longitude speed, degrees/day; negative = retrograde.
    pub speed: f64,
    pub retrograde: bool,
    pub sign: Sign,
    /// House number 1–12 under the chart's house system.
    pub house: u8,
}

/// A detected aspect between two chart bodies.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChartAspect {
    pub a: Body,
    pub b: Body,
    pub aspect: Aspect,
}

/// A complete natal chart.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Chart {
    pub birth: BirthData,
    /// Caller-supplied TT - UT1 correction, retained for reproducibility.
    pub delta_t_seconds: f64,
    /// Actual TT Julian Day used for planetary positions.
    pub jd_tt: f64,
    pub bodies: Vec<BodyPosition>,
    pub houses: Houses,
    pub aspects: Vec<ChartAspect>,
    /// Bodies unavailable at this date (e.g. Pluto outside 1885–2099).
    pub unavailable: Vec<Body>,
    pub engine_version: u32,
}

/// House number (1–12) containing an ecliptic longitude, given cusps in
/// zodiacal order.
#[must_use]
pub fn house_of(lon_deg: f64, cusps: &[f64; 12]) -> u8 {
    for i in 0..12 {
        let start = cusps[i];
        let end = cusps[(i + 1) % 12];
        let span = normalize_deg(end - start);
        let offset = normalize_deg(lon_deg - start);
        if offset < span {
            // House numbers are 1-based; i is 0..12.
            return (i + 1) as u8;
        }
    }
    12 // unreachable for well-formed cusps; safe default
}

/// Computes a natal chart. Charts compute in ecliptic-of-date, apparent
/// positions; the house system falls back to Porphyry above the polar
/// circles (flagged on [`Houses`]).
///
/// `birth.jd_ut` is UT1; `delta_t_seconds` is TT - UT1, in seconds.
/// The caller supplies the correction from its chosen data/model. UTC civil
/// time first requires timezone resolution and, for precise angles, DUT1.
/// There is deliberately no default correction or implicit timezone.
///
/// # Errors
/// Returns [`ChartError`] for non-finite values, unsupported dates, invalid
/// coordinates (including exact geographic poles), or a correction exceeding
/// one day in magnitude. Low-level numerical functions assume valid inputs.
pub fn natal_chart(
    birth: BirthData,
    system: HouseSystem,
    delta_t_seconds: f64,
) -> Result<Chart, ChartError> {
    validate_birth(birth, delta_t_seconds)?;
    let jd_tt = time::tt_from_ut1(birth.jd_ut, delta_t_seconds);

    // Local apparent sidereal time = GMST + Δψ·cos ε + east longitude.
    let (dpsi, _) = nutation::nutation(jd_tt);
    let eps = nutation::true_obliquity(jd_tt);
    let gast = time::gmst_deg(birth.jd_ut) + dpsi * eps.to_radians().cos();
    let armc = normalize_deg(gast + birth.lon_deg);

    let house_frame = houses::houses(system, armc, birth.lat_deg, eps);

    let mut bodies = Vec::with_capacity(Body::ALL.len());
    let mut unavailable = Vec::new();
    for body in Body::ALL {
        let Some(pos) = ephemeris::apparent(body, jd_tt) else {
            unavailable.push(body);
            continue;
        };
        let speed = ephemeris::longitude_speed_from(body, jd_tt, pos).unwrap_or(0.0);
        bodies.push(BodyPosition {
            body,
            lon_deg: pos.lon_deg,
            lat_deg: pos.lat_deg,
            dist_au: pos.dist_au,
            speed,
            retrograde: speed < 0.0,
            sign: Sign::from_longitude(pos.lon_deg),
            house: house_of(pos.lon_deg, &house_frame.cusps),
        });
    }

    let mut chart_aspects = Vec::new();
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            // Chart assembly omits aspects between two derived lunar points.
            // Call aspects::detect directly if that pair is needed.
            let derived = |b: Body| matches!(b, Body::MeanNode | Body::MeanApogee);
            if derived(bodies[i].body) && derived(bodies[j].body) {
                continue;
            }
            if let Some(aspect) = aspects::detect(
                bodies[i].lon_deg,
                bodies[j].lon_deg,
                Some(bodies[i].speed),
                Some(bodies[j].speed),
            ) {
                chart_aspects.push(ChartAspect {
                    a: bodies[i].body,
                    b: bodies[j].body,
                    aspect,
                });
            }
        }
    }

    Ok(Chart {
        birth,
        delta_t_seconds,
        jd_tt,
        bodies,
        houses: house_frame,
        aspects: chart_aspects,
        unavailable,
        engine_version: crate::ENGINE_VERSION,
    })
}

#[cfg(test)]
mod tests {
    use super::{house_of, natal_chart, BirthData};
    use crate::bodies::Body;
    use crate::houses::HouseSystem;
    use crate::time::julian_day;

    fn sample_chart() -> super::Chart {
        // Synthetic fixture: 1990-06-15 14:30 UT1, London coordinates.
        let jd_ut = julian_day(1990, 6, 15.0) + 14.5 / 24.0;
        natal_chart(
            BirthData {
                jd_ut,
                lat_deg: 51.5074,
                lon_deg: -0.1278,
            },
            HouseSystem::Placidus,
            57.0, // Explicit illustrative correction for this synthetic fixture.
        )
        .unwrap()
    }

    #[test]
    fn full_chart_assembles() {
        let chart = sample_chart();
        assert_eq!(chart.bodies.len(), 12);
        assert!(chart.unavailable.is_empty());
        assert!(!chart.houses.fell_back_to_porphyry);
        for body in &chart.bodies {
            assert!((0.0..360.0).contains(&body.lon_deg), "{:?}", body.body);
            assert!((1..=12).contains(&body.house), "{:?}", body.body);
        }
        // Sun in mid-June is in Gemini.
        let sun = chart.bodies.iter().find(|b| b.body == Body::Sun).unwrap();
        assert_eq!(sun.sign, crate::bodies::Sign::Gemini);
        // Sun and Moon never retrograde.
        let moon = chart.bodies.iter().find(|b| b.body == Body::Moon).unwrap();
        assert!(!sun.retrograde && !moon.retrograde);
        // Moon's speed is in its physical band (11.7–15.4 °/day).
        assert!(
            (11.0..16.0).contains(&moon.speed),
            "moon speed {}",
            moon.speed
        );
    }

    #[test]
    fn pluto_gated_outside_span() {
        let chart = natal_chart(
            BirthData {
                jd_ut: julian_day(1850, 1, 1.5),
                lat_deg: 0.0,
                lon_deg: 0.0,
            },
            HouseSystem::WholeSign,
            7.0, // Only Pluto availability is being tested here.
        )
        .unwrap();
        assert_eq!(chart.unavailable, vec![Body::Pluto]);
        assert_eq!(chart.bodies.len(), 11);
    }

    #[test]
    fn house_of_wraparound() {
        let cusps = [
            350.0, 20.0, 50.0, 80.0, 110.0, 140.0, 170.0, 200.0, 230.0, 260.0, 290.0, 320.0,
        ];
        assert_eq!(house_of(355.0, &cusps), 1);
        assert_eq!(house_of(10.0, &cusps), 1);
        assert_eq!(house_of(20.0, &cusps), 2);
        assert_eq!(house_of(349.9, &cusps), 12);
    }

    #[test]
    fn aspects_present_and_within_orb() {
        let chart = sample_chart();
        assert!(!chart.aspects.is_empty());
        for entry in &chart.aspects {
            assert!(entry.aspect.orb <= 8.0);
        }
    }
}
