//! Startup order (spec §2, plan.md step 6): lock → backup if the database exists →
//! migrations → load and check.

mod common;

use common::{Scratch, WEB, ctx, files_in, raw, setup_catalog};
use tt_app::{App, AppConfig, AppError};
use tt_core::{Command, Settings, TaskChoice};
use tt_store::{MIGRATIONS, Migration};

#[test]
fn a_new_database_is_created_without_a_backup() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    assert_eq!(app.status().broken, None);
    assert!(app.status().warnings.is_empty());
    assert!(files_in(&scratch.backups()).is_empty());
}

#[test]
fn an_existing_database_is_backed_up_on_every_start() {
    let scratch = Scratch::new();
    drop(scratch.open("2026-10-17T08:00"));
    drop(scratch.open("2026-10-17T09:00"));
    drop(scratch.open("2026-10-17T10:00"));
    assert_eq!(
        files_in(&scratch.backups()),
        [
            "timetracker-20261017T070000Z.db",
            "timetracker-20261017T080000Z.db"
        ]
    );
}

#[test]
fn the_backup_folder_setting_and_the_override_are_honoured() {
    let scratch = Scratch::new();
    let custom = scratch.dir.path().join("onedrive").join("tt");
    let app = scratch.open("2026-10-17T08:00");
    app.execute(
        &ctx("2026-10-17T08:00"),
        Command::UpdateSettings {
            settings: Settings {
                backup_dir: Some(custom.display().to_string()),
                backup_keep: 1,
                ..Settings::default()
            },
        },
    )
    .unwrap();
    drop(app);
    drop(scratch.open("2026-10-17T09:00"));
    drop(scratch.open("2026-10-17T10:00"));
    assert_eq!(
        files_in(&custom),
        ["timetracker-20261017T080000Z.db"],
        "keep = 1"
    );

    let other = scratch.dir.path().join("override");
    let config = AppConfig {
        backup_dir_override: Some(other.clone()),
        ..AppConfig::new(scratch.db())
    };
    drop(App::open(config, &ctx("2026-10-17T11:00")).unwrap());
    assert_eq!(files_in(&other), ["timetracker-20261017T090000Z.db"]);
}

/// The real migrations plus one more, so an existing database has one pending.
static WITH_TEST_MIGRATION: &[Migration] = &[
    MIGRATIONS[0],
    MIGRATIONS[1],
    Migration {
        version: 3,
        name: "test marker",
        sql: "CREATE TABLE test_marker (id INTEGER PRIMARY KEY) STRICT;",
    },
];

#[test]
fn the_backup_is_taken_before_migrations() {
    let scratch = Scratch::new();
    drop(scratch.open("2026-10-17T08:00"));
    let config = AppConfig {
        migrations: WITH_TEST_MIGRATION,
        ..AppConfig::new(scratch.db())
    };
    let app = App::open(config, &ctx("2026-10-17T09:00")).unwrap();
    assert_eq!(app.status().broken, None);
    drop(app);

    let has_marker = |db: &std::path::Path| -> bool {
        raw(db)
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name = 'test_marker'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
            == 1
    };
    assert!(has_marker(&scratch.db()), "the live database was migrated");
    let backup = scratch.backups().join("timetracker-20261017T070000Z.db");
    assert!(
        !has_marker(&backup),
        "the backup holds the pre-migration schema"
    );
}

/// Points the backup folder at a path below a regular file, so creating it fails.
fn break_backup_folder(scratch: &Scratch) {
    let blocker = scratch.dir.path().join("not-a-folder");
    std::fs::write(&blocker, "x").unwrap();
    let app = scratch.open("2026-10-17T08:00");
    app.execute(
        &ctx("2026-10-17T08:00"),
        Command::UpdateSettings {
            settings: Settings {
                backup_dir: Some(blocker.join("sub").display().to_string()),
                ..Settings::default()
            },
        },
    )
    .unwrap();
}

#[test]
fn a_failed_backup_only_warns_when_nothing_is_migrated() {
    let scratch = Scratch::new();
    break_backup_folder(&scratch);
    let app = scratch.open("2026-10-17T09:00");
    let status = app.status();
    assert_eq!(status.broken, None);
    assert_eq!(status.warnings.len(), 1);
    assert!(
        status.warnings[0].contains("backup"),
        "{:?}",
        status.warnings
    );
}

#[test]
fn a_failed_backup_before_a_migration_refuses_to_migrate() {
    let scratch = Scratch::new();
    break_backup_folder(&scratch);
    let config = AppConfig {
        migrations: WITH_TEST_MIGRATION,
        ..AppConfig::new(scratch.db())
    };
    let app = App::open(config, &ctx("2026-10-17T09:00")).unwrap();
    let reason = app.status().broken.expect("broken");
    assert!(reason.contains("not migrating"), "{reason}");
    let pending: i64 = raw(&scratch.db())
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name = 'test_marker'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(pending, 0, "the migration didn't run");
}

#[test]
fn a_failed_prune_only_warns_even_before_a_migration() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    app.execute(
        &ctx("2026-10-17T08:00"),
        Command::UpdateSettings {
            settings: Settings {
                backup_keep: 1,
                ..Settings::default()
            },
        },
    )
    .unwrap();
    drop(app);
    // An old "backup" that can't be deleted: a folder with a backup file name.
    std::fs::create_dir_all(scratch.backups().join("timetracker-20200101T000000Z.db")).unwrap();
    let config = AppConfig {
        migrations: WITH_TEST_MIGRATION,
        ..AppConfig::new(scratch.db())
    };
    let app = App::open(config, &ctx("2026-10-17T09:00")).unwrap();
    let status = app.status();
    assert_eq!(
        status.broken, None,
        "the backup was written, so it migrates"
    );
    assert_eq!(status.warnings.len(), 1);
    assert!(
        status.warnings[0].contains("old backups"),
        "{:?}",
        status.warnings
    );
    assert!(files_in(&scratch.backups()).contains(&"timetracker-20261017T070000Z.db".to_owned()));
}

#[test]
fn invalid_data_on_load_accepts_no_writes() {
    let scratch = Scratch::new();
    let app = scratch.open("2026-10-17T08:00");
    setup_catalog(&app);
    drop(app);
    // Two overlapping entries: the database has no backstop for R12, the core detects it.
    raw(&scratch.db())
        .execute_batch(
            "INSERT INTO time_entry (id, project_id, start_utc, end_utc, created_at, updated_at)
             VALUES (1, 1, 1000, 5000, 1000, 1000), (2, 1, 2000, 3000, 1000, 1000);
             UPDATE counter SET next_id = 3 WHERE name = 'entry';",
        )
        .unwrap();
    let app = scratch.open("2026-10-17T09:00");
    let reason = app.status().broken.expect("broken");
    assert!(reason.contains("overlap"), "{reason}");
    assert!(matches!(
        app.execute(
            &ctx("2026-10-17T09:00"),
            Command::Start {
                project: WEB,
                task: TaskChoice::NoTask
            }
        ),
        Err(AppError::Broken(_))
    ));
}

#[test]
fn a_second_open_of_the_same_database_is_refused() {
    let scratch = Scratch::new();
    let _first = scratch.open("2026-10-17T08:00");
    let second = App::open(AppConfig::new(scratch.db()), &ctx("2026-10-17T08:00"));
    assert!(matches!(second, Err(AppError::Locked(_))));
}
