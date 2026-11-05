use thiserror::Error;

use crate::{ClientId, EntryId, Instant, ProjectId, TaskId};

/// Why a command was rejected. Every invalid save is rejected with one of these, never
/// adjusted silently (R12).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("invalid name: {0}")]
    InvalidName(NameProblem),
    #[error("invalid abbreviation: {0}")]
    InvalidAbbreviation(NameProblem),
    /// Names and abbreviations share one namespace per parent, ignoring case; archived items
    /// count too (R1, R24, OQ-7, R2c). The new name clashes with `existing`.
    #[error("the name is already used by {existing}")]
    DuplicateName { existing: ItemRef },
    /// The new abbreviation clashes with the name or abbreviation of `existing` (R24).
    #[error("the abbreviation is already used by {existing}")]
    DuplicateAbbreviation { existing: ItemRef },
    /// A new entry, a changed combo or a new task would land on an inactive item (R2b). `item`
    /// is the archived one: the client, else the project, else the task.
    #[error("{item} is archived")]
    Archived { item: ItemRef },
    /// The running entry is on this project; stop the timer before archiving it (R2a).
    #[error("the timer is running on project {project}; stop it first")]
    ProjectRunning { project: ProjectId },
    /// The running entry is on a project of this client; stop the timer before archiving the
    /// client (R2g).
    #[error("the timer is running on a project of client {client}; stop it first")]
    ClientRunning { client: ClientId },
    #[error("unknown client {0}")]
    UnknownClient(ClientId),
    #[error("unknown project {0}")]
    UnknownProject(ProjectId),
    #[error("unknown task {0}")]
    UnknownTask(TaskId),
    #[error("unknown entry {0}")]
    UnknownEntry(EntryId),
    #[error("invalid setting: {0}")]
    InvalidSetting(SettingProblem),
    #[error("task {task} doesn't belong to project {project}")]
    TaskNotInProject { task: TaskId, project: ProjectId },
    #[error("the note is longer than {max} characters")]
    NoteTooLong { max: usize },
    #[error("no timer is running")]
    NotRunning,
    #[error("an entry can't start in the future")]
    StartInFuture,
    #[error("an entry can't end in the future")]
    EndInFuture,
    #[error("the end must be after the start")]
    EndNotAfterStart,
    #[error("a finished entry needs an end")]
    EndRequired,
    /// The typed time falls in the repeated hour at the end of DST (OQ-6).
    #[error("this local time exists twice (daylight saving time ends); pick another")]
    AmbiguousLocalTime,
    /// The typed time falls in the skipped hour at the start of DST (OQ-6).
    #[error("this local time doesn't exist (daylight saving time starts); pick another")]
    NonexistentLocalTime,
    #[error("times are recorded to the second")]
    SubsecondTime,
    #[error("the time is out of the supported range")]
    TimeOutOfRange,
    /// The new interval overlaps an existing entry (R12). `end: None` means it's running.
    #[error("overlaps entry {with}")]
    Overlap {
        with: EntryId,
        start: Instant,
        end: Option<Instant>,
    },
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
    #[error("the backup folder must be a full path, like C:\\Users\\Me\\OneDrive\\TimeTracker")]
    RelativeBackupDir,
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

impl ItemRef {
    /// The raw ID, whatever the kind of item.
    pub const fn id(self) -> u64 {
        match self {
            Self::Client(id) => id.get(),
            Self::Project(id) => id.get(),
            Self::Task(id) => id.get(),
        }
    }
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
