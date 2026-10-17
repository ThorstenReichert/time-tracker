//! Functional core of the time tracker: domain types, the Decider (`decide`/`evolve`),
//! invariants, rounding and read models. Pure and deterministic: no IO, no clock, no
//! time-zone lookup. Everything non-deterministic comes in through [`Ctx`].

mod ctx;
mod error;
mod ids;
mod instant;
mod name;
pub mod rounding;

#[cfg(feature = "test-support")]
pub mod testing;

pub use ctx::Ctx;
pub use error::{DomainError, NameProblem};
pub use ids::{ClientId, EntryId, ProjectId, TaskId};
pub use instant::Instant;
pub use name::Name;
pub use rounding::{RoundingDirection, RoundingLevel, RoundingRule, RoundingUnit};
