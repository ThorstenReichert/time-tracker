use std::path::PathBuf;

use thiserror::Error;
use tt_core::DomainError;
use tt_store::StoreError;

#[derive(Debug, Error)]
pub enum AppError {
    /// The command was rejected by a business rule; nothing changed.
    #[error(transparent)]
    Domain(#[from] DomainError),
    /// Writing failed. The in-memory state was reloaded from the database.
    #[error("saving failed: {0}")]
    Store(#[from] StoreError),
    /// Another process holds the database (spec §2 "Single writer").
    #[error("the database {0} is already in use by another process")]
    Locked(PathBuf),
    /// The app can't accept writes: the database couldn't be opened or loaded, or holds an
    /// invalid state.
    #[error("the app can't accept changes: {0}")]
    Broken(String),
}
