//! Functional core of the time tracker: domain types, the Decider (`decide`/`evolve`),
//! invariants, rounding and read models. Pure and deterministic: no IO, no clock, no
//! time-zone lookup. Everything non-deterministic comes in through [`Ctx`].

mod catalog;
mod command;
mod ctx;
mod decide;
mod entries;
mod error;
mod event;
mod evolve;
mod ids;
mod instant;
mod invariants;
mod localtime;
mod name;
mod note;
pub mod read;
pub mod rounding;
mod state;

#[cfg(feature = "test-support")]
pub mod testing;

pub use catalog::{Catalog, Client, Counters, DecimalSeparator, Project, Settings, Task};
pub use command::{Command, TaskChoice};
pub use ctx::Ctx;
pub use decide::decide;
pub use entries::{Combo, Entries, Entry};
pub use error::{DomainError, InvariantViolation, ItemRef, NameProblem, SettingProblem};
pub use event::Event;
pub use evolve::{evolve, evolve_all};
pub use ids::{ClientId, EntryId, ProjectId, TaskId};
pub use instant::Instant;
pub use localtime::{to_instant, to_local};
pub use name::Name;
pub use note::Note;
pub use rounding::{RoundingDirection, RoundingLevel, RoundingRule, RoundingUnit};
pub use state::State;
