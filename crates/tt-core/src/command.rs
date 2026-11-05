//! What a driver asks for. Commands are plain data; names arrive as raw text and are parsed
//! by `decide`, so an invalid name is a `DomainError`, not a deserialisation failure.

use jiff::civil;
use serde::{Deserialize, Serialize};

use crate::{ClientId, EntryId, ProjectId, RoundingRule, Settings, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    /// Abbreviations are optional everywhere: `None` or blank text means none (R24).
    CreateClient {
        name: String,
        abbreviation: Option<String>,
        rule: RoundingRule,
    },
    /// Renames a client, sets its abbreviation and/or changes its rounding rule (R17, R19,
    /// R24).
    UpdateClient {
        id: ClientId,
        name: String,
        abbreviation: Option<String>,
        rule: RoundingRule,
    },
    /// Archives a client: its projects are inactive through it and its entries stay (R2g).
    /// The projects' own archive state is untouched. Rejected while the timer runs on one of
    /// its projects. Already archived: nothing happens.
    ArchiveClient {
        id: ClientId,
    },
    /// Makes an archived client active again (R2g, R2i). Its projects come back as they were:
    /// those archived on their own stay archived. Active: nothing happens.
    UnarchiveClient {
        id: ClientId,
    },
    CreateProject {
        client: ClientId,
        name: String,
        abbreviation: Option<String>,
    },
    /// Renames a project and/or sets its abbreviation (R24).
    UpdateProject {
        id: ProjectId,
        name: String,
        abbreviation: Option<String>,
    },
    /// Archives a project: it is no longer offered or bookable, its entries stay (R2a). The
    /// running entry's project can't be archived. Already archived: nothing happens.
    ArchiveProject {
        id: ProjectId,
    },
    /// Makes an archived project active again, as it was (R2a, R2c). Active: nothing happens.
    UnarchiveProject {
        id: ProjectId,
    },
    CreateTask {
        project: ProjectId,
        name: String,
        abbreviation: Option<String>,
    },
    /// Renames a task and/or sets its abbreviation (R24).
    UpdateTask {
        id: TaskId,
        name: String,
        abbreviation: Option<String>,
    },
    UpdateSettings {
        settings: Settings,
    },
    /// Starts a timer on the combo at `now`. If another combo is running, it is stopped at
    /// the same instant (a switch, R5); if the same combo is running, nothing happens.
    Start {
        project: ProjectId,
        task: TaskChoice,
    },
    /// Stops the running entry at `now`, or at a typed earlier time (forgotten stop, R11).
    Stop {
        at: Option<civil::DateTime>,
    },
    /// Adds a finished entry after the fact (R10). Times are local wall-clock times.
    AddEntry {
        project: ProjectId,
        task: TaskChoice,
        start: civil::DateTime,
        end: civil::DateTime,
        note: Option<String>,
    },
    /// Replaces an entry's combo, times and note (R10, R11). On the running entry,
    /// `end: None` keeps it running; on a finished entry an end is required.
    EditEntry {
        id: EntryId,
        project: ProjectId,
        task: TaskChoice,
        start: civil::DateTime,
        end: Option<civil::DateTime>,
        note: Option<String>,
    },
    DeleteEntry {
        id: EntryId,
    },
}

/// The task part of a combo. `New` creates the task in the same command (R3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskChoice {
    NoTask,
    Existing(TaskId),
    New(String),
}
