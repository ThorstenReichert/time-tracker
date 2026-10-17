//! Startup order (spec §2): take the exclusive lock → back up if a database exists → run
//! migrations → load State and check its invariants. The backup must come before the
//! migrations, or a bad migration has no protection.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tt_core::{Ctx, Settings};
use tt_store::{DbLock, Store, StoreError, keys, raw_setting};

use crate::{App, AppConfig, AppError, Inner};

pub(crate) fn open(config: AppConfig, ctx: &Ctx) -> Result<App, AppError> {
    let lock = DbLock::acquire(&config.db_path).map_err(|e| match e {
        StoreError::Locked(path) => AppError::Locked(path),
        other => AppError::Store(other),
    })?;
    let mut warnings = Vec::new();
    let inner = match ready(&config, ctx, &mut warnings) {
        Ok((state, store)) => Inner::Ready { state, store },
        Err(reason) => Inner::Broken { reason },
    };
    Ok(App {
        inner: Mutex::new(inner),
        warnings,
        subscribers: Mutex::new(Vec::new()),
        _lock: lock,
    })
}

fn ready(
    config: &AppConfig,
    ctx: &Ctx,
    warnings: &mut Vec<String>,
) -> Result<(tt_core::State, Store), String> {
    let db = &config.db_path;
    // Read before the database is opened for real: the backup comes before migrations.
    let backup = BackupPlan::read(config).map_err(|e| format!("reading the settings: {e}"))?;
    let (mut store, info) =
        Store::open(db, config.migrations).map_err(|e| format!("opening {}: {e}", db.display()))?;

    if info.existed
        && let Err(e) = store.backup_to(&backup.dir, ctx.now, backup.keep)
    {
        let message = format!("backup to {} failed: {e}", backup.dir.display());
        if info.pending_migrations > 0 {
            // Never migrate without a backup.
            return Err(format!("{message}; not migrating without a backup"));
        }
        warnings.push(message);
    }
    store.migrate().map_err(|e| format!("migrating: {e}"))?;
    let state = store.load().map_err(|e| format!("loading: {e}"))?;
    state
        .check_invariants()
        .map_err(|v| format!("the database holds invalid data: {v}"))?;
    Ok((state, store))
}

struct BackupPlan {
    dir: PathBuf,
    keep: usize,
}

impl BackupPlan {
    /// The override, else the `backup_dir` setting, else `<db dir>/backups` (R22). The
    /// defaults come from `tt_core::Settings::default()`, so there is only one set.
    fn read(config: &AppConfig) -> Result<BackupPlan, StoreError> {
        let db = &config.db_path;
        let defaults = Settings::default();
        let dir = match &config.backup_dir_override {
            Some(dir) => dir.clone(),
            None => raw_setting(db, keys::BACKUP_DIR)?
                .map(PathBuf::from)
                .or_else(|| defaults.backup_dir.as_ref().map(PathBuf::from))
                .unwrap_or_else(|| default_backup_dir(db)),
        };
        let keep = raw_setting(db, keys::BACKUP_KEEP)?
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(defaults.backup_keep);
        Ok(BackupPlan {
            dir,
            keep: keep as usize,
        })
    }
}

fn default_backup_dir(db: &Path) -> PathBuf {
    db.parent().unwrap_or(Path::new(".")).join("backups")
}
