//! Test support (feature `test-support`): simulated time zone and clock helpers. Step 3 adds
//! the random command generator. This is the only module where the purity lints are relaxed,
//! because tests need tzdb lookups to build a `Ctx`.
#![allow(clippy::disallowed_methods)]

use jiff::{civil, tz::TimeZone};

use crate::{Command, Ctx, DomainError, Event, Instant, State, decide, evolve_all};

/// Given: the state after `events`, starting from an empty state.
pub fn given(events: &[Event]) -> State {
    evolve_all(State::default(), events)
}

/// When: decides `cmd` and, if it's accepted, also evolves the state with its events, so
/// the invariants are checked on the result (evolve does that in tests).
pub fn when(state: &State, ctx: &Ctx, cmd: Command) -> Result<(Vec<Event>, State), DomainError> {
    let events = decide(state, ctx, &cmd)?;
    let next = evolve_all(state.clone(), &events);
    Ok((events, next))
}

/// Runs commands that must all succeed, returning the final state.
pub fn run(state: State, ctx: &Ctx, cmds: impl IntoIterator<Item = Command>) -> State {
    cmds.into_iter().fold(state, |s, cmd| {
        let (_, next) = when(&s, ctx, cmd.clone()).unwrap_or_else(|e| panic!("{cmd:?}: {e}"));
        next
    })
}

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
