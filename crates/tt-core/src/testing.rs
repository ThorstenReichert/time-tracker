//! Test support (feature `test-support`): simulated time zone and clock helpers. Step 3 adds
//! the random command generator. This is the only module where the purity lints are relaxed,
//! because tests need tzdb lookups to build a `Ctx`.
#![allow(clippy::disallowed_methods)]

use jiff::{civil, tz::TimeZone};

use crate::{Ctx, Instant};

/// The zone the model tests run in: it has both DST changes (OQ-6).
pub fn berlin() -> TimeZone {
    TimeZone::get("Europe/Berlin").expect("bundled tzdb has Europe/Berlin")
}

/// The instant of a local wall-clock time in `tz`, e.g. `at(&berlin(), "2026-10-17T09:00")`.
/// Panics on an ambiguous or nonexistent time; tests pick unambiguous ones.
pub fn at(tz: &TimeZone, local: &str) -> Instant {
    let dt: civil::DateTime = local.parse().expect("local datetime like 2026-10-17T09:00");
    let zoned = tz
        .to_ambiguous_zoned(dt)
        .unambiguous()
        .expect("unambiguous local time");
    Instant::from(zoned.timestamp())
}

/// A `Ctx` at a local wall-clock time in Europe/Berlin.
pub fn ctx_berlin(local: &str) -> Ctx {
    let tz = berlin();
    Ctx::new(at(&tz, local), tz)
}
