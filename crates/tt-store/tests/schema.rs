//! Migrations and the SQL backstops (plan.md step 5). The backstops are fed events the core
//! would never produce, to prove the database rejects them on its own.

use tempfile::TempDir;
use tt_core::testing::{ctx_berlin, when};
use tt_core::{
    Abbreviation, ClientId, Command, Entry, EntryId, Event, Instant, Name, ProjectId, RoundingRule,
    State, TaskChoice, TaskId,
};
use tt_store::{MIGRATIONS, Store, StoreError};

fn fresh() -> (TempDir, Store) {
    let dir = TempDir::new().unwrap();
    let (mut store, info) = Store::open(&dir.path().join("t.db"), MIGRATIONS).unwrap();
    assert!(!info.existed);
    assert_eq!(info.pending_migrations, MIGRATIONS.len());
    store.migrate().unwrap();
    (dir, store)
}

/// Executes commands through the core and applies their events, like tt-app does.
fn exec(store: &mut Store, state: State, local_now: &str, cmds: Vec<Command>) -> State {
    let ctx = ctx_berlin(local_now);
    cmds.into_iter().fold(state, |s, cmd| {
        let (events, next) = when(&s, &ctx, cmd).unwrap();
        store.apply(&events).unwrap();
        next
    })
}

/// Acme (1) / Web (1) with task Review (1); Beta (2) / API (2).
fn catalog(store: &mut Store) -> State {
    exec(
        store,
        State::default(),
        "2026-10-17T08:00",
        vec![
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
                project: ProjectId::new(1),
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

fn running(id: u64, project: u64, task: Option<u64>, start: i64) -> Entry {
    let start = Instant::from_unix_seconds(start).unwrap();
    Entry {
        id: EntryId::new(id),
        project: ProjectId::new(project),
        task: task.map(TaskId::new),
        start,
        end: None,
        note: None,
        created_at: start,
        updated_at: start,
    }
}

fn is_constraint(err: StoreError) -> bool {
    matches!(
        err,
        StoreError::Sqlite(rusqlite::Error::SqliteFailure(e, _))
            if e.code == rusqlite::ErrorCode::ConstraintViolation
    )
}

#[test]
fn migrations_run_once_and_are_recorded() {
    let (dir, store) = fresh();
    drop(store);
    let (_, info) = Store::open(&dir.path().join("t.db"), MIGRATIONS).unwrap();
    assert!(info.existed);
    assert_eq!(info.pending_migrations, 0);
}

#[test]
fn an_empty_database_loads_as_the_empty_state() {
    let (_dir, store) = fresh();
    let state = store.load().unwrap();
    assert_eq!(state, State::default());
    state.check_invariants().unwrap();
}

#[test]
fn backstop_one_running_entry() {
    let (_dir, mut store) = fresh();
    catalog(&mut store);
    store
        .apply(&[Event::EntryStarted {
            entry: running(1, 1, None, 1_000),
        }])
        .unwrap();
    let err = store
        .apply(&[Event::EntryStarted {
            entry: running(2, 2, None, 2_000),
        }])
        .unwrap_err();
    assert!(is_constraint(err));
}

#[test]
fn backstop_names_unique_ignoring_ascii_case() {
    let (_dir, mut store) = fresh();
    let state = catalog(&mut store);
    let mut acme = state.catalog().client(ClientId::new(1)).unwrap().clone();
    acme.id = ClientId::new(3);
    acme.name = Name::parse("ACME").unwrap();
    let err = store
        .apply(&[Event::ClientCreated { client: acme }])
        .unwrap_err();
    assert!(is_constraint(err));
}

#[test]
fn backstop_task_belongs_to_the_entrys_project() {
    let (_dir, mut store) = fresh();
    catalog(&mut store);
    // Task 1 belongs to project 1, not 2.
    let err = store
        .apply(&[Event::EntryStarted {
            entry: running(1, 2, Some(1), 1_000),
        }])
        .unwrap_err();
    assert!(is_constraint(err));
    // Without a task, the composite key is skipped.
    store
        .apply(&[Event::EntryStarted {
            entry: running(1, 2, None, 1_000),
        }])
        .unwrap();
}

#[test]
fn backstop_end_after_start() {
    let (_dir, mut store) = fresh();
    catalog(&mut store);
    store
        .apply(&[Event::EntryStarted {
            entry: running(1, 1, None, 1_000),
        }])
        .unwrap();
    let err = store
        .apply(&[Event::EntryStopped {
            id: EntryId::new(1),
            end: Instant::from_unix_seconds(1_000).unwrap(),
            updated_at: Instant::from_unix_seconds(1_000).unwrap(),
        }])
        .unwrap_err();
    assert!(is_constraint(err));
}

#[test]
fn a_failing_event_rolls_back_the_whole_batch() {
    let (_dir, mut store) = fresh();
    let state = catalog(&mut store);
    let before = store.load().unwrap();
    assert_eq!(before, state);
    let err = store
        .apply(&[
            Event::EntryStarted {
                entry: running(1, 1, None, 1_000),
            },
            Event::EntryStarted {
                entry: running(2, 2, None, 2_000),
            },
        ])
        .unwrap_err();
    assert!(is_constraint(err));
    assert_eq!(store.load().unwrap(), before);
}

#[test]
fn an_update_of_a_missing_row_is_an_error() {
    let (_dir, mut store) = fresh();
    catalog(&mut store);
    let err = store
        .apply(&[Event::EntryDeleted {
            id: EntryId::new(42),
        }])
        .unwrap_err();
    assert!(matches!(err, StoreError::RowMissing("entry", 42)));
}

#[test]
fn a_switch_applies_in_order_stop_then_start() {
    let (_dir, mut store) = fresh();
    let state = catalog(&mut store);
    let state = exec(
        &mut store,
        state,
        "2026-10-17T09:00",
        vec![Command::Start {
            project: ProjectId::new(1),
            task: TaskChoice::NoTask,
        }],
    );
    let state = exec(
        &mut store,
        state,
        "2026-10-17T10:22",
        vec![Command::Start {
            project: ProjectId::new(2),
            task: TaskChoice::New("Deploy".into()),
        }],
    );
    assert_eq!(store.load().unwrap(), state);
}

// Abbreviations (R24, migration 0002).

#[test]
fn a_0001_database_migrates_to_abbreviations_and_keeps_its_data() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("t.db");
    // A 0.1.0 database: only the first migration, with a client in it.
    {
        let (mut store, _) = Store::open(&path, &MIGRATIONS[..1]).unwrap();
        store.migrate().unwrap();
        drop(store);
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute(
            "INSERT INTO client (id, name, round_unit_min, round_direction, round_level, created_at)
             VALUES (1, 'Acme', 15, 'nearest', 'project', 0)",
            [],
        )
        .unwrap();
        conn.execute("UPDATE counter SET next_id = 2 WHERE name = 'client'", [])
            .unwrap();
    }
    let (mut store, info) = Store::open(&path, MIGRATIONS).unwrap();
    assert_eq!(info.pending_migrations, 1);
    store.migrate().unwrap();
    let state = store.load().unwrap();
    state.check_invariants().unwrap();
    let acme = state.catalog().client(ClientId::new(1)).unwrap();
    assert_eq!(acme.name.as_str(), "Acme");
    assert_eq!(acme.abbreviation, None);
}

#[test]
fn abbreviations_are_stored_and_loaded() {
    let (_dir, mut store) = fresh();
    let state = catalog(&mut store);
    let state = exec(
        &mut store,
        state,
        "2026-10-17T09:00",
        vec![
            Command::UpdateClient {
                id: ClientId::new(1),
                name: "Acme".into(),
                abbreviation: Some("AC".into()),
                rule: RoundingRule::default(),
            },
            Command::UpdateProject {
                id: ProjectId::new(1),
                name: "Web".into(),
                abbreviation: Some("W".into()),
            },
            Command::UpdateTask {
                id: TaskId::new(1),
                name: "Review".into(),
                abbreviation: Some("Rev".into()),
            },
            Command::CreateProject {
                client: ClientId::new(2),
                name: "Mobile".into(),
                abbreviation: Some("Mob".into()),
            },
            Command::UpdateProject {
                id: ProjectId::new(1),
                name: "Web".into(),
                abbreviation: None,
            },
        ],
    );
    assert_eq!(store.load().unwrap(), state);
}

#[test]
fn backstop_abbreviations_unique_per_parent_ignoring_ascii_case() {
    let (_dir, mut store) = fresh();
    let state = catalog(&mut store);
    let abbr = |text: &str| Some(Abbreviation::parse(text).unwrap());
    store
        .apply(&[Event::ProjectUpdated {
            id: ProjectId::new(1),
            name: Name::parse("Web").unwrap(),
            abbreviation: abbr("W"),
        }])
        .unwrap();
    // Another client's project may use it.
    store
        .apply(&[Event::ProjectUpdated {
            id: ProjectId::new(2),
            name: Name::parse("API").unwrap(),
            abbreviation: abbr("W"),
        }])
        .unwrap();
    let mut mobile = state.catalog().project(ProjectId::new(1)).unwrap().clone();
    mobile.id = ProjectId::new(3);
    mobile.name = Name::parse("Mobile").unwrap();
    mobile.abbreviation = abbr("w");
    let err = store
        .apply(&[Event::ProjectCreated { project: mobile }])
        .unwrap_err();
    assert!(is_constraint(err));
}
