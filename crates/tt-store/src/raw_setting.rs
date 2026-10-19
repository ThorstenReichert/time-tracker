use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension};

use crate::StoreError;

/// Reads one setting directly from the database file, before it is opened for real and
/// migrated (the startup backup needs the backup folder, spec §2 "Startup order"). A missing
/// file, table or key gives `None`.
pub fn raw_setting(db_path: &Path, key: &str) -> Result<Option<String>, StoreError> {
    if !db_path.exists() {
        return Ok(None);
    }
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let has_table: Option<String> = conn
        .query_row(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = 'setting'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if has_table.is_none() {
        return Ok(None);
    }
    Ok(conn
        .query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?)
}
