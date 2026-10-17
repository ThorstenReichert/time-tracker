use thiserror::Error;

use crate::{ClientId, ProjectId, TaskId};

/// Why a command was rejected. Every invalid save is rejected with one of these, never
/// adjusted silently (R12).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("invalid name: {0}")]
    InvalidName(NameProblem),
    /// Names are unique within their parent, ignoring case; archived items count too (OQ-7).
    #[error("the name is already used by {existing}")]
    DuplicateName { existing: ItemRef },
    #[error("unknown client {0}")]
    UnknownClient(ClientId),
    #[error("unknown project {0}")]
    UnknownProject(ProjectId),
    #[error("unknown task {0}")]
    UnknownTask(TaskId),
    #[error("invalid setting: {0}")]
    InvalidSetting(SettingProblem),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NameProblem {
    #[error("it is empty")]
    Empty,
    #[error("it is longer than {max} characters")]
    TooLong { max: usize },
    #[error("it contains a control character")]
    ControlCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SettingProblem {
    #[error("the hotkey is empty")]
    EmptyHotkey,
    #[error("the backup folder is empty")]
    EmptyBackupDir,
    #[error("the number of recents must be 1 to {max}")]
    RecentsOutOfRange { max: u32 },
    #[error("the number of backups to keep must be 1 to {max}")]
    BackupKeepOutOfRange { max: u32 },
}

/// Points at an existing client, project or task, e.g. the one a new name clashes with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemRef {
    Client(ClientId),
    Project(ProjectId),
    Task(TaskId),
}

impl std::fmt::Display for ItemRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Client(id) => write!(f, "client {id}"),
            Self::Project(id) => write!(f, "project {id}"),
            Self::Task(id) => write!(f, "task {id}"),
        }
    }
}

/// A broken invariant in a loaded or evolved `State`. After a load this means the database
/// holds data the core would never have produced; the app then accepts no writes.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invariant violated: {0}")]
pub struct InvariantViolation(pub String);
