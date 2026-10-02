//! Deterministic regression fixtures for sidereal time and horizon projection.

use astro_engine::coords::equatorial_to_horizontal;
use astro_engine::time::gmst_deg;

const GOLDENS: &str = include_str!("data/sidereal_goldens.json");

#[derive(serde::Deserialize)]
struct Case {
    jd_ut: f64,
    ra_deg: f64,
    dec_deg: f64,
    lat_deg: f64,
    lon_deg: f64,
    gmst_deg: f64,
    az_deg: f64,
    alt_deg: f64,
}

#[test]
fn sidereal_fixture_matches_the_engine() {
    let cases: Vec<Case> = serde_json::from_str(GOLDENS).expect("fixture parses");
    assert!(cases.len() >= 5, "the fixture must stay meaningful");
    for case in cases {
        let gmst = gmst_deg(case.jd_ut);
        assert!(
            (gmst - case.gmst_deg).abs() < 1e-9,
            "gmst at {}: engine {gmst}, fixture {}",
            case.jd_ut,
            case.gmst_deg
        );
        let lst = (gmst + case.lon_deg).rem_euclid(360.0);
        let (az, alt) = equatorial_to_horizontal(case.ra_deg, case.dec_deg, case.lat_deg, lst);
        assert!(
            (az - case.az_deg).abs() < 1e-9,
            "az at {}: engine {az}, fixture {}",
            case.jd_ut,
            case.az_deg
        );
        assert!(
            (alt - case.alt_deg).abs() < 1e-9,
            "alt at {}: engine {alt}, fixture {}",
            case.jd_ut,
            case.alt_deg
        );
    }
}
