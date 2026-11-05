//! The small part of `State` that is always fully loaded (OQ-9): clients, projects, tasks,
//! settings and the ID counters.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::ItemRef;
use crate::{Abbreviation, ClientId, EntryId, Instant, Name, ProjectId, RoundingRule, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: Name,
    /// The short form for compact places (R24).
    #[serde(default)]
    pub abbreviation: Option<Abbreviation>,
    pub rule: RoundingRule,
    pub archived_at: Option<Instant>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub client: ClientId,
    pub name: Name,
    #[serde(default)]
    pub abbreviation: Option<Abbreviation>,
    pub archived_at: Option<Instant>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub project: ProjectId,
    pub name: Name,
    #[serde(default)]
    pub abbreviation: Option<Abbreviation>,
    pub archived_at: Option<Instant>,
    pub created_at: Instant,
}

/// Whether an item with `name` and `abbreviation` is called `key` (a lowercase name or
/// abbreviation key). Names and abbreviations share one namespace per parent (R1, R24).
fn is_called(name: &Name, abbreviation: Option<&Abbreviation>, key: &str) -> bool {
    name.key() == key || abbreviation.is_some_and(|a| a.key() == key)
}

impl Client {
    /// Called `key` by its name or its abbreviation, ignoring case.
    pub fn is_called(&self, key: &str) -> bool {
        is_called(&self.name, self.abbreviation.as_ref(), key)
    }

    /// The abbreviation, or else the name (R24).
    pub fn short_name(&self) -> &str {
        short_name(&self.name, self.abbreviation.as_ref())
    }
}

impl Project {
    pub fn is_called(&self, key: &str) -> bool {
        is_called(&self.name, self.abbreviation.as_ref(), key)
    }

    pub fn short_name(&self) -> &str {
        short_name(&self.name, self.abbreviation.as_ref())
    }
}

impl Task {
    pub fn is_called(&self, key: &str) -> bool {
        is_called(&self.name, self.abbreviation.as_ref(), key)
    }

    pub fn short_name(&self) -> &str {
        short_name(&self.name, self.abbreviation.as_ref())
    }
}

fn short_name<'a>(name: &'a Name, abbreviation: Option<&'a Abbreviation>) -> &'a str {
    abbreviation.map_or(name.as_str(), Abbreviation::as_str)
}

/// How hours are formatted for copying (R15). `Auto` follows the OS locale in the UI.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum DecimalSeparator {
    #[default]
    Auto,
    Dot,
    Comma,
}

impl DecimalSeparator {
    /// The text stored in the database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dot => "dot",
            Self::Comma => "comma",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Auto, Self::Dot, Self::Comma]
            .into_iter()
            .find(|s| s.as_str() == text)
    }
}

/// User settings (spec §2 `setting` table). Autostart isn't here: the OS entry is its only
/// source of truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The global quick-switch hotkey (R6). `None` means the driver's default.
    pub hotkey: Option<String>,
    pub decimal_separator: DecimalSeparator,
    /// How many recent combos to offer (R7).
    pub recents_n: u32,
    /// Backup folder (R22). `None` means `<db dir>/backups`.
    pub backup_dir: Option<String>,
    /// How many backups to keep (R22).
    pub backup_keep: u32,
}

impl Settings {
    pub const RECENTS_N_MAX: u32 = 50;
    pub const BACKUP_KEEP_MAX: u32 = 1000;
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: None,
            decimal_separator: DecimalSeparator::Auto,
            recents_n: 8,
            backup_dir: None,
            backup_keep: 14,
        }
    }
}

/// The next ID of each kind. IDs are never reused, even after the newest row was deleted,
/// because the counters are stored with the rows (spec §2 "IDs").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counters {
    pub next_client: u64,
    pub next_project: u64,
    pub next_task: u64,
    pub next_entry: u64,
}

impl Default for Counters {
    fn default() -> Self {
        Self {
            next_client: 1,
            next_project: 1,
            next_task: 1,
            next_entry: 1,
        }
    }
}

impl Counters {
    pub(crate) fn client(&self) -> ClientId {
        ClientId::new(self.next_client)
    }

    pub(crate) fn project(&self) -> ProjectId {
        ProjectId::new(self.next_project)
    }

