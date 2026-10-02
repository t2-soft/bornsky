//! Angle helpers shared across the engine.

/// Normalizes an angle in degrees to the half-open range `[0, 360)`.
///
/// # Examples
///
/// ```
/// assert!((astro_engine::angle::normalize_deg(-30.0) - 330.0).abs() < 1e-12);
/// assert!((astro_engine::angle::normalize_deg(720.5) - 0.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn normalize_deg(deg: f64) -> f64 {
    let r = deg.rem_euclid(360.0);
    // rem_euclid can return 360.0 for tiny negative inputs due to
    // floating-point rounding; fold that back to 0.
    if r >= 360.0 {
        r - 360.0
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_deg;
    use proptest::prelude::*;

    #[test]
    fn known_values() {
        assert!((normalize_deg(0.0) - 0.0).abs() < 1e-12);
        assert!((normalize_deg(360.0) - 0.0).abs() < 1e-12);
        assert!((normalize_deg(-30.0) - 330.0).abs() < 1e-12);
        assert!((normalize_deg(725.0) - 5.0).abs() < 1e-12);
    }

    proptest! {
        #[test]
        fn always_in_range(deg in -1.0e6f64..1.0e6) {
            let r = normalize_deg(deg);
            prop_assert!((0.0..360.0).contains(&r), "normalize_deg({deg}) = {r}");
        }
    }
}
