use astro_engine::chart::{natal_chart, BirthData, ChartError};
use astro_engine::houses::HouseSystem;
use astro_engine::time::{tt_from_ut1, J2000, JD_SUPPORTED};

fn sample() -> BirthData {
    BirthData {
        jd_ut: J2000,
        lat_deg: 51.5,
        lon_deg: -0.12,
    }
}

#[test]
fn rejects_invalid_inputs_before_computation() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            natal_chart(
                BirthData {
                    jd_ut: value,
                    ..sample()
                },
                HouseSystem::Equal,
                64.0
            ),
            Err(ChartError::UnsupportedJulianDay)
        );
        assert_eq!(
            natal_chart(
                BirthData {
                    lat_deg: value,
                    ..sample()
                },
                HouseSystem::Equal,
                64.0
            ),
            Err(ChartError::InvalidLatitude)
        );
        assert_eq!(
            natal_chart(
                BirthData {
                    lon_deg: value,
                    ..sample()
                },
                HouseSystem::Equal,
                64.0
            ),
            Err(ChartError::InvalidLongitude)
        );
        assert_eq!(
            natal_chart(sample(), HouseSystem::Equal, value),
            Err(ChartError::InvalidDeltaT)
        );
    }
    for lat in [-90.0, 90.0, 91.0] {
        assert_eq!(
            natal_chart(
                BirthData {
                    lat_deg: lat,
                    ..sample()
                },
                HouseSystem::Placidus,
                64.0
            ),
            Err(ChartError::InvalidLatitude)
        );
    }
    assert_eq!(
        natal_chart(
            BirthData {
                lon_deg: 181.0,
                ..sample()
            },
            HouseSystem::Equal,
            64.0
        ),
        Err(ChartError::InvalidLongitude)
    );
    assert_eq!(
        natal_chart(sample(), HouseSystem::Equal, 86_401.0),
        Err(ChartError::InvalidDeltaT)
    );
    assert_eq!(
        natal_chart(
            BirthData {
                jd_ut: *JD_SUPPORTED.end(),
                ..sample()
            },
            HouseSystem::Equal,
            64.0
        ),
        Err(ChartError::UnsupportedJulianDay)
    );
}

#[test]
#[allow(clippy::float_cmp)] // Exact supplied input and identical arithmetic are the contract.
fn explicit_time_correction_controls_planet_epoch() {
    let a = natal_chart(sample(), HouseSystem::Equal, 0.0).unwrap();
    let b = natal_chart(sample(), HouseSystem::Equal, 3_600.0).unwrap();
    assert_eq!(a.birth, b.birth);
    assert_eq!(b.delta_t_seconds, 3_600.0);
    assert_eq!(b.jd_tt, tt_from_ut1(J2000, 3_600.0));
    let moon = |chart: &astro_engine::chart::Chart| {
        chart
            .bodies
            .iter()
            .find(|p| p.body == astro_engine::bodies::Body::Moon)
            .unwrap()
            .lon_deg
    };
    assert!((moon(&a) - moon(&b)).abs() > 0.4);
}

#[test]
#[allow(clippy::float_cmp)] // The integer-valued input offset must survive serialization exactly.
fn polar_fallback_and_serialization_are_explicit() {
    let result = natal_chart(
        BirthData {
            lat_deg: 78.0,
            ..sample()
        },
        HouseSystem::Placidus,
        64.0,
    )
    .unwrap();
    assert!(result.houses.fell_back_to_porphyry);
    let json = serde_json::to_string(&result).unwrap();
    let decoded: astro_engine::chart::Chart = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.delta_t_seconds, result.delta_t_seconds);
    assert_eq!(decoded.bodies.len(), result.bodies.len());
    assert_eq!(decoded.engine_version, astro_engine::ENGINE_VERSION);
}
