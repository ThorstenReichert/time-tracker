//! Backups, pruning, the single-writer lock and the raw setting read (R22, plan.md step 5).

use std::path::Path;

use tempfile::TempDir;
use tt_core::testing::{ctx_berlin, run};
use tt_core::{ClientId, Command, Event, Instant, RoundingRule, Settings, State};
use tt_store::{DbLock, MIGRATIONS, Store, StoreError, keys, prune_backups, raw_setting};

fn migrated(path: &Path) -> Store {
    let (mut store, _) = Store::open(path, MIGRATIONS).unwrap();
    store.migrate().unwrap();
    store
}

fn at(seconds: i64) -> Instant {
    Instant::from_unix_seconds(seconds).unwrap()
}

fn backups_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn a_backup_is_a_complete_single_file_copy() {
    let dir = TempDir::new().unwrap();
    let mut store = migrated(&dir.path().join("t.db"));
    let ctx = ctx_berlin("2026-10-17T09:00");
    let state = run(
        State::default(),
        &ctx,
        [Command::CreateClient {
            name: "Acme".into(),
            abbreviation: None,
            rule: RoundingRule::default(),
        }],
    );
    let client = state.catalog().client(ClientId::new(1)).unwrap().clone();
    store.apply(&[Event::ClientCreated { client }]).unwrap();

    let backups = dir.path().join("backups");
    let file = store.backup_to(&backups, ctx.now).unwrap();
    assert_eq!(
        file.file_name().unwrap().to_str().unwrap(),
        "timetracker-20261017T070000Z.db"
    );
    assert_eq!(backups_in(&backups), ["timetracker-20261017T070000Z.db"]);
    let (copy, info) = Store::open(&file, MIGRATIONS).unwrap();
    assert_eq!(info.pending_migrations, 0);
    assert_eq!(copy.load().unwrap(), state);
}

#[test]
fn prune_keeps_the_newest_and_leaves_other_files_alone() {
    let dir = TempDir::new().unwrap();
    let store = migrated(&dir.path().join("t.db"));
    let backups = dir.path().join("backups");
    std::fs::create_dir_all(&backups).unwrap();
    std::fs::write(backups.join("notes.txt"), "mine").unwrap();
    for day in 0..20 {
        store
            .backup_to(&backups, at(1_792_224_000 + day * 86_400))
            .unwrap();
        prune_backups(&backups, 14).unwrap();
    }
    let names = backups_in(&backups);
    assert_eq!(names.len(), 15, "14 backups + notes.txt: {names:?}");
    assert!(names.contains(&"notes.txt".to_owned()));
    // The six oldest (days 0..6) are gone.
    assert_eq!(names[0], "notes.txt");
    assert_eq!(names[1], "timetracker-20261023T080000Z.db");
}

#[test]
fn a_backup_in_the_same_second_replaces_the_file() {
    let dir = TempDir::new().unwrap();
    let store = migrated(&dir.path().join("t.db"));
    let backups = dir.path().join("backups");
    store.backup_to(&backups, at(1_792_224_000)).unwrap();
    store.backup_to(&backups, at(1_792_224_000)).unwrap();
    assert_eq!(backups_in(&backups), ["timetracker-20261017T080000Z.db"]);
}

#[test]
fn a_failed_backup_leaves_no_partial_file() {
    let dir = TempDir::new().unwrap();
    let store = migrated(&dir.path().join("t.db"));
    let backups = dir.path().join("backups");
    // A folder under the backup's name makes the final rename fail.
    std::fs::create_dir_all(backups.join("timetracker-20261017T080000Z.db")).unwrap();
    assert!(store.backup_to(&backups, at(1_792_224_000)).is_err());
    assert_eq!(backups_in(&backups), ["timetracker-20261017T080000Z.db"]);
}

#[test]
fn the_lock_is_exclusive_while_held() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("t.db");
    let lock = DbLock::acquire(&db).unwrap();
    assert!(matches!(DbLock::acquire(&db), Err(StoreError::Locked(_))));
    drop(lock);
    DbLock::acquire(&db).unwrap();
}

#[test]
fn raw_setting_reads_before_migration_and_tolerates_missing_parts() {
    let dir = TempDir::new().unwrap();
    let db = dir.path().join("t.db");
    assert_eq!(raw_setting(&db, keys::BACKUP_DIR).unwrap(), None, "no file");

    std::fs::write(&db, b"").unwrap();
    assert_eq!(
        raw_setting(&db, keys::BACKUP_DIR).unwrap(),
        None,
        "empty file"
    );

    let mut store = migrated(&db);
    assert_eq!(raw_setting(&db, keys::BACKUP_DIR).unwrap(), None, "no key");
    store
        .apply(&[Event::SettingsChanged {
            settings: Settings {
                backup_dir: Some(r"D:\backups".into()),
                ..Settings::default()
            },
        }])
        .unwrap();
    assert_eq!(
        raw_setting(&db, keys::BACKUP_DIR).unwrap().as_deref(),
        Some(r"D:\backups")
    );
}
