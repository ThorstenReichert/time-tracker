//! Given State / when Command / then Events-or-error: tracking (R4, R5, R10–R13) and typed
//! local times (OQ-6), plan.md step 3.

use jiff::civil;
use tt_core::testing::{at, berlin, ctx_berlin, run, when};
use tt_core::{
    ClientId, Combo, Command, Ctx, DomainError, Entry, EntryId, Event, Instant, Note, ProjectId,
    RoundingRule, State, TaskChoice, TaskId,
};

const WEB: ProjectId = ProjectId::new(1);
const API: ProjectId = ProjectId::new(2);
const REVIEW: TaskId = TaskId::new(1);

fn ctx(local: &str) -> Ctx {
    ctx_berlin(local)
}

fn t(local: &str) -> Instant {
    at(&berlin(), local)
}

fn dt(local: &str) -> civil::DateTime {
    local.parse().unwrap()
}

/// Acme (1) with Web (1) and its task Review (1); Beta (2) with API (2). No entries.
fn catalog() -> State {
    let c = ctx("2026-10-17T08:00");
    run(
        State::default(),
        &c,
        [
            Command::CreateClient {
                name: "Acme".into(),
                abbreviation: None,
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: ClientId::new(1),
                name: "Web".into(),
                abbreviation: None,
            },
            Command::CreateTask {
                project: WEB,
                name: "Review".into(),
                abbreviation: None,
            },
            Command::CreateClient {
                name: "Beta".into(),
                abbreviation: None,
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: ClientId::new(2),
                name: "API".into(),
                abbreviation: None,
            },
        ],
    )
}

fn start(project: ProjectId, task: TaskChoice) -> Command {
    Command::Start { project, task }
}

fn add(project: ProjectId, start: &str, end: &str) -> Command {
    Command::AddEntry {
        project,
        task: TaskChoice::NoTask,
        start: dt(start),
        end: dt(end),
        note: None,
    }
}

/// The catalog, plus Web running since 09:00 (entry 1).
fn web_running() -> State {
    run(
        catalog(),
        &ctx("2026-10-17T09:00"),
        [start(WEB, TaskChoice::NoTask)],
    )
}

fn running_entry(id: u64, project: ProjectId, task: Option<TaskId>, start: Instant) -> Entry {
    Entry {
        id: EntryId::new(id),
        project,
        task,
        start,
        end: None,
        note: None,
        created_at: start,
        updated_at: start,
    }
}

#[test]
fn start_when_idle_creates_a_running_entry_at_now() {
    let (events, state) = when(
        &catalog(),
        &ctx("2026-10-17T09:00"),
        start(WEB, TaskChoice::Existing(REVIEW)),
    )
    .unwrap();
    let entry = running_entry(1, WEB, Some(REVIEW), t("2026-10-17T09:00"));
    assert_eq!(
        events,
        [Event::EntryStarted {
            entry: entry.clone()
        }]
    );
    assert_eq!(state.entries().running(), Some(&entry));
}

#[test]
fn switch_stops_then_starts_at_the_same_instant() {
    let now = t("2026-10-17T10:22");
    let (events, state) = when(
        &web_running(),
        &ctx("2026-10-17T10:22"),
        start(API, TaskChoice::NoTask),
    )
    .unwrap();
    assert_eq!(
        events,
        [
            Event::EntryStopped {
                id: EntryId::new(1),
                end: now,
                updated_at: now
            },
            Event::EntryStarted {
                entry: running_entry(2, API, None, now)
            },
        ]
    );
    let stopped = state.entries().get(EntryId::new(1)).unwrap();
    assert_eq!(stopped.end, Some(now));
    assert_eq!(
        state.entries().running().map(|e| e.id),
        Some(EntryId::new(2))
    );
}

