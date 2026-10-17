use jiff::tz::TimeZone;

use crate::Instant;

/// Everything non-deterministic a decision or read may depend on. The driver builds it from
/// the system clock and the OS time zone; tests build it from a simulated clock and zone.
#[derive(Debug, Clone)]
pub struct Ctx {
    /// The current instant, to the second.
    pub now: Instant,
    /// Used for local dates and to convert typed local times to UTC.
    pub tz: TimeZone,
}

impl Ctx {
    pub fn new(now: Instant, tz: TimeZone) -> Self {
        Self { now, tz }
    }
}
