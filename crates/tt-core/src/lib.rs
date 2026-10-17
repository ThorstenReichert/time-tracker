//! Functional core of the time tracker: domain types, the Decider (`decide`/`evolve`),
//! invariants, rounding and read models. Pure and deterministic: no IO, no clock, no
//! time-zone lookup. Everything non-deterministic comes in through `Ctx`.
