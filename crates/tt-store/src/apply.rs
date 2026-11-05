//! Applies events in order, in one transaction. Events are self-contained, so nothing is
//! read from `State` here (OQ-9). A switch arrives as "stop, then start", so the
//! `one_running` index is never violated in between.

use rusqlite::{Transaction, params};
use tt_core::{Abbreviation, Entry, Event, Settings};

use crate::{Store, StoreError, keys};

impl Store {
    /// Applies `events` atomically: all of them, or none if any statement fails.
    pub fn apply(&mut self, events: &[Event]) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        for event in events {
            apply_one(&tx, event)?;
        }
        tx.commit()?;
        Ok(())
    }
}

fn i(id: u64) -> i64 {
    i64::try_from(id).expect("IDs stay below i64::MAX")
}

/// Fails if an update or delete hit no row: the database and memory disagree.
fn expect_one(changed: usize, table: &'static str, id: u64) -> Result<(), StoreError> {
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::RowMissing(table, id))
    }
}

fn bump_counter(tx: &Transaction<'_>, name: &str, id: u64) -> Result<(), StoreError> {
    tx.execute(
        "UPDATE counter SET next_id = max(next_id, ?1) WHERE name = ?2",
        params![i(id) + 1, name],
    )?;
    Ok(())
}

/// An abbreviation as stored: its text, or NULL for none (R24).
fn abbr(abbreviation: Option<&Abbreviation>) -> Option<&str> {
    abbreviation.map(Abbreviation::as_str)
}

fn apply_one(tx: &Transaction<'_>, event: &Event) -> Result<(), StoreError> {
    match event {
        Event::ClientCreated { client } => {
            tx.execute(
                "INSERT INTO client (id, name, round_unit_min, round_direction, round_level,
                                     archived_at, created_at, abbreviation)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    i(client.id.get()),
                    client.name.as_str(),
                    client.rule.unit.minutes(),
                    client.rule.direction.as_str(),
                    client.rule.level.as_str(),
                    client.archived_at.map(|a| a.unix_seconds()),
                    client.created_at.unix_seconds(),
                    abbr(client.abbreviation.as_ref()),
                ],
            )?;
            bump_counter(tx, "client", client.id.get())
        }
        Event::ClientUpdated {
            id,
            name,
            abbreviation,
            rule,
        } => {
            let changed = tx.execute(
                "UPDATE client SET name = ?2, round_unit_min = ?3, round_direction = ?4,
                                   round_level = ?5, abbreviation = ?6
                 WHERE id = ?1",
                params![
                    i(id.get()),
                    name.as_str(),
                    rule.unit.minutes(),
                    rule.direction.as_str(),
                    rule.level.as_str(),
                    abbr(abbreviation.as_ref()),
                ],
            )?;
            expect_one(changed, "client", id.get())
        }
        Event::ProjectCreated { project } => {
            tx.execute(
                "INSERT INTO project (id, client_id, name, abbreviation, archived_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    i(project.id.get()),
                    i(project.client.get()),
                    project.name.as_str(),
                    abbr(project.abbreviation.as_ref()),
                    project.archived_at.map(|a| a.unix_seconds()),
                    project.created_at.unix_seconds(),
                ],
            )?;
            bump_counter(tx, "project", project.id.get())
        }
        Event::ProjectUpdated {
            id,
            name,
            abbreviation,
        } => {
            let changed = tx.execute(
                "UPDATE project SET name = ?2, abbreviation = ?3 WHERE id = ?1",
                params![i(id.get()), name.as_str(), abbr(abbreviation.as_ref())],
            )?;
            expect_one(changed, "project", id.get())
        }
        Event::ProjectArchived { id, at } => {
            let changed = tx.execute(
                "UPDATE project SET archived_at = ?2 WHERE id = ?1",
                params![i(id.get()), at.unix_seconds()],
            )?;
            expect_one(changed, "project", id.get())
        }
        Event::ProjectUnarchived { id } => {
            let changed = tx.execute(
                "UPDATE project SET archived_at = NULL WHERE id = ?1",
                params![i(id.get())],
            )?;
            expect_one(changed, "project", id.get())
        }
        Event::TaskCreated { task } => {
            tx.execute(
                "INSERT INTO task (id, project_id, name, abbreviation, archived_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    i(task.id.get()),
                    i(task.project.get()),
                    task.name.as_str(),
                    abbr(task.abbreviation.as_ref()),
                    task.archived_at.map(|a| a.unix_seconds()),
                    task.created_at.unix_seconds(),
                ],
            )?;
            bump_counter(tx, "task", task.id.get())
        }
        Event::TaskUpdated {
            id,
            name,
            abbreviation,
        } => {
            let changed = tx.execute(
                "UPDATE task SET name = ?2, abbreviation = ?3 WHERE id = ?1",
                params![i(id.get()), name.as_str(), abbr(abbreviation.as_ref())],
            )?;
            expect_one(changed, "task", id.get())
        }
        Event::SettingsChanged { settings } => write_settings(tx, settings),
        Event::EntryStarted { entry } | Event::EntryAdded { entry } => {
            insert_entry(tx, entry)?;
            bump_counter(tx, "entry", entry.id.get())
        }
        Event::EntryStopped {
            id,
            end,
            updated_at,
        } => {
            let changed = tx.execute(
                "UPDATE time_entry SET end_utc = ?2, updated_at = ?3
                 WHERE id = ?1 AND end_utc IS NULL",
                params![i(id.get()), end.unix_seconds(), updated_at.unix_seconds()],
            )?;
            expect_one(changed, "running entry", id.get())
        }
        Event::EntryUpdated { entry } => {
            let changed = tx.execute(
                "UPDATE time_entry SET project_id = ?2, task_id = ?3, start_utc = ?4,
                                       end_utc = ?5, note = ?6, created_at = ?7, updated_at = ?8
                 WHERE id = ?1",
                entry_params(entry),
            )?;
            expect_one(changed, "entry", entry.id.get())
        }
        Event::EntryDeleted { id } => {
            let changed =
                tx.execute("DELETE FROM time_entry WHERE id = ?1", params![i(id.get())])?;
            expect_one(changed, "entry", id.get())
        }
    }
}

