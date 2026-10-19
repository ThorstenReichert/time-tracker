use jiff::{civil, tz::TimeZone};

use crate::{DomainError, Instant};

/// The local date an instant falls on. Entries count on the date of their start (R18 step 1).
pub fn local_date(instant: Instant, tz: &TimeZone) -> civil::Date {
    tz.to_datetime(instant.to_timestamp()).date()
}

/// The instants `[from, to)` of a local day. Days around DST changes are 23 or 25 hours long,
/// and a day that starts in a DST gap starts at the first instant that exists.
pub fn day_bounds(date: civil::Date, tz: &TimeZone) -> Result<(Instant, Instant), DomainError> {
    let start_of = |d: civil::Date| {
        d.to_zoned(tz.clone())
            .and_then(|z| z.start_of_day())
            .map(|z| Instant::from(z.timestamp()))
            .map_err(|_| DomainError::TimeOutOfRange)
    };
    let next = date.tomorrow().map_err(|_| DomainError::TimeOutOfRange)?;
    Ok((start_of(date)?, start_of(next)?))
}
