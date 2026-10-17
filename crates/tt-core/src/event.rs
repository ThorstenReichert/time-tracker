//! What changed. Events are self-contained: each carries every column it writes, so the store
//! applies it without reading `State` (OQ-9). Counters follow from the IDs in the events.

use serde::{Deserialize, Serialize};

use crate::{
    Client, ClientId, Entry, EntryId, Instant, Name, Project, ProjectId, RoundingRule, Settings,
    Task, TaskId,
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
        rule: RoundingRule,
    },
    ProjectCreated {
        project: Project,
    },
    ProjectRenamed {
        id: ProjectId,
        name: Name,
    },
    TaskCreated {
        task: Task,
    },
    TaskRenamed {
        id: TaskId,
        name: Name,
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
