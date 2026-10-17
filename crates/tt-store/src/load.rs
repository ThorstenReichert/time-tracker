//! Loads the whole `State` (spec §2 "Whole world in memory"). Values are parsed into the
//! core's types; anything the core can't represent is `StoreError::Corrupt`. The caller runs
//! `State::check_invariants` on the result.

use rusqlite::Row;
use tt_core::{
    Catalog, Client, ClientId, Counters, DecimalSeparator, Entries, Entry, EntryId, Instant, Name,
    Note, Project, ProjectId, RoundingDirection, RoundingLevel, RoundingRule, RoundingUnit,
    Settings, State, Task, TaskId,
};

use crate::{Store, StoreError, keys};

fn corrupt(what: impl Into<String>) -> StoreError {
    StoreError::Corrupt(what.into())
}

fn instant(seconds: i64) -> Result<Instant, StoreError> {
    Instant::from_unix_seconds(seconds).ok_or_else(|| corrupt(format!("instant {seconds}")))
}

fn opt_instant(seconds: Option<i64>) -> Result<Option<Instant>, StoreError> {
    seconds.map(instant).transpose()
}

fn name(text: &str) -> Result<Name, StoreError> {
    Name::parse(text).map_err(|e| corrupt(format!("name {text:?}: {e}")))
}

fn id(raw: i64) -> Result<u64, StoreError> {
    u64::try_from(raw).map_err(|_| corrupt(format!("id {raw}")))
}

impl Store {
    pub fn load(&self) -> Result<State, StoreError> {
        let catalog = Catalog::from_parts(
            self.load_clients()?,
            self.load_projects()?,
            self.load_tasks()?,
            self.load_settings()?,
            self.load_counters()?,
        );
        Ok(State::from_parts(
            catalog,
            Entries::from_parts(self.load_entries()?),
        ))
    }

    fn rows<T>(
        &self,
        sql: &str,
        map: impl Fn(&Row<'_>) -> Result<T, StoreError>,
    ) -> Result<Vec<T>, StoreError> {
        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map(row)?);
        }
        Ok(out)
    }

    fn load_clients(&self) -> Result<Vec<Client>, StoreError> {
        self.rows(
            "SELECT id, name, round_unit_min, round_direction, round_level, archived_at, created_at
             FROM client",
            |r| {
                let unit: u32 = r.get(2)?;
                let direction: String = r.get(3)?;
                let level: String = r.get(4)?;
                Ok(Client {
                    id: ClientId::new(id(r.get(0)?)?),
                    name: name(&r.get::<_, String>(1)?)?,
                    rule: RoundingRule {
                        unit: RoundingUnit::from_minutes(unit)
                            .ok_or_else(|| corrupt(format!("rounding unit {unit}")))?,
                        direction: RoundingDirection::parse(&direction)
                            .ok_or_else(|| corrupt(format!("direction {direction:?}")))?,
                        level: RoundingLevel::parse(&level)
                            .ok_or_else(|| corrupt(format!("level {level:?}")))?,
                    },
                    archived_at: opt_instant(r.get(5)?)?,
                    created_at: instant(r.get(6)?)?,
                })
            },
        )
    }

    fn load_projects(&self) -> Result<Vec<Project>, StoreError> {
        self.rows(
            "SELECT id, client_id, name, archived_at, created_at FROM project",
            |r| {
                Ok(Project {
                    id: ProjectId::new(id(r.get(0)?)?),
                    client: ClientId::new(id(r.get(1)?)?),
                    name: name(&r.get::<_, String>(2)?)?,
                    archived_at: opt_instant(r.get(3)?)?,
                    created_at: instant(r.get(4)?)?,
                })
            },
        )
    }

    fn load_tasks(&self) -> Result<Vec<Task>, StoreError> {
        self.rows(
            "SELECT id, project_id, name, archived_at, created_at FROM task",
            |r| {
                Ok(Task {
                    id: TaskId::new(id(r.get(0)?)?),
                    project: ProjectId::new(id(r.get(1)?)?),
                    name: name(&r.get::<_, String>(2)?)?,
                    archived_at: opt_instant(r.get(3)?)?,
                    created_at: instant(r.get(4)?)?,
                })
            },
        )
    }

    fn load_entries(&self) -> Result<Vec<Entry>, StoreError> {
        self.rows(
            "SELECT id, project_id, task_id, start_utc, end_utc, note, created_at, updated_at
             FROM time_entry",
            |r| {
                let note: Option<String> = r.get(5)?;
                let note = match note {
                    None => None,
                    Some(text) => Some(
                        Note::parse(&text)
                            .ok()
                            .flatten()
                            .ok_or_else(|| corrupt(format!("note {text:?}")))?,
                    ),
                };
                Ok(Entry {
                    id: EntryId::new(id(r.get(0)?)?),
                    project: ProjectId::new(id(r.get(1)?)?),
                    task: r
                        .get::<_, Option<i64>>(2)?
                        .map(id)
                        .transpose()?
                        .map(TaskId::new),
                    start: instant(r.get(3)?)?,
                    end: opt_instant(r.get(4)?)?,
                    note,
                    created_at: instant(r.get(6)?)?,
                    updated_at: instant(r.get(7)?)?,
                })
            },
        )
    }

    fn load_settings(&self) -> Result<Settings, StoreError> {
        let mut settings = Settings::default();
        for (key, value) in self.rows("SELECT key, value FROM setting", |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })? {
            let bad = || corrupt(format!("setting {key} = {value:?}"));
            match key.as_str() {
                keys::HOTKEY => settings.hotkey = Some(value.clone()),
                keys::DECIMAL_SEPARATOR => {
                    settings.decimal_separator = DecimalSeparator::parse(&value).ok_or_else(bad)?
                }
                keys::RECENTS_N => settings.recents_n = value.parse().map_err(|_| bad())?,
                keys::BACKUP_DIR => settings.backup_dir = Some(value.clone()),
                keys::BACKUP_KEEP => settings.backup_keep = value.parse().map_err(|_| bad())?,
                _ => {} // a key from a newer version; ignored
            }
        }
        Ok(settings)
    }

    fn load_counters(&self) -> Result<Counters, StoreError> {
        let mut counters = Counters::default();
        for (name, next) in self.rows("SELECT name, next_id FROM counter", |r| {
            Ok((r.get::<_, String>(0)?, id(r.get(1)?)?))
        })? {
            match name.as_str() {
                "client" => counters.next_client = next,
                "project" => counters.next_project = next,
                "task" => counters.next_task = next,
                "entry" => counters.next_entry = next,
                _ => {} // e.g. "favorite" (slice 2)
            }
        }
        Ok(counters)
    }
}
