//! Catalog commands: clients, projects, tasks (R1, R2a, R3, R17, R24) and settings.

use crate::error::{ItemRef, SettingProblem};
use crate::{
    Abbreviation, Catalog, Client, ClientId, Ctx, DomainError, Event, Name, Project, ProjectId,
    RoundingRule, Settings, State, Task, TaskId,
};

/// A name and an optional abbreviation as typed (R24).
pub(super) type Typed<'a> = (&'a str, Option<&'a str>);

/// A parsed name and abbreviation.
struct Naming {
    name: Name,
    abbreviation: Option<Abbreviation>,
}

impl Naming {
    fn parse((name, abbreviation): Typed<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            name: Name::parse(name)?,
            abbreviation: Abbreviation::parse_optional(abbreviation)?,
        })
    }
}

pub(super) fn create_client(
    catalog: &Catalog,
    ctx: &Ctx,
    typed: Typed<'_>,
    rule: RoundingRule,
) -> Result<Vec<Event>, DomainError> {
    let Naming { name, abbreviation } = Naming::parse(typed)?;
    ensure_free(&name, abbreviation.as_ref(), |key| {
        catalog.client_called(key).map(|c| ItemRef::Client(c.id))
    })?;
    let client = Client {
        id: catalog.counters.client(),
        name,
        abbreviation,
        rule,
        archived_at: None,
        created_at: ctx.now,
    };
    Ok(vec![Event::ClientCreated { client }])
}

pub(super) fn update_client(
    catalog: &Catalog,
    id: ClientId,
    typed: Typed<'_>,
    rule: RoundingRule,
) -> Result<Vec<Event>, DomainError> {
    let client = catalog.client(id).ok_or(DomainError::UnknownClient(id))?;
    let Naming { name, abbreviation } = Naming::parse(typed)?;
    ensure_free(&name, abbreviation.as_ref(), |key| {
        catalog
            .clients()
            .find(|c| c.id != id && c.is_called(key))
            .map(|c| ItemRef::Client(c.id))
    })?;
    if client.name == name && client.abbreviation == abbreviation && client.rule == rule {
        return Ok(vec![]);
    }
    Ok(vec![Event::ClientUpdated {
        id,
        name,
        abbreviation,
        rule,
    }])
}

pub(super) fn create_project(
    catalog: &Catalog,
    ctx: &Ctx,
    client: ClientId,
    typed: Typed<'_>,
) -> Result<Vec<Event>, DomainError> {
    catalog
        .client(client)
        .ok_or(DomainError::UnknownClient(client))?;
    let Naming { name, abbreviation } = Naming::parse(typed)?;
    ensure_free(&name, abbreviation.as_ref(), |key| {
        catalog
            .project_called(client, key)
            .map(|p| ItemRef::Project(p.id))
    })?;
    let project = Project {
        id: catalog.counters.project(),
        client,
        name,
        abbreviation,
        archived_at: None,
        created_at: ctx.now,
    };
    Ok(vec![Event::ProjectCreated { project }])
}

pub(super) fn update_project(
    catalog: &Catalog,
    id: ProjectId,
    typed: Typed<'_>,
) -> Result<Vec<Event>, DomainError> {
    let project = catalog.project(id).ok_or(DomainError::UnknownProject(id))?;
    let Naming { name, abbreviation } = Naming::parse(typed)?;
    ensure_free(&name, abbreviation.as_ref(), |key| {
        catalog
            .projects_of(project.client)
            .find(|p| p.id != id && p.is_called(key))
            .map(|p| ItemRef::Project(p.id))
    })?;
    if project.name == name && project.abbreviation == abbreviation {
        return Ok(vec![]);
    }
    Ok(vec![Event::ProjectUpdated {
        id,
        name,
        abbreviation,
    }])
}

pub(super) fn archive_project(
    state: &State,
    ctx: &Ctx,
    id: ProjectId,
) -> Result<Vec<Event>, DomainError> {
    let project = state
        .catalog
        .project(id)
        .ok_or(DomainError::UnknownProject(id))?;
    if project.archived_at.is_some() {
        return Ok(vec![]);
    }
    if state.entries.running().is_some_and(|e| e.project == id) {
        return Err(DomainError::ProjectRunning { project: id });
    }
    Ok(vec![Event::ProjectArchived { id, at: ctx.now }])
}

pub(super) fn unarchive_project(
    catalog: &Catalog,
    id: ProjectId,
) -> Result<Vec<Event>, DomainError> {
    let project = catalog.project(id).ok_or(DomainError::UnknownProject(id))?;
    if project.archived_at.is_none() {
        return Ok(vec![]);
    }
    // Archived names keep counting (OQ-7), so the project can't clash with a sibling now.
    Ok(vec![Event::ProjectUnarchived { id }])
}