    pub(crate) fn task(&self) -> TaskId {
        TaskId::new(self.next_task)
    }

    pub(crate) fn entry(&self) -> EntryId {
        EntryId::new(self.next_entry)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub(crate) clients: BTreeMap<ClientId, Client>,
    pub(crate) projects: BTreeMap<ProjectId, Project>,
    pub(crate) tasks: BTreeMap<TaskId, Task>,
    pub(crate) settings: Settings,
    pub(crate) counters: Counters,
}

impl Catalog {
    /// Builds a catalog from loaded rows. It isn't validated here; the loader runs
    /// `State::check_invariants` on the whole state.
    pub fn from_parts(
        clients: impl IntoIterator<Item = Client>,
        projects: impl IntoIterator<Item = Project>,
        tasks: impl IntoIterator<Item = Task>,
        settings: Settings,
        counters: Counters,
    ) -> Self {
        Self {
            clients: clients.into_iter().map(|c| (c.id, c)).collect(),
            projects: projects.into_iter().map(|p| (p.id, p)).collect(),
            tasks: tasks.into_iter().map(|t| (t.id, t)).collect(),
            settings,
            counters,
        }
    }

    pub fn client(&self, id: ClientId) -> Option<&Client> {
        self.clients.get(&id)
    }

    pub fn project(&self, id: ProjectId) -> Option<&Project> {
        self.projects.get(&id)
    }

    pub fn task(&self, id: TaskId) -> Option<&Task> {
        self.tasks.get(&id)
    }

    /// All clients, ordered by ID.
    pub fn clients(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }

    /// All projects, ordered by ID.
    pub fn projects(&self) -> impl Iterator<Item = &Project> {
        self.projects.values()
    }

    /// All tasks, ordered by ID.
    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values()
    }

    pub fn projects_of(&self, client: ClientId) -> impl Iterator<Item = &Project> {
        self.projects.values().filter(move |p| p.client == client)
    }

    pub fn tasks_of(&self, project: ProjectId) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(move |t| t.project == project)
    }

    /// Active means: the item and all its parents are not archived (R2). Only active items
    /// are offered for new entries (quick-switch, recents, pickers).
    pub fn is_client_active(&self, id: ClientId) -> bool {
        self.client(id).is_some_and(|c| c.archived_at.is_none())
    }

    pub fn is_project_active(&self, id: ProjectId) -> bool {
        self.project(id)
            .is_some_and(|p| p.archived_at.is_none() && self.is_client_active(p.client))
    }

    pub fn is_task_active(&self, id: TaskId) -> bool {
        self.task(id)
            .is_some_and(|t| t.archived_at.is_none() && self.is_project_active(t.project))
    }

    /// The archived item that makes this combo inactive, looked up from the top: the client,
    /// then the project, then the task (R2b). `None` if the combo is active, and also for an
    /// unknown project or task: callers check that they exist first.
    pub fn archived_in(&self, project: ProjectId, task: Option<TaskId>) -> Option<ItemRef> {
        let p = self.project(project)?;
        if self
            .client(p.client)
            .is_some_and(|c| c.archived_at.is_some())
        {
            return Some(ItemRef::Client(p.client));
        }
        if p.archived_at.is_some() {
            return Some(ItemRef::Project(project));
        }
        task.and_then(|t| self.task(t))
            .filter(|t| t.archived_at.is_some())
            .map(|t| ItemRef::Task(t.id))
    }

    /// The client whose name or abbreviation has this key, ignoring case (archived ones
    /// included, OQ-7). Names and abbreviations share one namespace (R1, R24).
    pub fn client_called(&self, key: &str) -> Option<&Client> {
        self.clients.values().find(|c| c.is_called(key))
    }

    /// The project of `client` whose name or abbreviation has this key (archived included).
    pub fn project_called(&self, client: ClientId, key: &str) -> Option<&Project> {
        self.projects_of(client).find(|p| p.is_called(key))
    }

    /// The task of `project` whose name or abbreviation has this key (archived included).
    pub fn task_called(&self, project: ProjectId, key: &str) -> Option<&Task> {
        self.tasks_of(project).find(|t| t.is_called(key))
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn counters(&self) -> Counters {
        self.counters
    }
}
