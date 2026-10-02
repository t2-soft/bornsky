//! Julian Day, sidereal time and explicit time-scale conversion.
//! No timezone lookup, leap-second table or Delta-T estimation is bundled.

/// The Julian Day of the J2000.0 epoch (2000 January 1.5 TT).
pub const J2000: f64 = 2_451_545.0;

/// Converts a UT1 Julian Day to TT using an explicit TT - UT1 offset in seconds.
///
/// This is arithmetic, not an estimated Delta-T model. The caller must validate
/// its inputs and obtain the offset appropriate to the date. The checked chart
/// entry point performs finite/range validation before using this helper.
#[must_use]
pub fn tt_from_ut1(jd_ut1: f64, delta_t_seconds: f64) -> f64 {
    jd_ut1 + delta_t_seconds / 86_400.0
}

/// The engine's supported Julian Day window: calendar years 1550–2650
/// with a one-year margin. Besides
/// bounding theory validity, staying in this window guarantees float
/// increments of fractions of a day remain exact enough for scanning
/// loops (`jd + 1.0 > jd` by a wide margin).
pub const JD_SUPPORTED: core::ops::RangeInclusive<f64> = 2_286_819.5..=2_690_014.5;

/// Whether a Julian Day is finite and inside [`JD_SUPPORTED`].
#[must_use]
pub fn jd_supported(jd: f64) -> bool {
    jd.is_finite() && JD_SUPPORTED.contains(&jd)
}

/// Converts a proleptic **Gregorian** calendar date to a Julian Day.
///
/// `day` may carry a fractional part (0.5 = noon UT of the previous JD
/// boundary convention). Month is 1-based (1 = January).
///
/// Algorithm: Meeus, *Astronomical Algorithms* (2nd ed.), ch. 7. Valid
/// for all Gregorian-calendar dates (the engine's supported span is
/// 1550–2650, well inside).
///
/// # Examples
///
/// ```
/// let jd = astro_engine::time::julian_day(2000, 1, 1.5);
/// assert!((jd - astro_engine::time::J2000).abs() < 1e-9);
/// ```
#[must_use]
pub fn julian_day(year: i32, month: u8, day: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, i32::from(month) + 12)
    } else {
        (year, i32::from(month))
    };
    let a = y.div_euclid(100);
    let b = 2 - a + a.div_euclid(4);
    // `y + 4716 > 0` for every supported year, so floor == truncation here.
    let year_days = (365.25 * f64::from(y + 4716)).floor();
    let month_days = (30.6001 * f64::from(m + 1)).floor();
    year_days + month_days + day + f64::from(b) - 1524.5
}

/// Converts a Julian Day back to a proleptic **Gregorian** calendar date
/// `(year, month, day)`, the exact inverse of [`julian_day`].
///
/// `day` carries the fractional part. Algorithm: Meeus, *Astronomical
/// Algorithms* (2nd ed.), ch. 7, with the Gregorian correction applied
/// unconditionally so the pair round-trips over the engine's whole
/// supported span (our [`julian_day`] is proleptic Gregorian, never
/// Julian-calendar).
///
/// # Examples
///
/// ```
/// let (y, m, d) = astro_engine::time::calendar_from_jd(2_436_116.31);
/// assert_eq!((y, m), (1957, 10));
/// assert!((d - 4.81).abs() < 1e-6);
/// ```
#[must_use]
pub fn calendar_from_jd(jd: f64) -> (i32, u8, f64) {
    // Descriptive names for Meeus's Z, F, α, A, B, C, D, E (in order).
    let shifted = jd + 0.5;
    let integral = shifted.floor();
    let frac = shifted - integral;

    // Gregorian correction, applied unconditionally (see doc comment).
    let centuries = ((integral - 1_867_216.25) / 36_524.25).floor();
    let corrected = integral + 1.0 + centuries - (centuries / 4.0).floor();

    let days_epoch = corrected + 1524.0;
    let year_count = ((days_epoch - 122.1) / 365.25).floor();
    let days_in_years = (365.25 * year_count).floor();
    let month_index = ((days_epoch - days_in_years) / 30.6001).floor();

    let day = days_epoch - days_in_years - (30.6001 * month_index).floor() + frac;
    let month_f = if month_index < 14.0 {
        month_index - 1.0
    } else {
        month_index - 13.0
    };
    let month = month_f as i32; // always in 1..=12 by construction
    let year = if month > 2 {
        year_count - 4716.0
    } else {
        year_count - 4715.0
    };

    (
        year as i32,
        u8::try_from(month.clamp(1, 12)).unwrap_or(1),
        day,
    )
}

