//! Bornsky's standalone calculation foundation.
//!
//! Pure functions with explicit inputs: no network, database, account, catalog
//! or system clock. Callers resolve civil time to UT1 and provide their chosen
//! TT - UT1 correction. See [`chart::natal_chart`] and [`time::tt_from_ut1`].

pub mod angle;
pub mod aspects;
pub mod bodies;
pub mod chart;
pub mod coords;
pub mod ephemeris;
pub mod houses;
pub mod nutation;
pub mod time;

/// Output version for this standalone calculation library.
///
/// This has its own version sequence; it is not a hosted-service cache version.
pub const ENGINE_VERSION: u32 = 1;
