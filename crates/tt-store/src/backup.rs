//! Rolling backups (R22) with SQLite's online backup API.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, MAIN_DB};
use tt_core::Instant;

use crate::{Store, StoreError};

/// Backup files are named `timetracker-YYYYMMDDTHHMMSSZ.db` (UTC), so sorting by name sorts
/// by time, and pruning never touches other files.
pub const BACKUP_PREFIX: &str = "timetracker-";
const BACKUP_SUFFIX: &str = ".db";

pub fn backup_file_name(now: Instant) -> String {
    let stamp = now.to_timestamp().strftime("%Y%m%dT%H%M%SZ");
    format!("{BACKUP_PREFIX}{stamp}{BACKUP_SUFFIX}")
}

fn is_backup_file_name(name: &str) -> bool {
    let Some(stamp) = name
        .strip_prefix(BACKUP_PREFIX)
        .and_then(|rest| rest.strip_suffix(BACKUP_SUFFIX))
    else {
        return false;
    };
    let b = stamp.as_bytes();
    b.len() == 16
        && b[8] == b'T'
        && b[15] == b'Z'
        && b[..8].iter().chain(&b[9..15]).all(u8::is_ascii_digit)
}

impl Store {
    /// Writes a consistent copy of the database to `dir` and keeps the newest `keep` backup
    /// files there. A backup with the same name (same second) is replaced. The copy is
    /// written to a temporary file first, so a failed backup never leaves a partial file
    /// under a backup name.
    pub fn backup_to(&self, dir: &Path, now: Instant, keep: usize) -> Result<PathBuf, StoreError> {
        std::fs::create_dir_all(dir)?;
        let target = dir.join(backup_file_name(now));
        let partial = target.with_extension("db.partial");
        let _ = std::fs::remove_file(&partial);
        self.conn.backup(MAIN_DB, &partial, None)?;
        // A single-file copy: no -wal/-shm files next to it in a (possibly synced) folder.
        Connection::open(&partial)?.pragma_update(None, "journal_mode", "DELETE")?;
        std::fs::rename(&partial, &target)?;
        prune(dir, keep)?;
        Ok(target)
    }
}

/// Deletes all but the newest `keep` backup files in `dir`.
fn prune(dir: &Path, keep: usize) -> Result<(), StoreError> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_str().is_some_and(is_backup_file_name))
        .map(|e| e.path())
        .collect();
    backups.sort();
    let excess = backups.len().saturating_sub(keep);
    for old in &backups[..excess] {
        std::fs::remove_file(old)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_sort_by_time_and_are_recognised() {
        let name = backup_file_name(Instant::from_unix_seconds(1_792_224_000).unwrap());
        assert_eq!(name, "timetracker-20261017T080000Z.db");
        assert!(is_backup_file_name(&name));
        assert!(!is_backup_file_name("timetracker.db"));
        assert!(!is_backup_file_name("timetracker-2026.db"));
        assert!(!is_backup_file_name(
            "timetracker-20261017T080000Z.db.partial"
        ));
    }
}
