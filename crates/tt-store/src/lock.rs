use std::fs::{File, TryLockError};
use std::path::{Path, PathBuf};

use crate::StoreError;

/// The single-writer lock (spec §2 "Single writer"): an exclusive OS lock on `<db>.lock`,
/// held as long as this value lives. Taken before the database is touched at all.
#[derive(Debug)]
pub struct DbLock {
    _file: File,
    path: PathBuf,
}

impl DbLock {
    pub fn acquire(db_path: &Path) -> Result<DbLock, StoreError> {
        let path = lock_path(db_path);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)?;
        match file.try_lock() {
            Ok(()) => Ok(DbLock { _file: file, path }),
            Err(TryLockError::WouldBlock) => Err(StoreError::Locked(db_path.to_path_buf())),
            Err(TryLockError::Error(e)) => Err(e.into()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn lock_path(db_path: &Path) -> PathBuf {
    let mut name = db_path.as_os_str().to_owned();
    name.push(".lock");
    PathBuf::from(name)
}
