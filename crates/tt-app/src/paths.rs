use std::path::PathBuf;

/// The app's data folder: `%LOCALAPPDATA%\ThorSoft.TimeTracker` (not roamed or synced, OQ-8).
/// `None` if `LOCALAPPDATA` isn't set.
pub fn app_data_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join("ThorSoft.TimeTracker"))
}

/// The live database of the installed app. Test drivers (tt-cli) refuse to open it.
pub fn live_db_path() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("timetracker.db"))
}
