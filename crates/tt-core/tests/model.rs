//! Model-based property tests (plan.md step 3): random command sequences with a simulated
//! clock in Europe/Berlin around both DST changes. After every step the invariants hold,
//! `decide` is deterministic, and accepted commands leave no start or end in the future.

use std::collections::BTreeMap;

use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;
use tt_core::testing::{Step, advance, arb_start_ctx, arb_steps, materialize};
use tt_core::{
    Command, Ctx, DomainError, Entry, Event, Instant, State, TaskChoice, decide, evolve_all,
};

fn all_entries(state: &State) -> Vec<&Entry> {
    state.entries().started_between(
        Instant::from(jiff::Timestamp::MIN),
        Instant::from(jiff::Timestamp::MAX),
    )
}

/// Runs a session and reports every outcome to `observe`. Returns the first failed check.
fn run_session(
    start: Ctx,
    steps: &[Step],
    mut observe: impl FnMut(&Command, &Result<Vec<Event>, DomainError>),
) -> Result<State, TestCaseError> {
    let mut ctx = start;
    let mut state = State::default();
    for step in steps {
        ctx = advance(&ctx, step.advance_secs);
        let cmd = materialize(&step.op, &state, &ctx);
        let outcome = decide(&state, &ctx, &cmd);
        prop_assert_eq!(
            &outcome,
            &decide(&state, &ctx, &cmd),
            "decide is deterministic"
        );
        observe(&cmd, &outcome);
        let Ok(events) = outcome else {
            continue; // decide takes &State, so a rejection changes nothing
        };
        state = evolve_all(state, &events);
        state
            .check_invariants()
            .map_err(|v| TestCaseError::fail(format!("{v} after {cmd:?}")))?;

        for entry in all_entries(&state) {
            prop_assert!(entry.start <= ctx.now, "start in the future: {entry:?}");
            prop_assert!(
                entry.end.is_none_or(|e| e <= ctx.now),
                "end in the future: {entry:?}"
            );
        }
        match &cmd {
            Command::Start { project, task } if !events.is_empty() => {
                let running = state.entries().running().expect("a timer runs after Start");
                prop_assert_eq!(running.project, *project);
                prop_assert_eq!(running.start, ctx.now);
                if let TaskChoice::Existing(t) = task {
                    prop_assert_eq!(running.task, Some(*t));
                }
            }
            Command::Stop { .. } => prop_assert!(state.entries().running().is_none()),
            _ => {}
        }
    }
    Ok(state)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn invariants_hold_after_every_step(start in arb_start_ctx(), steps in arb_steps(200)) {
        run_session(start, &steps, |_, _| {})?;
    }
}

/// Guards the property test against a generator that only produces rejections: across a
/// fixed set of sessions, every kind of command is accepted and every interesting rejection
/// happens at least once.
#[test]
fn generator_reaches_the_interesting_cases() {
    let mut runner = TestRunner::deterministic();
    let strategy = (arb_start_ctx(), arb_steps(200));
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for _ in 0..64 {
        let (start, steps) = strategy.new_tree(&mut runner).unwrap().current();
        run_session(start, &steps, |cmd, outcome| {
            let kind = match outcome {
                Ok(events) if events.is_empty() => "no-op".to_owned(),
                Ok(events) => match (cmd, events.as_slice()) {
                    (Command::Start { .. }, [Event::EntryStopped { .. }, ..]) => "switch".into(),
                    (Command::Start { .. }, [Event::EntryDeleted { .. }, ..]) => {
                        "zero-length discard".into()
                    }
                    _ => format!("ok {}", command_name(cmd)),
                },
                Err(e) => format!("err {}", error_name(e)),
            };
            *seen.entry(kind).or_default() += 1;
        })
        .unwrap();
    }
    for expected in [
        "ok CreateClient",
        "ok CreateProject",
        "ok CreateTask",
        "ok Start",
        "switch",
        "zero-length discard",
        "ok Stop",
        "ok AddEntry",
        "ok EditEntry",
        "ok DeleteEntry",
        "no-op",
        "err Overlap",
        "err DuplicateName",
        "err NonexistentLocalTime",
        "err AmbiguousLocalTime",
        "err StartInFuture",
        "err EndNotAfterStart",
        "err TaskNotInProject",
    ] {
        assert!(
            seen.contains_key(expected),
            "never saw {expected:?}: {seen:#?}"
        );
    }
}

fn command_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::CreateClient { .. } => "CreateClient",
        Command::UpdateClient { .. } => "UpdateClient",
        Command::CreateProject { .. } => "CreateProject",
        Command::RenameProject { .. } => "RenameProject",
        Command::CreateTask { .. } => "CreateTask",
        Command::RenameTask { .. } => "RenameTask",
        Command::UpdateSettings { .. } => "UpdateSettings",
        Command::Start { .. } => "Start",
        Command::Stop { .. } => "Stop",
        Command::AddEntry { .. } => "AddEntry",
        Command::EditEntry { .. } => "EditEntry",
        Command::DeleteEntry { .. } => "DeleteEntry",
    }
}

fn error_name(e: &DomainError) -> String {
    let debug = format!("{e:?}");
    debug
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or_default()
        .to_owned()
}