type EntryParams<'a> = (
    i64,
    i64,
    Option<i64>,
    i64,
    Option<i64>,
    Option<&'a str>,
    i64,
    i64,
);

fn entry_params(entry: &Entry) -> EntryParams<'_> {
    (
        i(entry.id.get()),
        i(entry.project.get()),
        entry.task.map(|t| i(t.get())),
        entry.start.unix_seconds(),
        entry.end.map(|e| e.unix_seconds()),
        entry.note.as_ref().map(|n| n.as_str()),
        entry.created_at.unix_seconds(),
        entry.updated_at.unix_seconds(),
    )
}

fn insert_entry(tx: &Transaction<'_>, entry: &Entry) -> Result<(), StoreError> {
    tx.execute(
        "INSERT INTO time_entry (id, project_id, task_id, start_utc, end_utc, note, created_at,
                                 updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        entry_params(entry),
    )?;
    Ok(())
}

/// The settings event carries all settings, so the table is rewritten as a whole.
fn write_settings(tx: &Transaction<'_>, settings: &Settings) -> Result<(), StoreError> {
    tx.execute("DELETE FROM setting", [])?;
    let mut insert = tx.prepare("INSERT INTO setting (key, value) VALUES (?1, ?2)")?;
    if let Some(hotkey) = &settings.hotkey {
        insert.execute(params![keys::HOTKEY, hotkey])?;
    }
    insert.execute(params![
        keys::DECIMAL_SEPARATOR,
        settings.decimal_separator.as_str()
    ])?;
    insert.execute(params![keys::RECENTS_N, settings.recents_n.to_string()])?;
    if let Some(dir) = &settings.backup_dir {
        insert.execute(params![keys::BACKUP_DIR, dir])?;
    }
    insert.execute(params![keys::BACKUP_KEEP, settings.backup_keep.to_string()])?;
    Ok(())
}
