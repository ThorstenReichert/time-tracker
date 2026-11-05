//! What changed. Events are self-contained: each carries every column it writes, so the store
//! applies it without reading `State` (OQ-9). Counters follow from the IDs in the events.

use serde::{Deserialize, Serialize};

use crate::{
    Abbreviation, Client, ClientId, Entry, EntryId, Instant, Name, Project, ProjectId,
    RoundingRule, Settings, Task, TaskId,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    ClientCreated {
        client: Client,
    },
    ClientUpdated {
        id: ClientId,
        name: Name,
        abbreviation: Option<Abbreviation>,
        rule: RoundingRule,
    },
    /// The client got `archived_at = at` (R2g).
    ClientArchived {
        id: ClientId,
        at: Instant,
    },
    /// The client's `archived_at` was cleared (R2g).
    ClientUnarchived {
        id: ClientId,
    },
    ProjectCreated {
        project: Project,
    },
    ProjectUpdated {
        id: ProjectId,
        name: Name,
        abbreviation: Option<Abbreviation>,
    },
    /// The project got `archived_at = at` (R2a).
    ProjectArchived {
        id: ProjectId,
        at: Instant,
    },
    /// The project's `archived_at` was cleared (R2a).
    ProjectUnarchived {
        id: ProjectId,
    },
    TaskCreated {
        task: Task,
    },
    TaskUpdated {
        id: TaskId,
        name: Name,
        abbreviation: Option<Abbreviation>,
    },
    SettingsChanged {
        settings: Settings,
    },
    /// A timer started (an insert of a running entry).
    EntryStarted {
        entry: Entry,
    },
    /// The running entry got its end.
    EntryStopped {
        id: EntryId,
        end: Instant,
        updated_at: Instant,
    },
    /// A finished entry was added after the fact.
    EntryAdded {
        entry: Entry,
    },
    /// An entry was replaced (combo, times, note).
    EntryUpdated {
        entry: Entry,
    },
    EntryDeleted {
        id: EntryId,
    },
}
