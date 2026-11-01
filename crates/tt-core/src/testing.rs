//! Test support (feature `test-support`): given/when helpers, a simulated time zone and
//! clock, and the random command generator for the model-based property tests (spec §2
//! "Testing approach"). This is the only module where the purity lints are relaxed, because
//! tests need tzdb lookups to build a `Ctx`.
#![allow(clippy::disallowed_methods)]

use jiff::{Span, civil, tz::TimeZone};
use proptest::prelude::*;

use crate::{
    ClientId, Command, Ctx, DomainError, EntryId, Event, Instant, ProjectId, RoundingDirection,
    RoundingLevel, RoundingRule, RoundingUnit, Settings, State, TaskChoice, TaskId, decide,
    evolve_all, to_local,
};

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

// ---------------------------------------------------------------------------------------
// Random command sequences

/// One step of a simulated session: the clock moves forward, then an operation runs.
#[derive(Debug, Clone)]
pub struct Step {
    pub advance_secs: i64,
    pub op: Op,
}

/// An abstract command. IDs depend on what earlier steps created, so operations pick
/// existing items by index (`u8`, wrapped around) and are turned into a concrete [`Command`]
/// by [`materialize`] against the current state. A pick of 250 or more refers to an item
/// that doesn't exist, to exercise rejections.
#[derive(Debug, Clone)]
pub enum Op {
    CreateClient {
        name: u8,
        abbreviation: Option<u8>,
        rule: RoundingRule,
    },
    UpdateClient {
        client: u8,
        name: u8,
        abbreviation: Option<u8>,
        rule: RoundingRule,
    },
    CreateProject {
        client: u8,
        name: u8,
        abbreviation: Option<u8>,
    },
    UpdateProject {
        project: u8,
        name: u8,
        abbreviation: Option<u8>,
    },
    CreateTask {
        project: u8,
        name: u8,
        abbreviation: Option<u8>,
    },
    UpdateTask {
        task: u8,
        name: u8,
        abbreviation: Option<u8>,
    },
    Start {
        project: u8,
        task: TaskPick,
    },
    /// Stop now, or at a typed time this many minutes before now (negative: in the future).
    Stop {
        minutes_back: Option<i32>,
    },
    AddEntry {
        project: u8,
        task: TaskPick,
        minutes_back: i32,
        length_minutes: i32,
        note: u8,
    },
    EditEntry {
        entry: u8,
        project: u8,
        task: TaskPick,
        minutes_back: i32,
        length_minutes: Option<i32>,
        note: u8,
    },
    DeleteEntry {
        entry: u8,
    },
    UpdateSettings {
        recents_n: u32,
    },
}

#[derive(Debug, Clone)]
pub enum TaskPick {
    NoTask,
    Existing(u8),
    New(u8),
}

/// A small pool, so that names collide (in any case) often.
const NAMES: [&str; 8] = [
    "Acme", "acme", "Beta", "Web", "API", "Review", "Ops", " web ",
];
/// Abbreviations (R24): some clash with the names above, one is blank (no abbreviation).
const ABBREVIATIONS: [&str; 6] = ["ACM", "web", "Rev", "  ", "api", "X"];
const NOTES: [&str; 4] = ["", "   ", "standup", "fixed bug\nand reviewed"];

fn arb_rule() -> impl Strategy<Value = RoundingRule> {
    (
        proptest::sample::select(RoundingUnit::ALL.to_vec()),
        proptest::sample::select(vec![
            RoundingDirection::Up,
            RoundingDirection::Nearest,
            RoundingDirection::Down,
        ]),
        proptest::sample::select(vec![RoundingLevel::Project, RoundingLevel::Client]),
    )
        .prop_map(|(unit, direction, level)| RoundingRule {
            unit,
            direction,
            level,
        })
}

fn arb_task_pick() -> impl Strategy<Value = TaskPick> {
    prop_oneof![
        2 => Just(TaskPick::NoTask),
        3 => any::<u8>().prop_map(TaskPick::Existing),
        1 => any::<u8>().prop_map(TaskPick::New),
    ]
}

fn abbreviation() -> impl Strategy<Value = Option<u8>> {
    proptest::option::of(any::<u8>())
}

