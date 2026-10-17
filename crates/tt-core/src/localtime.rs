//! Typed local times (R10, R11) → UTC instants, with the zone from `Ctx` (OQ-6).

use jiff::{
    civil,
    tz::{AmbiguousOffset, TimeZone},
};

use crate::{DomainError, Instant};

/// Converts a typed wall-clock time to an instant. A time in the skipped hour (DST start)
/// or the repeated hour (DST end) is rejected, not resolved silently (OQ-6, R12). Times are
/// recorded to the second, so a fractional second is rejected too.
pub fn to_instant(tz: &TimeZone, local: civil::DateTime) -> Result<Instant, DomainError> {
    if local.subsec_nanosecond() != 0 {
        return Err(DomainError::SubsecondTime);
    }
    let ambiguous = tz.to_ambiguous_timestamp(local);
    match ambiguous.offset() {
        AmbiguousOffset::Unambiguous { .. } => ambiguous
            .unambiguous()
            .map(Instant::from)
            .map_err(|_| DomainError::TimeOutOfRange),
        AmbiguousOffset::Gap { .. } => Err(DomainError::NonexistentLocalTime),
        AmbiguousOffset::Fold { .. } => Err(DomainError::AmbiguousLocalTime),
    }
}

/// The wall-clock time of `instant` in `tz`.
pub fn to_local(tz: &TimeZone, instant: Instant) -> civil::DateTime {
    tz.to_datetime(instant.to_timestamp())
}
