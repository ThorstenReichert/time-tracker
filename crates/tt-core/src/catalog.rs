//! The small part of `State` that is always fully loaded (OQ-9): clients, projects, tasks,
//! settings and the ID counters.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ClientId, EntryId, Instant, Name, ProjectId, RoundingRule, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: Name,
    pub rule: RoundingRule,
    pub archived_at: Option<Instant>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub client: ClientId,
    pub name: Name,
    pub archived_at: Option<Instant>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub project: ProjectId,
    pub name: Name,
    pub archived_at: Option<Instant>,
    pub created_at: Instant,
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

    /// The client with this name, ignoring case (archived ones included, OQ-7).
    pub fn client_named(&self, name: &Name) -> Option<&Client> {
        self.clients.values().find(|c| c.name.matches(name))
    }

    /// The project of `client` with this name, ignoring case (archived ones included).
    pub fn project_named(&self, client: ClientId, name: &Name) -> Option<&Project> {
        self.projects_of(client).find(|p| p.name.matches(name))
    }

    /// The task of `project` with this name, ignoring case (archived ones included).
    pub fn task_named(&self, project: ProjectId, name: &Name) -> Option<&Task> {
        self.tasks_of(project).find(|t| t.name.matches(name))
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn counters(&self) -> Counters {
        self.counters
    }
}