fn arb_op() -> impl Strategy<Value = Op> {
    let minutes_back = prop_oneof![9 => 0..600i32, 1 => -30..0i32];
    let length = prop_oneof![9 => 1..240i32, 1 => -10..1i32];
    prop_oneof![
        2 => (any::<u8>(), abbreviation(), arb_rule())
            .prop_map(|(name, abbreviation, rule)| Op::CreateClient { name, abbreviation, rule }),
        1 => (any::<u8>(), any::<u8>(), abbreviation(), arb_rule())
            .prop_map(|(client, name, abbreviation, rule)| Op::UpdateClient { client, name, abbreviation, rule }),
        3 => (any::<u8>(), any::<u8>(), abbreviation())
            .prop_map(|(client, name, abbreviation)| Op::CreateProject { client, name, abbreviation }),
        1 => (any::<u8>(), any::<u8>(), abbreviation())
            .prop_map(|(project, name, abbreviation)| Op::UpdateProject { project, name, abbreviation }),
        2 => (any::<u8>(), any::<u8>(), abbreviation())
            .prop_map(|(project, name, abbreviation)| Op::CreateTask { project, name, abbreviation }),
        1 => (any::<u8>(), any::<u8>(), abbreviation())
            .prop_map(|(task, name, abbreviation)| Op::UpdateTask { task, name, abbreviation }),
        8 => (any::<u8>(), arb_task_pick()).prop_map(|(project, task)| Op::Start { project, task }),
        4 => proptest::option::of(minutes_back.clone())
            .prop_map(|minutes_back| Op::Stop { minutes_back }),
        4 => (any::<u8>(), arb_task_pick(), minutes_back.clone(), length.clone(), any::<u8>())
            .prop_map(|(project, task, minutes_back, length_minutes, note)| Op::AddEntry {
                project, task, minutes_back, length_minutes, note,
            }),
        3 => (any::<u8>(), any::<u8>(), arb_task_pick(), minutes_back, proptest::option::of(length), any::<u8>())
            .prop_map(|(entry, project, task, minutes_back, length_minutes, note)| Op::EditEntry {
                entry, project, task, minutes_back, length_minutes, note,
            }),
        1 => any::<u8>().prop_map(|entry| Op::DeleteEntry { entry }),
        1 => (0..12u32).prop_map(|recents_n| Op::UpdateSettings { recents_n }),
    ]
}

fn arb_advance() -> impl Strategy<Value = i64> {
    prop_oneof![
        2 => Just(0i64),
        3 => 1..120i64,
        5 => 120..2 * 3600i64,
    ]
}

/// Up to `max_steps` steps of a simulated session.
pub fn arb_steps(max_steps: usize) -> impl Strategy<Value = Vec<Step>> {
    proptest::collection::vec(
        (arb_advance(), arb_op()).prop_map(|(advance_secs, op)| Step { advance_secs, op }),
        1..=max_steps,
    )
}

/// The simulated clock's start: Berlin, a few hours before one of the two 2026 DST changes,
/// so a session crosses it once it has a catalog, and typed times can land in the gap or
/// the fold.
pub fn arb_start_ctx() -> impl Strategy<Value = Ctx> {
    proptest::sample::select(vec![
        "2026-03-28T23:00",
        "2026-03-29T01:30",
        "2026-10-24T23:00",
        "2026-10-25T01:30",
    ])
    .prop_map(ctx_berlin)
}

/// Moves the clock forward.
pub fn advance(ctx: &Ctx, secs: i64) -> Ctx {
    Ctx::new(
        ctx.now.plus_seconds(secs).expect("clock stays in range"),
        ctx.tz.clone(),
    )
}

fn pick<T: Copy>(choice: u8, items: &[T], missing: T) -> T {
    if choice >= 250 || items.is_empty() {
        missing
    } else {
        items[choice as usize % items.len()]
    }
}

fn name(choice: u8) -> String {
    NAMES[choice as usize % NAMES.len()].to_owned()
}

fn abbreviation_of(choice: Option<u8>) -> Option<String> {
    choice.map(|c| ABBREVIATIONS[c as usize % ABBREVIATIONS.len()].to_owned())
}

/// A typed local time `minutes_back` minutes of wall-clock time before now. Wall-clock
/// arithmetic, so near a DST change it can land in the gap or the fold.
fn local_back(ctx: &Ctx, minutes_back: i32) -> civil::DateTime {
    to_local(&ctx.tz, ctx.now)
        .checked_sub(Span::new().minutes(minutes_back))
        .expect("in range")
}