#[test]
fn switch_to_a_new_task_creates_it_between_stop_and_start() {
    let (events, _) = when(
        &web_running(),
        &ctx("2026-10-17T10:00"),
        start(WEB, TaskChoice::New("Deploy".into())),
    )
    .unwrap();
    let kinds: Vec<&str> = events
        .iter()
        .map(|e| match e {
            Event::EntryStopped { .. } => "stopped",
            Event::TaskCreated { task } => {
                assert_eq!((task.id, task.name.as_str()), (TaskId::new(2), "Deploy"));
                "task"
            }
            Event::EntryStarted { entry } => {
                assert_eq!(entry.task, Some(TaskId::new(2)));
                "started"
            }
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(kinds, ["stopped", "task", "started"]);
}

#[test]
fn switching_to_the_running_combo_does_nothing() {
    let (events, _) = when(
        &web_running(),
        &ctx("2026-10-17T10:00"),
        start(WEB, TaskChoice::NoTask),
    )
    .unwrap();
    assert!(events.is_empty());
    // A different task on the same project is a different combo.
    let (events, _) = when(
        &web_running(),
        &ctx("2026-10-17T10:00"),
        start(WEB, TaskChoice::Existing(REVIEW)),
    )
    .unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn ending_an_entry_in_the_second_it_started_discards_it() {
    let same_second = ctx("2026-10-17T09:00");
    let (events, state) = when(&web_running(), &same_second, Command::Stop { at: None }).unwrap();
    assert_eq!(
        events,
        [Event::EntryDeleted {
            id: EntryId::new(1)
        }]
    );
    assert!(state.entries().get(EntryId::new(1)).is_none());

    let (events, _) = when(&web_running(), &same_second, start(API, TaskChoice::NoTask)).unwrap();
    assert!(matches!(
        events.as_slice(),
        [Event::EntryDeleted { .. }, Event::EntryStarted { .. }]
    ));
}

#[test]
fn stop_at_a_typed_past_time_fixes_a_forgotten_stop() {
    let (events, _) = when(
        &web_running(),
        &ctx("2026-10-17T18:00"),
        Command::Stop {
            at: Some(dt("2026-10-17T12:30")),
        },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::EntryStopped {
            id: EntryId::new(1),
            end: t("2026-10-17T12:30"),
            updated_at: t("2026-10-17T18:00")
        }]
    );
}

#[test]
fn stop_rejects_bad_times_and_an_idle_timer() {
    let state = web_running();
    let c = ctx("2026-10-17T12:00");
    let stop_at = |local: &str| Command::Stop {
        at: Some(dt(local)),
    };
    assert_eq!(
        when(&state, &c, stop_at("2026-10-17T09:00")).unwrap_err(),
        DomainError::EndNotAfterStart
    );
    assert_eq!(
        when(&state, &c, stop_at("2026-10-17T08:00")).unwrap_err(),
        DomainError::EndNotAfterStart
    );
    assert_eq!(
        when(&state, &c, stop_at("2026-10-17T12:01")).unwrap_err(),
        DomainError::EndInFuture
    );
    assert_eq!(
        when(&catalog(), &c, Command::Stop { at: None }).unwrap_err(),
        DomainError::NotRunning
    );
}

#[test]
fn a_clock_that_went_back_is_rejected_not_adjusted() {
    let earlier = ctx("2026-10-17T08:59");
    assert_eq!(
        when(&web_running(), &earlier, start(API, TaskChoice::NoTask)).unwrap_err(),
        DomainError::EndNotAfterStart
    );
    // A finished entry that ends after `now` also blocks a start.
    let state = run(
        catalog(),
        &ctx("2026-10-17T11:00"),
        [add(WEB, "2026-10-17T09:00", "2026-10-17T10:00")],
    );
    assert_eq!(
        when(
            &state,
            &ctx("2026-10-17T09:30"),
            start(API, TaskChoice::NoTask)
        )
        .unwrap_err(),
        DomainError::Overlap {
            with: EntryId::new(1),
            start: t("2026-10-17T09:00"),
            end: Some(t("2026-10-17T10:00"))
        }
    );
}

#[test]
fn add_entry_after_the_fact() {
    let (events, _) = when(
        &catalog(),
        &ctx("2026-10-17T18:00"),
        Command::AddEntry {
            project: WEB,
            task: TaskChoice::Existing(REVIEW),
            start: dt("2026-10-17T09:00"),
            end: dt("2026-10-17T10:22"),
            note: Some("  standup + review ".into()),
        },
    )
    .unwrap();
    assert_eq!(
        events,
        [Event::EntryAdded {
            entry: Entry {
                id: EntryId::new(1),
                project: WEB,
                task: Some(REVIEW),
                start: t("2026-10-17T09:00"),
                end: Some(t("2026-10-17T10:22")),
                note: Note::parse("standup + review").unwrap(),
                created_at: t("2026-10-17T18:00"),
                updated_at: t("2026-10-17T18:00"),
            }
        }]
    );
}

#[test]
fn overlaps_are_rejected_naming_the_entry_and_boundaries_may_touch() {
    let c = ctx("2026-10-17T18:00");
    let state = run(
        catalog(),
        &c,
        [add(WEB, "2026-10-17T09:00", "2026-10-17T10:00")],
    );
    let conflict = DomainError::Overlap {
        with: EntryId::new(1),
        start: t("2026-10-17T09:00"),
        end: Some(t("2026-10-17T10:00")),
    };
    assert_eq!(
        when(&state, &c, add(API, "2026-10-17T09:59", "2026-10-17T11:00")).unwrap_err(),
        conflict
    );
    assert_eq!(
        when(&state, &c, add(API, "2026-10-17T08:00", "2026-10-17T12:00")).unwrap_err(),
        conflict
    );
    assert_eq!(
        when(&state, &c, add(API, "2026-10-17T09:15", "2026-10-17T09:45")).unwrap_err(),
        conflict
    );
    // Half-open intervals: sharing a boundary is fine on both sides.
    assert!(when(&state, &c, add(API, "2026-10-17T10:00", "2026-10-17T11:00")).is_ok());
    assert!(when(&state, &c, add(API, "2026-10-17T08:00", "2026-10-17T09:00")).is_ok());
}

#[test]
fn the_running_entry_counts_until_now_for_overlaps() {
    let state = web_running();
    assert_eq!(
        when(
            &state,
            &ctx("2026-10-17T12:00"),
            add(API, "2026-10-17T11:00", "2026-10-17T11:30")
        )
        .unwrap_err(),
        DomainError::Overlap {
            with: EntryId::new(1),
            start: t("2026-10-17T09:00"),
            end: None
        }
    );
    assert!(
        when(
            &state,
            &ctx("2026-10-17T12:00"),
            add(API, "2026-10-17T08:00", "2026-10-17T09:00")
        )
        .is_ok()
    );
}

#[test]
fn times_in_the_future_or_backwards_are_rejected() {
    let c = ctx("2026-10-17T12:00");
    let state = catalog();
    assert_eq!(
        when(&state, &c, add(WEB, "2026-10-17T12:01", "2026-10-17T13:00")).unwrap_err(),
        DomainError::StartInFuture
    );
    assert_eq!(
        when(&state, &c, add(WEB, "2026-10-17T11:00", "2026-10-17T12:01")).unwrap_err(),
        DomainError::EndInFuture
    );
    assert_eq!(
        when(&state, &c, add(WEB, "2026-10-17T11:00", "2026-10-17T11:00")).unwrap_err(),
        DomainError::EndNotAfterStart
    );
    // Up to now is fine.
    assert!(when(&state, &c, add(WEB, "2026-10-17T11:00", "2026-10-17T12:00")).is_ok());
}

#[test]
fn typed_times_in_the_dst_gap_or_fold_are_rejected() {
    let c = ctx("2026-10-26T12:00");
    let state = catalog();
    assert_eq!(
        when(&state, &c, add(WEB, "2026-03-29T02:30", "2026-03-29T04:00")).unwrap_err(),
        DomainError::NonexistentLocalTime
    );
    assert_eq!(
        when(&state, &c, add(WEB, "2026-10-25T01:00", "2026-10-25T02:30")).unwrap_err(),
        DomainError::AmbiguousLocalTime
    );
    // Right after the changes, the times are unambiguous again.
    assert!(when(&state, &c, add(WEB, "2026-10-25T03:00", "2026-10-25T04:00")).is_ok());
    assert_eq!(
        when(
            &state,
            &c,
            add(WEB, "2026-10-17T09:00:00.5", "2026-10-17T10:00")
        )
        .unwrap_err(),
        DomainError::SubsecondTime
    );
}

#[test]
fn an_entry_across_the_dst_fold_is_measured_in_real_seconds() {
    // 01:30 to 03:30 local on the night the clocks go back is three real hours.
    let (events, _) = when(
        &catalog(),
        &ctx("2026-10-26T12:00"),
        add(WEB, "2026-10-25T01:30", "2026-10-25T03:30"),
    )
    .unwrap();
    let Event::EntryAdded { entry } = &events[0] else {
        panic!()
    };
    assert_eq!(entry.duration_until(t("2026-10-26T12:00")), 3 * 3600);
}

#[test]
fn editing_the_running_entry_keeps_it_running() {
    let (events, state) = when(
        &web_running(),
        &ctx("2026-10-17T11:00"),
        Command::EditEntry {
            id: EntryId::new(1),
            project: API,
            task: TaskChoice::NoTask,
            start: dt("2026-10-17T08:45"),
            end: None,
            note: Some("moved".into()),
        },
    )
    .unwrap();
    assert_eq!(events.len(), 1);
    let running = state.entries().running().unwrap();
    assert_eq!(
        (running.id, running.project, running.start),
        (EntryId::new(1), API, t("2026-10-17T08:45"))
    );
    assert_eq!(running.note.as_ref().map(Note::as_str), Some("moved"));
    assert_eq!(running.created_at, t("2026-10-17T09:00"));
    assert_eq!(running.updated_at, t("2026-10-17T11:00"));
}

#[test]
fn editing_the_running_entry_with_an_end_stops_it() {
    let (_, state) = when(
        &web_running(),
        &ctx("2026-10-17T11:00"),
        Command::EditEntry {
            id: EntryId::new(1),
            project: WEB,
            task: TaskChoice::NoTask,
            start: dt("2026-10-17T09:00"),
            end: Some(dt("2026-10-17T10:00")),
            note: None,
        },
    )
    .unwrap();
    assert!(state.entries().running().is_none());
}

#[test]
fn editing_a_finished_entry() {
    let c = ctx("2026-10-17T18:00");
    let state = run(
        catalog(),
        &c,
        [
            add(WEB, "2026-10-17T09:00", "2026-10-17T10:00"),
            add(API, "2026-10-17T10:00", "2026-10-17T11:00"),
        ],
    );
    let edit = |start: &str, end: Option<&str>| Command::EditEntry {
        id: EntryId::new(1),
        project: WEB,
        task: TaskChoice::NoTask,
        start: dt(start),
        end: end.map(dt),
        note: None,
    };
    // Moving within its own old interval doesn't conflict with itself.
    assert!(
        when(
            &state,
            &c,
            edit("2026-10-17T09:30", Some("2026-10-17T10:00"))
        )
        .is_ok()
    );
    assert_eq!(
        when(
            &state,
            &c,
            edit("2026-10-17T09:00", Some("2026-10-17T10:30"))
        )
        .unwrap_err(),
        DomainError::Overlap {
            with: EntryId::new(2),
            start: t("2026-10-17T10:00"),
            end: Some(t("2026-10-17T11:00"))
        }
    );
    assert_eq!(
        when(&state, &c, edit("2026-10-17T09:00", None)).unwrap_err(),
        DomainError::EndRequired
    );
    let (events, _) = when(
        &state,
        &c,
        edit("2026-10-17T09:00", Some("2026-10-17T10:00")),
    )
    .unwrap();
    assert!(events.is_empty(), "unchanged edit emits nothing");
}

#[test]
fn a_task_must_belong_to_the_entrys_project() {
    assert_eq!(
        when(
            &catalog(),
            &ctx("2026-10-17T09:00"),
            start(API, TaskChoice::Existing(REVIEW))
        )
        .unwrap_err(),
        DomainError::TaskNotInProject {
            task: REVIEW,
            project: API
        }
    );
    assert_eq!(
        when(
            &catalog(),
            &ctx("2026-10-17T09:00"),
            start(ProjectId::new(9), TaskChoice::NoTask)
        )
        .unwrap_err(),
        DomainError::UnknownProject(ProjectId::new(9))
    );
}

#[test]
fn delete_entry_and_ids_are_never_reused() {
    let c = ctx("2026-10-17T18:00");
    let state = run(
        catalog(),
        &c,
        [
            add(WEB, "2026-10-17T09:00", "2026-10-17T10:00"),
            add(WEB, "2026-10-17T10:00", "2026-10-17T11:00"),
            Command::DeleteEntry {
                id: EntryId::new(2),
            },
        ],
    );
    let (events, _) = when(&state, &c, add(WEB, "2026-10-17T12:00", "2026-10-17T13:00")).unwrap();
    let Event::EntryAdded { entry } = &events[0] else {
        panic!()
    };
    assert_eq!(entry.id, EntryId::new(3));
    assert_eq!(
        when(
            &state,
            &c,
            Command::DeleteEntry {
                id: EntryId::new(2)
            }
        )
        .unwrap_err(),
        DomainError::UnknownEntry(EntryId::new(2))
    );
}

#[test]
fn notes_are_optional_and_limited() {
    let c = ctx("2026-10-17T18:00");
    let with_note = |note: String| Command::AddEntry {
        project: WEB,
        task: TaskChoice::NoTask,
        start: dt("2026-10-17T09:00"),
        end: dt("2026-10-17T10:00"),
        note: Some(note),
    };
    let (events, _) = when(&catalog(), &c, with_note("   ".into())).unwrap();
    assert!(matches!(&events[0], Event::EntryAdded { entry } if entry.note.is_none()));
    assert_eq!(
        when(&catalog(), &c, with_note("x".repeat(Note::MAX_CHARS + 1))).unwrap_err(),
        DomainError::NoteTooLong {
            max: Note::MAX_CHARS
        }
    );
}

#[test]
fn recent_combos_are_distinct_newest_first() {
    let c = ctx("2026-10-17T18:00");
    let state = run(
        catalog(),
        &c,
        [
            add(WEB, "2026-10-17T09:00", "2026-10-17T10:00"),
            add(API, "2026-10-17T10:00", "2026-10-17T11:00"),
            add(WEB, "2026-10-17T11:00", "2026-10-17T12:00"),
        ],
    );
    let combo = |project| Combo {
        project,
        task: None,
    };
    assert_eq!(state.entries().recent_combos(8), [combo(WEB), combo(API)]);
    assert_eq!(state.entries().recent_combos(1), [combo(WEB)]);
}

#[test]
fn state_with_entries_round_trips_through_serde() {
    let state = run(
        web_running(),
        &ctx("2026-10-17T10:00"),
        [start(API, TaskChoice::NoTask)],
    );
    let json = serde_json::to_string(&state).unwrap();
    let back: State = serde_json::from_str(&json).unwrap();
    assert_eq!(back, state);
    assert_eq!(
        back.entries().running().map(|e| e.id),
        Some(EntryId::new(2))
    );
    back.check_invariants().unwrap();
}
