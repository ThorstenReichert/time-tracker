//! SQLite persistence for the time tracker: migrations, loading `State`, applying `Event`s
//! in one transaction, and backups. No business rules: the core already decided; the SQL
//! constraints are only a backstop.

mod apply;
mod backup;
mod load;
mod lock;
mod migrate;
mod raw_setting;

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use thiserror::Error;

pub use backup::{BACKUP_PREFIX, backup_file_name};
pub use lock::DbLock;
pub use migrate::{MIGRATIONS, Migration};
pub use raw_setting::raw_setting;

/// Setting keys in the `setting` table.
pub mod keys {
    pub const HOTKEY: &str = "hotkey";
    pub const DECIMAL_SEPARATOR: &str = "decimal_separator";
    pub const RECENTS_N: &str = "recents_n";
    pub const BACKUP_DIR: &str = "backup_dir";
    pub const BACKUP_KEEP: &str = "backup_keep";
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("the database {0} is in use by another process")]
    Locked(PathBuf),
    /// The database holds a value the core can't represent.
    #[error("unreadable data: {0}")]
    Corrupt(String),
    /// An update or delete found no row, so the database and the in-memory state differ.
    #[error("{0} {1} is missing in the database")]
    RowMissing(&'static str, u64),
    #[error("the database has migration {0}, which this version doesn't know")]
    UnknownMigration(i64),
}

/// What [`Store::open`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenInfo {
    /// The database file existed before this open.
    pub existed: bool,
    /// Migrations not yet applied.
    pub pending_migrations: usize,
}

/// An open database. One per process and file (see [`DbLock`]).
pub struct Store {
    conn: Connection,
    path: PathBuf,
    migrations: &'static [Migration],
}

impl Store {
    /// Opens (or creates) the database without migrating it, so the caller can take a backup
    /// first (spec §2 "Startup order").
    pub fn open(
        path: &Path,
        migrations: &'static [Migration],
    ) -> Result<(Store, OpenInfo), StoreError> {
        let existed = path.exists();
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        let store = Store {
            conn,
            path: path.to_path_buf(),
            migrations,
        };
        let pending_migrations = store.pending_migrations()?.len();
        Ok((
            store,
            OpenInfo {
                existed,
                pending_migrations,
            },
        ))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
