//! The celestial bodies and points the engine computes.

/// A body or computed point in a chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    /// Valid ~1885–2099 only (Goffin 1986 series; see `ephemeris::pluto`).
    Pluto,
    /// Mean ascending lunar node (the "North Node" / Rahu).
    MeanNode,
    /// Mean lunar apogee ("Black Moon Lilith"). Provisional formula; see
    /// `ephemeris::points`.
    MeanApogee,
}

impl Body {
    /// All bodies in traditional chart order.
    pub const ALL: [Body; 12] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
        Body::MeanNode,
        Body::MeanApogee,
    ];

    /// Stable machine name (used in serialized output and cache keys).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Body::Sun => "sun",
            Body::Moon => "moon",
            Body::Mercury => "mercury",
            Body::Venus => "venus",
            Body::Mars => "mars",
            Body::Jupiter => "jupiter",
            Body::Saturn => "saturn",
            Body::Uranus => "uranus",
            Body::Neptune => "neptune",
            Body::Pluto => "pluto",
            Body::MeanNode => "mean_node",
            Body::MeanApogee => "mean_apogee",
        }
    }
}

/// The twelve zodiac signs (tropical).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl Sign {
    const ALL: [Sign; 12] = [
        Sign::Aries,
        Sign::Taurus,
        Sign::Gemini,
        Sign::Cancer,
        Sign::Leo,
        Sign::Virgo,
        Sign::Libra,
        Sign::Scorpio,
        Sign::Sagittarius,
        Sign::Capricorn,
        Sign::Aquarius,
        Sign::Pisces,
    ];

    /// The sign containing an ecliptic longitude (degrees, any range).
    #[must_use]
    pub fn from_longitude(lon_deg: f64) -> Sign {
        let normalized = crate::angle::normalize_deg(lon_deg);
        // normalize_deg returns [0, 360), so the index is always 0..=11.
        let index = usize::try_from((normalized / 30.0) as i32).unwrap_or(0);
        Sign::ALL[index.min(11)]
    }

    /// Zodiacal position, Aries = 0. Whole-sign counting needs it.
    #[must_use]
    pub fn index(self) -> usize {
        Sign::ALL
            .iter()
            .position(|sign| *sign == self)
            .unwrap_or_default()
    }

    /// The sign at a zodiacal position, counted round the circle, so
    /// `at_index(12)` is Aries again.
    #[must_use]
    pub fn at_index(index: usize) -> Sign {
        Sign::ALL[index % 12]
    }

    /// Degrees into the sign, `[0, 30)`.
    #[must_use]
    pub fn degree_in_sign(lon_deg: f64) -> f64 {
        crate::angle::normalize_deg(lon_deg) % 30.0
    }

    /// Stable machine name, mirroring [`Body::key`] — serialized output
    /// must never ride `Debug` formatting.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Sign::Aries => "aries",
            Sign::Taurus => "taurus",
            Sign::Gemini => "gemini",
            Sign::Cancer => "cancer",
            Sign::Leo => "leo",
            Sign::Virgo => "virgo",
            Sign::Libra => "libra",
            Sign::Scorpio => "scorpio",
            Sign::Sagittarius => "sagittarius",
            Sign::Capricorn => "capricorn",
            Sign::Aquarius => "aquarius",
            Sign::Pisces => "pisces",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sign;

    #[test]
    fn sign_boundaries() {
        assert_eq!(Sign::from_longitude(0.0), Sign::Aries);
        assert_eq!(Sign::from_longitude(29.999), Sign::Aries);
        assert_eq!(Sign::from_longitude(30.0), Sign::Taurus);
        assert_eq!(Sign::from_longitude(359.999), Sign::Pisces);
        assert_eq!(Sign::from_longitude(360.0), Sign::Aries);
        assert_eq!(Sign::from_longitude(-1.0), Sign::Pisces);
    }

    #[test]
    fn degree_in_sign() {
        assert!((Sign::degree_in_sign(45.5) - 15.5).abs() < 1e-12);
    }
}
