//! Embedded SQL migrations, recorded in `schema_migrations`.

use rusqlite::OptionalExtension;

use crate::{Store, StoreError};

#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// All migrations, in order. Never edit a released one; add a new one.
pub static MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "init",
        sql: include_str!("../migrations/0001_init.sql"),
    },
    Migration {
        version: 2,
        name: "abbreviations",
        sql: include_str!("../migrations/0002_abbreviations.sql"),
    },
];

const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS schema_migrations (
    version    INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    applied_at INTEGER NOT NULL
) STRICT";

impl Store {
    pub(crate) fn pending_migrations(&self) -> Result<Vec<Migration>, StoreError> {
        let exists: Option<String> = self
            .conn
            .query_row(
                "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = 'schema_migrations'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let applied: Vec<i64> = match exists {
            None => Vec::new(),
            Some(_) => {
                let mut stmt = self
                    .conn
                    .prepare("SELECT version FROM schema_migrations ORDER BY version")?;
                stmt.query_map([], |r| r.get(0))?
                    .collect::<Result<_, _>>()?
            }
        };
        if let Some(unknown) = applied
            .iter()
            .find(|v| !self.migrations.iter().any(|m| m.version == **v))
        {
            return Err(StoreError::UnknownMigration(*unknown));
        }
        Ok(self
            .migrations
            .iter()
            .filter(|m| !applied.contains(&m.version))
            .copied()
            .collect())
    }

    /// Applies the pending migrations, each in its own transaction.
    pub fn migrate(&mut self) -> Result<(), StoreError> {
        self.conn.execute(CREATE_TABLE, [])?;
        for migration in self.pending_migrations()? {
            let tx = self.conn.transaction()?;
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, unixepoch())",
                (migration.version, migration.name),
            )?;
            tx.commit()?;
        }
        Ok(())
    }
}
