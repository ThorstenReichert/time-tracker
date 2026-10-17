//! Per-client rounding rules (R17) and the rounding function (R18 step 3). Rounding is used
//! only by the daily overview and the billing error (R16), never on stored entries.

use serde::{Deserialize, Serialize};

/// A rounding unit in minutes. Only the units that give exact values at 2 decimal hours
/// exist (OQ-4, AC-3), so a unit is always a divisor of 60.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(try_from = "u32", into = "u32")]
pub enum RoundingUnit {
    M6,
    M12,
    #[default]
    M15,
    M30,
    M60,
}

impl RoundingUnit {
    pub const ALL: [RoundingUnit; 5] = [Self::M6, Self::M12, Self::M15, Self::M30, Self::M60];

    pub const fn minutes(self) -> u32 {
        match self {
            Self::M6 => 6,
            Self::M12 => 12,
            Self::M15 => 15,
            Self::M30 => 30,
            Self::M60 => 60,
        }
    }

    pub const fn seconds(self) -> i64 {
        self.minutes() as i64 * 60
    }

    pub fn from_minutes(minutes: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|u| u.minutes() == minutes)
    }
}

impl TryFrom<u32> for RoundingUnit {
    type Error = String;

    fn try_from(minutes: u32) -> Result<Self, Self::Error> {
        Self::from_minutes(minutes).ok_or_else(|| {
            format!("rounding unit must be 6, 12, 15, 30 or 60 minutes, not {minutes}")
        })
    }
}

impl From<RoundingUnit> for u32 {
    fn from(unit: RoundingUnit) -> Self {
        unit.minutes()
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum RoundingDirection {
    Up,
    #[default]
    Nearest,
    Down,
}

impl RoundingDirection {
    /// The text stored in the database (`client.round_direction`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Nearest => "nearest",
            Self::Down => "down",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Up, Self::Nearest, Self::Down]
            .into_iter()
            .find(|d| d.as_str() == text)
    }
}

/// What gets rounded: each project's day total, or the client's day total once (R17).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum RoundingLevel {
    #[default]
    Project,
    Client,
}

impl RoundingLevel {
    /// The text stored in the database (`client.round_level`).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Client => "client",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Project, Self::Client]
            .into_iter()
            .find(|l| l.as_str() == text)
    }
}

/// A client's rounding rule (R17). The default is 15 minutes, nearest, project level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoundingRule {
    pub unit: RoundingUnit,
    pub direction: RoundingDirection,
    pub level: RoundingLevel,
}

/// Rounds a group's day total `secs` (≥ 0) to a multiple of the rule's unit `u` (R18 step 3):
/// `down = ⌊t/u⌋·u`, `up = ⌈t/u⌉·u`, `nearest = ⌊(t + u/2)/u⌋·u` (exactly half rounds up).
/// Zero stays zero. The level doesn't matter here; it decides what `secs` is summed over.
pub fn round(secs: i64, rule: &RoundingRule) -> i64 {
    debug_assert!(secs >= 0, "day totals are never negative: {secs}");
    let u = rule.unit.seconds();
    let units = match rule.direction {
        RoundingDirection::Down => secs.div_euclid(u),
        RoundingDirection::Up => (secs + u - 1).div_euclid(u),
        RoundingDirection::Nearest => (secs + u / 2).div_euclid(u),
    };
    units * u
}
