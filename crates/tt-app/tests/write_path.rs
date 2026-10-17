//! The single write path (plan.md step 6): execute, publish, and recovery from store errors.

mod common;

use std::sync::mpsc;

use common::{API, Scratch, WEB, ctx, raw, setup_catalog};
use tt_app::AppError;
use tt_core::{Command, DomainError, EntryId, Event, TaskChoice};
use tt_store::{MIGRATIONS, Store};

fn start(project: tt_core::ProjectId) -> Command {
    Command::Start {
        project,
        task: TaskChoice::NoTask,
    }
}

fn loaded(scratch: &Scratch) -> tt_core::State {
    let (store, _) = Store::open(&scratch.db(), MIGRATIONS).unwrap();
    store.load().unwrap()
}

#[test]
fn execute_writes_swaps_and_publishes_in_order() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    let (tx, rx) = mpsc::channel();
    app.subscribe(move |events| tx.send(events.to_vec()).unwrap());
    setup_catalog(&app);

    app.execute(&ctx("2026-10-17T09:00"), start(WEB)).unwrap();
    let events = app.execute(&ctx("2026-10-17T10:22"), start(API)).unwrap();
    assert!(matches!(
        events.as_slice(),
        [Event::EntryStopped { .. }, Event::EntryStarted { .. }]
    ));

    let published: Vec<Vec<Event>> = rx.try_iter().collect();
    assert_eq!(published.len(), 6, "four catalog commands, two starts");
    assert_eq!(published.last().unwrap(), &events);

    let running = app.running(&ctx("2026-10-17T10:30")).unwrap().unwrap();
    assert_eq!(
        (running.combo.label.as_str(), running.duration),
        ("Beta / API", 8 * 60)
    );
    assert_eq!(app.snapshot().unwrap(), loaded(&scratch));
}

#[test]
fn a_rejected_command_writes_and_publishes_nothing() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    setup_catalog(&app);
    let (tx, rx) = mpsc::channel();
    app.subscribe(move |events| tx.send(events.to_vec()).unwrap());
    let before = app.snapshot().unwrap();

    let err = app
        .execute(&ctx("2026-10-17T09:00"), Command::Stop { at: None })
        .unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::NotRunning)));
    // A no-op (same combo) is accepted with no events, and isn't published either.
    app.execute(&ctx("2026-10-17T09:00"), start(WEB)).unwrap();
    let noop = app.execute(&ctx("2026-10-17T09:05"), start(WEB)).unwrap();
    assert!(noop.is_empty());
    assert_eq!(rx.try_iter().count(), 1);
    assert_ne!(app.snapshot().unwrap(), before);
}

#[test]
fn a_store_error_reloads_state_from_the_database() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    setup_catalog(&app);
    let before = app.snapshot().unwrap();

    // Fault injection: the next insert of an entry fails inside the transaction.
    let conn = raw(&scratch.db());
    conn.execute_batch(
        "CREATE TRIGGER inject BEFORE INSERT ON time_entry
         BEGIN SELECT RAISE(ABORT, 'injected failure'); END;",
    )
    .unwrap();
    let err = app
        .execute(&ctx("2026-10-17T09:00"), start(WEB))
        .unwrap_err();
    assert!(matches!(err, AppError::Store(_)), "{err:?}");
    assert_eq!(
        app.snapshot().unwrap(),
        before,
        "the candidate was discarded"
    );
    assert_eq!(app.status().broken, None);

    conn.execute_batch("DROP TRIGGER inject").unwrap();
    app.execute(&ctx("2026-10-17T09:00"), start(WEB)).unwrap();
    assert_eq!(app.snapshot().unwrap(), loaded(&scratch));
}

#[test]
fn the_database_is_the_authority_after_out_of_band_changes() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    setup_catalog(&app);
    app.execute(&ctx("2026-10-17T09:00"), start(WEB)).unwrap();

    // The running entry disappears behind the app's back; stopping it can't find the row.
    raw(&scratch.db())
        .execute("DELETE FROM time_entry", [])
        .unwrap();
    let err = app
        .execute(&ctx("2026-10-17T10:00"), Command::Stop { at: None })
        .unwrap_err();
    assert!(matches!(err, AppError::Store(_)), "{err:?}");
    // The reload brought the memory in line with the database.
    assert!(
        app.snapshot()
            .unwrap()
            .entries()
            .get(EntryId::new(1))
            .is_none()
    );
    assert_eq!(app.snapshot().unwrap(), loaded(&scratch));
}

#[test]
fn a_failed_reload_stops_all_writes_and_reads() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    setup_catalog(&app);
    // The save fails, and so does the reload (a table it reads is gone).
    raw(&scratch.db())
        .execute_batch(
            "CREATE TRIGGER inject BEFORE INSERT ON time_entry
             BEGIN SELECT RAISE(ABORT, 'injected failure'); END;
             ALTER TABLE setting RENAME TO setting_gone;",
        )
        .unwrap();
    let err = app
        .execute(&ctx("2026-10-17T09:00"), start(WEB))
        .unwrap_err();
    assert!(matches!(err, AppError::Store(_)));
    let broken = app.status().broken.expect("broken after a failed reload");
    assert!(broken.contains("reload"), "{broken}");
    assert!(matches!(
        app.execute(&ctx("2026-10-17T09:01"), start(API)),
        Err(AppError::Broken(_))
    ));
    assert!(matches!(app.catalog(), Err(AppError::Broken(_))));
}

#[test]
fn reads_go_through_the_read_models() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    setup_catalog(&app);
    app.execute(&ctx("2026-10-17T09:00"), start(WEB)).unwrap();
    app.execute(&ctx("2026-10-17T10:22"), start(API)).unwrap();
    app.execute(&ctx("2026-10-17T11:00"), Command::Stop { at: None })
        .unwrap();

    let c = ctx("2026-10-17T12:00");
    let day = "2026-10-17".parse().unwrap();
    let overview = app.daily_overview(&c, day).unwrap();
    let totals: Vec<_> = overview
        .clients
        .iter()
        .map(|b| (b.name.as_str(), b.total.raw))
        .collect();
    assert_eq!(totals, [("Acme", 82 * 60), ("Beta", 38 * 60)]);
    assert_eq!(app.day_entries(&c, day).unwrap().len(), 2);
    let recents: Vec<_> = app
        .recents()
        .unwrap()
        .into_iter()
        .map(|r| r.label)
        .collect();
    assert_eq!(recents, ["Beta / API", "Acme / Web"]);
    assert_eq!(app.quick_switch("acweb").unwrap()[0].label, "Acme / Web");
    assert!(app.task_matches(WEB, "x").unwrap().offer_create.is_some());
}
