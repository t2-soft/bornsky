//! Aspect detection between chart points.
//! Default orbs: major aspects 8 degrees, sextile 6, minor aspects 4.

use crate::angle::normalize_deg;

/// The aspect types the engine detects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectKind {
    Conjunction,
    Sextile,
    Square,
    Trine,
    Opposition,
    SemiSextile,
    SemiSquare,
    Sesquiquadrate,
    Quincunx,
}

impl AspectKind {
    /// All detected aspects with their exact angle and default orb.
    pub const ALL: [(AspectKind, f64, f64); 9] = [
        (AspectKind::Conjunction, 0.0, 8.0),
        (AspectKind::Sextile, 60.0, 6.0),
        (AspectKind::Square, 90.0, 8.0),
        (AspectKind::Trine, 120.0, 8.0),
        (AspectKind::Opposition, 180.0, 8.0),
        (AspectKind::SemiSextile, 30.0, 4.0),
        (AspectKind::SemiSquare, 45.0, 4.0),
        (AspectKind::Sesquiquadrate, 135.0, 4.0),
        (AspectKind::Quincunx, 150.0, 4.0),
    ];

    /// Exact angle in degrees.
    #[must_use]
    pub fn angle(self) -> f64 {
        Self::ALL
            .iter()
            .find(|(kind, _, _)| *kind == self)
            .map_or(0.0, |&(_, angle, _)| angle)
    }

    /// Whether this is one of the five Ptolemaic (major) aspects.
    #[must_use]
    pub fn is_major(self) -> bool {
        matches!(
            self,
            AspectKind::Conjunction
                | AspectKind::Sextile
                | AspectKind::Square
                | AspectKind::Trine
                | AspectKind::Opposition
        )
    }
}

/// A detected aspect between two longitudes.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Aspect {
    pub kind: AspectKind,
    /// Deviation from exactness in degrees (always ≥ 0).
    pub orb: f64,
    /// True when the faster point is moving toward exactness.
    pub applying: Option<bool>,
}

/// The unsigned separation between two longitudes, `[0, 180]`.
#[must_use]
pub fn separation(lon_a: f64, lon_b: f64) -> f64 {
    let diff = normalize_deg(lon_a - lon_b);
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Detects the aspect (if any) between two longitudes. When both
/// longitude speeds are supplied, `applying` reports whether the pair is
/// moving toward exactness.
#[must_use]
pub fn detect(
    lon_a: f64,
    lon_b: f64,
    speed_a: Option<f64>,
    speed_b: Option<f64>,
) -> Option<Aspect> {
    let sep = separation(lon_a, lon_b);

    let mut best: Option<(AspectKind, f64, f64)> = None;
    for &(kind, angle, orb_limit) in &AspectKind::ALL {
        let orb = (sep - angle).abs();
        if orb <= orb_limit && best.is_none_or(|(_, _, b)| orb < b) {
            best = Some((kind, angle, orb));
        }
    }
    let (kind, angle, orb) = best?;

    let applying = match (speed_a, speed_b) {
        (Some(va), Some(vb)) => {
            // Separation derivative: does |sep − angle| shrink?
            let probe_days = 0.01;
            let sep_next = separation(lon_a + va * probe_days, lon_b + vb * probe_days);
            Some((sep_next - angle).abs() < (sep - angle).abs())
        }
        _ => None,
    };

    Some(Aspect {
        kind,
        orb,
        applying,
    })
}

#[cfg(test)]
mod tests {
    use super::{detect, separation, AspectKind};
    use proptest::prelude::*;

    #[test]
    fn exact_aspects() {
        assert_eq!(
            detect(10.0, 10.0, None, None).unwrap().kind,
            AspectKind::Conjunction
        );
        assert_eq!(
            detect(10.0, 190.0, None, None).unwrap().kind,
            AspectKind::Opposition
        );
        assert_eq!(
            detect(10.0, 130.0, None, None).unwrap().kind,
            AspectKind::Trine
        );
        assert_eq!(
            detect(355.0, 25.0, None, None).unwrap().kind,
            AspectKind::SemiSextile
        );
    }

    #[test]
    fn orb_limits_respected() {
        // 68.5° from exact sextile by 8.5° — outside every orb.
        assert!(detect(0.0, 68.5, None, None).is_none());
        // 66° = sextile with 6.0° orb — exactly on the limit.
        assert!(detect(0.0, 66.0, None, None).is_some());
    }

    #[test]
    fn applying_flag() {
        // A at 0° gaining on B at 122°: the 122° separation shrinks
        // TOWARD the exact 120° trine — applying.
        let aspect = detect(0.0, 122.0, Some(1.0), Some(0.05)).unwrap();
        assert_eq!(aspect.kind, AspectKind::Trine);
        assert_eq!(aspect.applying, Some(true));
        // A at 0° gaining on B at 118°: separation shrinks AWAY from
        // 120° — separating.
        let aspect = detect(0.0, 118.0, Some(1.0), Some(0.05)).unwrap();
        assert_eq!(aspect.applying, Some(false));
    }

    proptest! {
        /// Separation is symmetric and within [0, 180].
        #[test]
        fn separation_properties(a in 0.0f64..360.0, b in 0.0f64..360.0) {
            let sep = separation(a, b);
            prop_assert!((0.0..=180.0).contains(&sep));
            prop_assert!((sep - separation(b, a)).abs() < 1e-9);
        }

        /// Detection is symmetric in its arguments.
        #[test]
        fn detect_symmetric(a in 0.0f64..360.0, b in 0.0f64..360.0) {
            let ab = detect(a, b, None, None).map(|x| (x.kind, x.orb));
            let ba = detect(b, a, None, None).map(|x| (x.kind, x.orb));
            match (ab, ba) {
                (Some((ka, oa)), Some((kb, ob))) => {
                    prop_assert_eq!(ka, kb);
                    prop_assert!((oa - ob).abs() < 1e-9);
                }
                (None, None) => {}
                _ => prop_assert!(false, "asymmetric detection"),
            }
        }
    }
}
