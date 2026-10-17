//! The tt-app instance in Tauri's managed state, the database location and `Ctx`.

use std::path::PathBuf;

use jiff::{Timestamp, tz::TimeZone};
use tt_app::{App, AppConfig};
use tt_core::{Ctx, Instant};

use crate::errors::UiError;

/// Builds `Ctx` from the system clock and the OS time zone. The only place the GUI reads
/// them; tt-app and tt-core get them from here.
pub fn ctx() -> Ctx {
    Ctx::new(Instant::from(Timestamp::now()), TimeZone::system())
}

/// Release builds use the live database (OQ-8). Debug builds use a separate one, so
/// `cargo tauri dev` never touches live data; `TT_DB` points a debug build at another file
/// (e.g. a tt-cli scenario database).
pub fn db_path() -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        if let Some(path) = std::env::var_os("TT_DB") {
            return Ok(PathBuf::from(path));
        }
        return tt_app::app_data_dir()
            .map(|dir| dir.join("dev").join("timetracker.db"))
            .ok_or_else(|| "LOCALAPPDATA is not set".to_owned());
    }
    tt_app::live_db_path().ok_or_else(|| "LOCALAPPDATA is not set".to_owned())
}

pub struct Backend {
    app: Result<App, String>,
    db_path: String,
    /// The database didn't exist before this start.
    pub created: bool,
}

impl Backend {
    pub fn open(ctx: &Ctx) -> Backend {
        let path = match db_path() {
            Ok(path) => path,
            Err(reason) => {
                return Backend {
                    app: Err(reason),
                    db_path: String::new(),
                    created: false,
                };
            }
        };
        let created = !path.exists();
        let app = App::open(AppConfig::new(&path), ctx).map_err(|e| e.to_string());
        Backend {
            app,
            db_path: path.display().to_string(),
            created,
        }
    }

    pub fn app(&self) -> Result<&App, UiError> {
        self.app
            .as_ref()
            .map_err(|reason| UiError::broken(reason.clone()))
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }
}
