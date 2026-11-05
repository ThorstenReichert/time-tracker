//! UI DTOs: the shapes the frontend sees, generated into `ui/src/bindings.ts` by specta.
//! They map from tt-core types, so tt-core carries no UI derives (spec §2 "Frontend").
//! Instants are UTC Unix seconds; durations are seconds; dates and local times are ISO text.

use serde::{Deserialize, Serialize};
use specta::Type;
use tt_core::read::{
    ClientBlock, ComboRow, EntryRow, Figure, NameCheck, Overview, ProjectRow, TaskMatches,
};
use tt_core::{
    Catalog, Client, DecimalSeparator, ItemRef, Project, RoundingDirection, RoundingLevel,
    RoundingRule, RoundingUnit, Settings, Task, TaskChoice, TaskId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Nearest,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Project,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuleDto {
    /// Minutes: 6, 12, 15, 30 or 60.
    pub unit: u32,
    pub direction: Direction,
    pub level: Level,
}

impl From<RoundingRule> for RuleDto {
    fn from(rule: RoundingRule) -> Self {
        Self {
            unit: rule.unit.minutes(),
            direction: match rule.direction {
                RoundingDirection::Up => Direction::Up,
                RoundingDirection::Nearest => Direction::Nearest,
                RoundingDirection::Down => Direction::Down,
            },
            level: match rule.level {
                RoundingLevel::Project => Level::Project,
                RoundingLevel::Client => Level::Client,
            },
        }
    }
}

impl RuleDto {
    /// `None` for a unit the core doesn't offer.
    pub fn to_rule(self) -> Option<RoundingRule> {
        Some(RoundingRule {
            unit: RoundingUnit::from_minutes(self.unit)?,
            direction: match self.direction {
                Direction::Up => RoundingDirection::Up,
                Direction::Nearest => RoundingDirection::Nearest,
                Direction::Down => RoundingDirection::Down,
            },
            level: match self.level {
                Level::Project => RoundingLevel::Project,
                Level::Client => RoundingLevel::Client,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientDto {
    pub id: u64,
    pub name: String,
    /// The optional short form (R24).
    pub abbreviation: Option<String>,
    pub rule: RuleDto,
    pub archived: bool,
}

impl From<&Client> for ClientDto {
    fn from(c: &Client) -> Self {
        Self {
            id: c.id.get(),
            name: c.name.to_string(),
            abbreviation: c.abbreviation.as_ref().map(ToString::to_string),
            rule: c.rule.into(),
            archived: c.archived_at.is_some(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: u64,
    pub client: u64,
    pub name: String,
    pub abbreviation: Option<String>,
    pub archived: bool,
}

impl From<&Project> for ProjectDto {
    fn from(p: &Project) -> Self {
        Self {
            id: p.id.get(),
            client: p.client.get(),
            name: p.name.to_string(),
            abbreviation: p.abbreviation.as_ref().map(ToString::to_string),
            archived: p.archived_at.is_some(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum NameCheckKind {
    Free,
    Unarchive,
    Taken,
}

/// A project named by a name check, e.g. `archived project "Web" (W)` (R2d).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CheckedProjectDto {
    pub id: u64,
    pub label: String,
}

/// The New project dialog's check (R2c). `projects` is the archived project to unarchive
/// (`unarchive`), the clashing projects (`taken`), or empty (`free`).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NameCheckDto {
    pub kind: NameCheckKind,
    pub projects: Vec<CheckedProjectDto>,
}

impl NameCheckDto {
    pub fn new(check: &NameCheck, catalog: &Catalog) -> Self {
        let (kind, ids) = match check {
            NameCheck::Free => (NameCheckKind::Free, &[][..]),
            NameCheck::Unarchive { project } => {
                (NameCheckKind::Unarchive, std::slice::from_ref(project))
            }
            NameCheck::Taken { by } => (NameCheckKind::Taken, by.as_slice()),
        };
        Self {
            kind,
            projects: ids
                .iter()
                .filter_map(|id| {
                    let label = crate::errors::item_label(catalog, ItemRef::Project(*id))?;
                    Some(CheckedProjectDto {
                        id: id.get(),
                        label,
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: u64,
    pub project: u64,
    pub name: String,
    pub abbreviation: Option<String>,
    pub archived: bool,
}

impl From<&Task> for TaskDto {
    fn from(t: &Task) -> Self {
        Self {
            id: t.id.get(),
            project: t.project.get(),
            name: t.name.to_string(),
            abbreviation: t.abbreviation.as_ref().map(ToString::to_string),
            archived: t.archived_at.is_some(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Separator {
    Auto,
    Dot,
    Comma,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    /// `None`: the app's default hotkey.
    pub hotkey: Option<String>,
    pub decimal_separator: Separator,
    pub recents_n: u32,
    /// `None`: `backups` next to the database.
    pub backup_dir: Option<String>,
    pub backup_keep: u32,
}

impl From<&Settings> for SettingsDto {
    fn from(s: &Settings) -> Self {
        Self {
            hotkey: s.hotkey.clone(),
            decimal_separator: match s.decimal_separator {
                DecimalSeparator::Auto => Separator::Auto,
                DecimalSeparator::Dot => Separator::Dot,
                DecimalSeparator::Comma => Separator::Comma,
            },
            recents_n: s.recents_n,
            backup_dir: s.backup_dir.clone(),
            backup_keep: s.backup_keep,
        }
    }
}

impl From<SettingsDto> for Settings {
    fn from(s: SettingsDto) -> Self {
        Self {
            hotkey: s.hotkey,
            decimal_separator: match s.decimal_separator {
                Separator::Auto => DecimalSeparator::Auto,
                Separator::Dot => DecimalSeparator::Dot,
                Separator::Comma => DecimalSeparator::Comma,
            },
            recents_n: s.recents_n,
            backup_dir: s.backup_dir,
            backup_keep: s.backup_keep,
        }
    }
}

/// What an unset setting falls back to, for the settings form.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDefaultsDto {
    pub hotkey: String,
    pub backup_dir: String,
}

/// The autostart entry (R9). Windows holds it; the app keeps no copy (spec §2).
#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AutostartDto {
    /// `false` in debug builds, which never touch the entry.
    pub available: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub clients: Vec<ClientDto>,
    pub projects: Vec<ProjectDto>,
    pub tasks: Vec<TaskDto>,
    pub settings: SettingsDto,
    /// The rule a new client starts with (R17), so the UI keeps no copy of the defaults.
    pub default_rule: RuleDto,
    /// The rounding units the core offers, in minutes (R17).
    pub rounding_units: Vec<u32>,
}

impl From<&Catalog> for CatalogDto {
    fn from(c: &Catalog) -> Self {
        Self {
            clients: c.clients().map(Into::into).collect(),
            projects: c.projects().map(Into::into).collect(),
            tasks: c.tasks().map(Into::into).collect(),
            settings: c.settings().into(),
            default_rule: RoundingRule::default().into(),
            rounding_units: RoundingUnit::ALL.iter().map(|u| u.minutes()).collect(),
        }
    }
}

/// The task part of a combo, as the UI sends it (R3: `new` creates the task).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum TaskChoiceDto {
    None,
    Existing(u64),
    New(String),
}

impl From<TaskChoiceDto> for TaskChoice {
    fn from(t: TaskChoiceDto) -> Self {
        match t {
            TaskChoiceDto::None => TaskChoice::NoTask,
            TaskChoiceDto::Existing(id) => TaskChoice::Existing(TaskId::new(id)),
            TaskChoiceDto::New(name) => TaskChoice::New(name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ComboDto {
    pub project: u64,
    pub task: Option<u64>,
    pub client: u64,
    pub client_name: String,
    pub client_abbreviation: Option<String>,
    pub project_name: String,
    pub project_abbreviation: Option<String>,
    pub task_name: Option<String>,
    pub task_abbreviation: Option<String>,
    /// "Client / Project / Task"
    pub label: String,
    /// The same with each level's abbreviation where it has one (R24).
    pub short_label: String,
}

impl From<&ComboRow> for ComboDto {
    fn from(r: &ComboRow) -> Self {
        Self {
            project: r.combo.project.get(),
            task: r.combo.task.map(|t| t.get()),
            client: r.client.get(),
            client_name: r.client_name.to_string(),
            client_abbreviation: text(r.client_abbreviation.as_ref()),
            project_name: r.project_name.to_string(),
            project_abbreviation: text(r.project_abbreviation.as_ref()),
            task_name: r.task_name.as_ref().map(ToString::to_string),
            task_abbreviation: text(r.task_abbreviation.as_ref()),
            label: r.label.clone(),
            short_label: r.short_label.clone(),
        }
    }
}

fn text(abbreviation: Option<&tt_core::Abbreviation>) -> Option<String> {
    abbreviation.map(ToString::to_string)
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EntryDto {
    pub id: u64,
    pub combo: ComboDto,
    /// UTC Unix seconds.
    pub start: i64,
    pub end: Option<i64>,
    /// The start and end as local wall-clock times (for editing).
    pub start_local: String,
    pub end_local: Option<String>,
    /// Seconds; a running entry counts up to now.
    pub duration: i64,
    pub running: bool,
    pub note: Option<String>,
}

impl EntryDto {
    pub fn new(r: &EntryRow, tz: &jiff::tz::TimeZone) -> Self {
        let local = |i: tt_core::Instant| tt_core::to_local(tz, i).to_string();
        Self {
            id: r.id.get(),
            combo: (&r.combo).into(),
            start: r.start.unix_seconds(),
            end: r.end.map(|e| e.unix_seconds()),
            start_local: local(r.start),
            end_local: r.end.map(local),
            duration: r.duration,
            running: r.running,
            note: r.note.as_ref().map(ToString::to_string),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FigureDto {
    /// Seconds.
    pub raw: i64,
    pub rounded: i64,
    /// `rounded − raw`.
    pub diff: i64,
}

impl From<Figure> for FigureDto {
    fn from(f: Figure) -> Self {
        Self {
            raw: f.raw,
            rounded: f.rounded,
            diff: f.diff,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRowDto {
    pub project: u64,
    pub name: String,
    pub raw: i64,
    /// `None` for client-level clients: their project rows are raw only.
    pub booked: Option<FigureDto>,
    pub running: bool,
}

impl From<&ProjectRow> for ProjectRowDto {
    fn from(p: &ProjectRow) -> Self {
        Self {
            project: p.project.get(),
            name: p.name.to_string(),
            raw: p.raw,
            booked: p.booked.map(Into::into),
            running: p.running,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClientBlockDto {
    pub client: u64,
    pub name: String,
    pub rule: RuleDto,
    pub total: FigureDto,
    pub projects: Vec<ProjectRowDto>,
    pub running: bool,
}

impl From<&ClientBlock> for ClientBlockDto {
    fn from(b: &ClientBlock) -> Self {
        Self {
            client: b.client.get(),
            name: b.name.to_string(),
            rule: b.rule.into(),
            total: b.total.into(),
            projects: b.projects.iter().map(Into::into).collect(),
            running: b.running,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OverviewDto {
    /// YYYY-MM-DD
    pub date: String,
    pub clients: Vec<ClientBlockDto>,
}

impl From<&Overview> for OverviewDto {
    fn from(o: &Overview) -> Self {
        Self {
            date: o.date.to_string(),
            clients: o.clients.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskMatchesDto {
    pub tasks: Vec<TaskDto>,
    /// Offer `Create "<name>"` when set.
    pub offer_create: Option<String>,
}

impl TaskMatchesDto {
    pub fn new(m: &TaskMatches, project: u64) -> Self {
        Self {
            tasks: m
                .tasks
                .iter()
                .map(|t| TaskDto {
                    id: t.id.get(),
                    project,
                    name: t.name.to_string(),
                    abbreviation: text(t.abbreviation.as_ref()),
                    archived: false,
                })
                .collect(),
            offer_create: m.offer_create.as_ref().map(ToString::to_string),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusDto {
    /// Why the app accepts no changes; `None` when it works.
    pub broken: Option<String>,
    pub warnings: Vec<String>,
    /// The database file in use.
    pub db_path: String,
}