/// Greenwich Mean Sidereal Time at `jd` (UT1), in degrees `[0, 360)`.
///
/// Algorithm: Meeus, *Astronomical Algorithms* (2nd ed.), eq. 12.4 —
/// valid for any instant, not just 0h UT. The T² and T³ terms keep the
/// expression accurate across the engine's supported span; the dominant
/// term advances ~360.9856°/day (one sidereal rotation ≈ 23h56m04s).
///
/// This seeds the sidereal-time layer the house-system math needs
/// (ASC/MC computation takes local sidereal time = GMST + east
/// longitude). Apparent sidereal time (GMST + equation of the equinoxes)
/// lands with the nutation module in Phase 1.
///
/// # Examples
///
/// ```
/// // Meeus ex. 12.b: 1987 April 10, 19h21m00s UT.
/// let jd = astro_engine::time::julian_day(1987, 4, 10.0) + (19.0 + 21.0 / 60.0) / 24.0;
/// let gmst = astro_engine::time::gmst_deg(jd);
/// assert!((gmst - 128.737_873).abs() < 1e-4);
/// ```
#[must_use]
pub fn gmst_deg(jd: f64) -> f64 {
    let t = (jd - J2000) / 36_525.0;
    let theta = 280.460_618_37 + 360.985_647_366_29 * (jd - J2000) + 0.000_387_933 * t * t
        - t * t * t / 38_710_000.0;
    crate::angle::normalize_deg(theta)
}

#[cfg(test)]
mod tests {
    use super::{calendar_from_jd, gmst_deg, julian_day, J2000};
    use proptest::prelude::*;

    /// Golden values from Meeus, *Astronomical Algorithms* (2nd ed.),
    /// ch. 7 (example 7.a and the chapter's test table).
    #[test]
    fn meeus_golden_values() {
        let cases = [
            ((1957, 10, 4.81), 2_436_116.31), // Sputnik 1 launch (ex. 7.a)
            ((2000, 1, 1.5), 2_451_545.0),    // J2000.0
            ((1999, 1, 1.0), 2_451_179.5),
            ((1987, 1, 27.0), 2_446_822.5),
            ((1987, 6, 19.5), 2_446_966.0),
            ((1988, 1, 27.0), 2_447_187.5),
            ((1988, 6, 19.5), 2_447_332.0),
            ((1900, 1, 1.0), 2_415_020.5),
            ((1600, 1, 1.0), 2_305_447.5),
            ((1600, 12, 31.0), 2_305_812.5),
        ];
        for ((y, m, d), expected) in cases {
            let jd = julian_day(y, m, d);
            assert!(
                (jd - expected).abs() < 1e-6,
                "julian_day({y}, {m}, {d}) = {jd}, expected {expected}"
            );
        }
    }

    #[test]
    fn j2000_constant_matches() {
        assert!((julian_day(2000, 1, 1.5) - J2000).abs() < 1e-9);
    }