fn task_choice(pick_: &TaskPick, state: &State, project: ProjectId) -> TaskChoice {
    match pick_ {
        TaskPick::NoTask => TaskChoice::NoTask,
        // Mostly tasks of the chosen project; any task otherwise, to exercise the check.
        TaskPick::Existing(choice) => {
            let own: Vec<TaskId> = state.catalog.tasks_of(project).map(|t| t.id).collect();
            let all: Vec<TaskId> = state.catalog.tasks().map(|t| t.id).collect();
            let pool = if choice % 8 == 0 || own.is_empty() {
                all
            } else {
                own
            };
            TaskChoice::Existing(pick(*choice, &pool, TaskId::new(999)))
        }
        TaskPick::New(choice) => TaskChoice::New(name(*choice)),
    }
}

/// Turns an abstract operation into a concrete command against `state`.
pub fn materialize(op: &Op, state: &State, ctx: &Ctx) -> Command {
    let clients: Vec<ClientId> = state.catalog.clients().map(|c| c.id).collect();
    let projects: Vec<ProjectId> = state.catalog.projects().map(|p| p.id).collect();
    let tasks: Vec<TaskId> = state.catalog.tasks().map(|t| t.id).collect();
    let entries: Vec<EntryId> = state.entries.all().map(|e| e.id).collect();
    let project_of = |choice| pick(choice, &projects, ProjectId::new(999));
    match op {
        Op::CreateClient {
            name: n,
            abbreviation: a,
            rule,
        } => Command::CreateClient {
            name: name(*n),
            abbreviation: abbreviation_of(*a),
            rule: *rule,
        },
        Op::UpdateClient {
            client,
            name: n,
            abbreviation: a,
            rule,
        } => Command::UpdateClient {
            id: pick(*client, &clients, ClientId::new(999)),
            name: name(*n),
            abbreviation: abbreviation_of(*a),
            rule: *rule,
        },
        Op::CreateProject {
            client,
            name: n,
            abbreviation: a,
        } => Command::CreateProject {
            client: pick(*client, &clients, ClientId::new(999)),
            name: name(*n),
            abbreviation: abbreviation_of(*a),
        },
        Op::UpdateProject {
            project,
            name: n,
            abbreviation: a,
        } => Command::UpdateProject {
            id: project_of(*project),
            name: name(*n),
            abbreviation: abbreviation_of(*a),
        },
        Op::CreateTask {
            project,
            name: n,
            abbreviation: a,
        } => Command::CreateTask {
            project: project_of(*project),
            name: name(*n),
            abbreviation: abbreviation_of(*a),
        },
        Op::UpdateTask {
            task,
            name: n,
            abbreviation: a,
        } => Command::UpdateTask {
            id: pick(*task, &tasks, TaskId::new(999)),
            name: name(*n),
            abbreviation: abbreviation_of(*a),
        },
        Op::Start { project, task } => {
            let project = project_of(*project);
            Command::Start {
                project,
                task: task_choice(task, state, project),
            }
        }
        Op::Stop { minutes_back } => Command::Stop {
            at: minutes_back.map(|m| local_back(ctx, m)),
        },
        Op::AddEntry {
            project,
            task,
            minutes_back,
            length_minutes,
            note,
        } => {
            let project = project_of(*project);
            let start = local_back(ctx, *minutes_back);
            Command::AddEntry {
                project,
                task: task_choice(task, state, project),
                start,
                end: local_back(ctx, minutes_back - length_minutes),
                note: Some(NOTES[*note as usize % NOTES.len()].to_owned()),
            }
        }
        Op::EditEntry {
            entry,
            project,
            task,
            minutes_back,
            length_minutes,
            note,
        } => {
            let project = project_of(*project);
            Command::EditEntry {
                id: pick(*entry, &entries, EntryId::new(999)),
                project,
                task: task_choice(task, state, project),
                start: local_back(ctx, *minutes_back),
                end: length_minutes.map(|len| local_back(ctx, minutes_back - len)),
                note: Some(NOTES[*note as usize % NOTES.len()].to_owned()),
            }
        }
        Op::DeleteEntry { entry } => Command::DeleteEntry {
            id: pick(*entry, &entries, EntryId::new(999)),
        },
        Op::UpdateSettings { recents_n } => Command::UpdateSettings {
            settings: Settings {
                recents_n: *recents_n,
                ..state.catalog.settings.clone()
            },
        },
    }
}