pub(super) fn create_task(
    catalog: &Catalog,
    ctx: &Ctx,
    project: ProjectId,
    typed: Typed<'_>,
) -> Result<Vec<Event>, DomainError> {
    let task = new_task(catalog, ctx, project, typed)?;
    Ok(vec![Event::TaskCreated { task }])
}

/// A new task for `project`, with the next task ID. Also used when an entry command creates
/// its task on the fly (R3), without an abbreviation. An inactive project gets no new tasks
/// (R2b).
pub(crate) fn new_task(
    catalog: &Catalog,
    ctx: &Ctx,
    project: ProjectId,
    typed: Typed<'_>,
) -> Result<Task, DomainError> {
    catalog
        .project(project)
        .ok_or(DomainError::UnknownProject(project))?;
    if let Some(item) = catalog.archived_in(project, None) {
        return Err(DomainError::Archived { item });
    }
    let Naming { name, abbreviation } = Naming::parse(typed)?;
    ensure_free(&name, abbreviation.as_ref(), |key| {
        catalog
            .task_called(project, key)
            .map(|t| ItemRef::Task(t.id))
    })?;
    Ok(Task {
        id: catalog.counters.task(),
        project,
        name,
        abbreviation,
        archived_at: None,
        created_at: ctx.now,
    })
}

pub(super) fn update_task(
    catalog: &Catalog,
    id: TaskId,
    typed: Typed<'_>,
) -> Result<Vec<Event>, DomainError> {
    let task = catalog.task(id).ok_or(DomainError::UnknownTask(id))?;
    let Naming { name, abbreviation } = Naming::parse(typed)?;
    ensure_free(&name, abbreviation.as_ref(), |key| {
        catalog
            .tasks_of(task.project)
            .find(|t| t.id != id && t.is_called(key))
            .map(|t| ItemRef::Task(t.id))
    })?;
    if task.name == name && task.abbreviation == abbreviation {
        return Ok(vec![]);
    }
    Ok(vec![Event::TaskUpdated {
        id,
        name,
        abbreviation,
    }])
}

pub(super) fn update_settings(settings: &Settings) -> Result<Vec<Event>, DomainError> {
    validate_settings(settings).map_err(DomainError::InvalidSetting)?;
    Ok(vec![Event::SettingsChanged {
        settings: settings.clone(),
    }])
}

pub(crate) fn validate_settings(settings: &Settings) -> Result<(), SettingProblem> {
    if settings
        .hotkey
        .as_deref()
        .is_some_and(|h| h.trim().is_empty())
    {
        return Err(SettingProblem::EmptyHotkey);
    }
    if settings
        .backup_dir
        .as_deref()
        .is_some_and(|d| d.trim().is_empty())
    {
        return Err(SettingProblem::EmptyBackupDir);
    }
    // A relative folder would depend on the process's working directory, which differs
    // between a Start menu launch and autostart (R22).
    if settings
        .backup_dir
        .as_deref()
        .is_some_and(|d| !std::path::Path::new(d).is_absolute())
    {
        return Err(SettingProblem::RelativeBackupDir);
    }
    if !(1..=Settings::RECENTS_N_MAX).contains(&settings.recents_n) {
        return Err(SettingProblem::RecentsOutOfRange {
            max: Settings::RECENTS_N_MAX,
        });
    }
    if !(1..=Settings::BACKUP_KEEP_MAX).contains(&settings.backup_keep) {
        return Err(SettingProblem::BackupKeepOutOfRange {
            max: Settings::BACKUP_KEEP_MAX,
        });
    }
    Ok(())
}

// Uniqueness (R1, R24): names and abbreviations share one namespace per parent, ignoring
// case, archived items included (OQ-7). `sibling_called` finds another item of the same parent
// whose name or abbreviation has the key; the item being updated is not its own sibling, so it
// may keep its name in any case, or use its name as its abbreviation.
fn ensure_free(
    name: &Name,
    abbreviation: Option<&Abbreviation>,
    sibling_called: impl Fn(&str) -> Option<ItemRef>,
) -> Result<(), DomainError> {
    if let Some(existing) = sibling_called(name.key()) {
        return Err(DomainError::DuplicateName { existing });
    }
    if let Some(existing) = abbreviation.and_then(|a| sibling_called(a.key())) {
        return Err(DomainError::DuplicateAbbreviation { existing });
    }
    Ok(())
}