    /// The same Meeus table, driven through the inverse.
    #[test]
    fn inverse_golden_values() {
        let cases = [
            (2_436_116.31, (1957, 10, 4.81)),
            (2_451_545.0, (2000, 1, 1.5)),
            (2_446_822.5, (1987, 1, 27.0)),
            (2_415_020.5, (1900, 1, 1.0)),
        ];
        for (jd, (ey, em, ed)) in cases {
            let (y, m, d) = calendar_from_jd(jd);
            assert_eq!((y, m), (ey, em), "calendar_from_jd({jd})");
            assert!(
                (d - ed).abs() < 1e-6,
                "calendar_from_jd({jd}) day = {d}, expected {ed}"
            );
        }
    }

    /// Meeus ch. 12 worked examples.
    #[test]
    fn gmst_golden_values() {
        // Ex. 12.a: 1987 April 10.0 UT (JD 2446895.5) →
        // θ0 = 13h10m46.3668s = 197.693195°.
        let gmst_midnight = gmst_deg(2_446_895.5);
        assert!(
            (gmst_midnight - 197.693_195).abs() < 1e-5,
            "gmst(2446895.5) = {gmst_midnight}, expected 197.693195"
        );
        // Ex. 12.b: same date at 19h21m00s UT → 128.737873°.
        let jd = 2_446_895.5 + (19.0 + 21.0 / 60.0) / 24.0;
        let gmst_evening = gmst_deg(jd);
        assert!(
            (gmst_evening - 128.737_873).abs() < 1e-4,
            "gmst({jd}) = {gmst_evening}, expected 128.737873"
        );
    }

    proptest! {
        /// GMST is always a normalized angle. Offsets span the engine's
        /// documented supported range, 1550–2650 (review feedback: match
        /// the year range the other proptests in this file cover).
        #[test]
        fn gmst_in_range(offset in -165_000.0f64..238_000.0) {
            let g = gmst_deg(J2000 + offset);
            prop_assert!((0.0..360.0).contains(&g));
        }

        /// The sky advances ~0.9856° per solar day relative to the Sun:
        /// GMST one day later is ahead by 360.9856…° mod 360. Same
        /// 1550–2650 offset span as `gmst_in_range`.
        #[test]
        fn gmst_daily_advance(offset in -165_000.0f64..238_000.0) {
            let g0 = gmst_deg(J2000 + offset);
            let g1 = gmst_deg(J2000 + offset + 1.0);
            let advance = (g1 - g0).rem_euclid(360.0);
            prop_assert!((advance - 0.985_647).abs() < 1e-3,
                "daily advance was {advance}");
        }

        /// Calendar → JD → calendar is the identity across the whole
        /// supported span, fractional day included.
        #[test]
        fn calendar_roundtrips_through_jd(
            year in 1600i32..2600,
            month in 1u8..=12,
            day in 1u8..=28,
            fraction in 0.0f64..1.0,
        ) {
            let d = f64::from(day) + fraction;
            let (y2, m2, d2) = calendar_from_jd(julian_day(year, month, d));
            prop_assert_eq!((year, month), (y2, m2));
            prop_assert!((d - d2).abs() < 1e-8, "day {d} came back {d2}");
        }

        /// JD strictly increases with the date: one calendar day later
        /// is exactly +1.0 JD.
        #[test]
        fn one_day_later_is_plus_one(year in 1600i32..2600, month in 1u8..=12, day in 1u8..=27) {
            let d = f64::from(day);
            let jd0 = julian_day(year, month, d);
            let jd1 = julian_day(year, month, d + 1.0);
            prop_assert!((jd1 - jd0 - 1.0).abs() < 1e-9);
        }

        /// Round-trip: calendar → JD → calendar is the identity over the
        /// engine's supported span (fractional days included).
        #[test]
        fn jd_roundtrips(year in 1600i32..2600, month in 1u8..=12, day in 1u8..=28, frac in 0u32..1000) {
            let d = f64::from(day) + f64::from(frac) / 1000.0;
            let (y, m, d2) = calendar_from_jd(julian_day(year, month, d));
            prop_assert_eq!((y, m), (year, month));
            prop_assert!((d2 - d).abs() < 1e-6, "day {} came back as {}", d, d2);
        }
    }
}
