use std::path::{Path, PathBuf};

/// The app's data folder: `%LOCALAPPDATA%\ThorSoft.TimeTracker` (not roamed or synced, OQ-8).
/// `None` if `LOCALAPPDATA` isn't set.
pub fn app_data_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join("ThorSoft.TimeTracker"))
}

/// The live database of the installed app. Test drivers (tt-cli) refuse to open it.
pub fn live_db_path() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("timetracker.db"))
}

/// The backup folder when the settings name none: `backups` next to the database (R22).
pub fn default_backup_dir(db: &Path) -> PathBuf {
    db.parent().unwrap_or(Path::new(".")).join("backups")
}
