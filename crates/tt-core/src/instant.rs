use std::fmt;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// A point in time in UTC, as whole Unix seconds (spec §2 "Time handling"). This is also how
/// the database stores it. Always within jiff's supported range, so it converts to a
/// [`Timestamp`] without failing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub struct Instant(i64);

impl Instant {
    /// `None` if the value is outside the range jiff supports (years −9999 to 9999).
    pub fn from_unix_seconds(seconds: i64) -> Option<Self> {
        Timestamp::from_second(seconds).ok().map(|_| Self(seconds))
    }

    pub const fn unix_seconds(self) -> i64 {
        self.0
    }

    pub fn to_timestamp(self) -> Timestamp {
        Timestamp::from_second(self.0).expect("Instant is always in jiff's range")
    }

    /// Seconds from `earlier` to `self` (negative if `earlier` is later).
    pub const fn seconds_since(self, earlier: Instant) -> i64 {
        self.0 - earlier.0
    }

    /// `None` if the result leaves the supported range.
    pub fn plus_seconds(self, seconds: i64) -> Option<Self> {
        self.0
            .checked_add(seconds)
            .and_then(Self::from_unix_seconds)
    }
}

/// Truncates to the second ("times are recorded to the second", R5).
impl From<Timestamp> for Instant {
    fn from(ts: Timestamp) -> Self {
        Self(ts.as_second())
    }
}

impl TryFrom<i64> for Instant {
    type Error = String;

    fn try_from(seconds: i64) -> Result<Self, Self::Error> {
        Self::from_unix_seconds(seconds).ok_or_else(|| format!("instant out of range: {seconds}"))
    }
}

impl From<Instant> for i64 {
    fn from(instant: Instant) -> Self {
        instant.0
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_timestamp().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_subseconds_from_timestamps() {
        let ts: Timestamp = "2026-10-17T09:00:05.999Z".parse().unwrap();
        assert_eq!(Instant::from(ts).unix_seconds(), ts.as_second());
        assert_eq!(Instant::from(ts).to_string(), "2026-10-17T09:00:05Z");
    }

    #[test]
    fn rejects_out_of_range_seconds() {
        assert!(Instant::from_unix_seconds(i64::MAX).is_none());
        assert!(serde_json::from_str::<Instant>(&i64::MIN.to_string()).is_err());
        assert_eq!(serde_json::to_string(&Instant(42)).unwrap(), "42");
    }
}
