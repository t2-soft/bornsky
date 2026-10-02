use astro_engine::chart::{natal_chart, BirthData};
use astro_engine::houses::HouseSystem;
use astro_engine::time::julian_day;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Synthetic example. UT1 is explicit; this is not a local civil timestamp.
    // 64 seconds is illustrative, not a built-in time model or a present-day default.
    let birth = BirthData {
        jd_ut: julian_day(2000, 1, 1.5),
        lat_deg: 51.5,
        lon_deg: -0.12,
    };
    let chart = natal_chart(birth, HouseSystem::Placidus, 64.0)?;
    println!("{}", serde_json::to_string_pretty(&chart)?);
    Ok(())
}
