//! Catalog commands: clients, projects, tasks (R1, R3, R17) and settings.

use crate::error::{ItemRef, SettingProblem};
use crate::{
    Catalog, Client, ClientId, Ctx, DomainError, Event, Name, Project, ProjectId, RoundingRule,
    Settings, Task, TaskId,
};

pub(super) fn create_client(
    catalog: &Catalog,
    ctx: &Ctx,
    name: &str,
    rule: RoundingRule,
) -> Result<Vec<Event>, DomainError> {
    let name = Name::parse(name)?;
    ensure_client_name_free(catalog, &name, None)?;
    let client = Client {
        id: catalog.counters.client(),
        name,
        rule,
        archived_at: None,
        created_at: ctx.now,
    };
    Ok(vec![Event::ClientCreated { client }])
}

pub(super) fn update_client(
    catalog: &Catalog,
    id: ClientId,
    name: &str,
    rule: RoundingRule,
) -> Result<Vec<Event>, DomainError> {
    let client = catalog.client(id).ok_or(DomainError::UnknownClient(id))?;
    let name = Name::parse(name)?;
    ensure_client_name_free(catalog, &name, Some(id))?;
    if client.name == name && client.rule == rule {
        return Ok(vec![]);
    }
    Ok(vec![Event::ClientUpdated { id, name, rule }])
}

pub(super) fn create_project(
    catalog: &Catalog,
    ctx: &Ctx,
    client: ClientId,
    name: &str,
) -> Result<Vec<Event>, DomainError> {
    catalog
        .client(client)
        .ok_or(DomainError::UnknownClient(client))?;
    let name = Name::parse(name)?;
    ensure_project_name_free(catalog, client, &name, None)?;
    let project = Project {
        id: catalog.counters.project(),
        client,
        name,
        archived_at: None,
        created_at: ctx.now,
    };
    Ok(vec![Event::ProjectCreated { project }])
}

pub(super) fn rename_project(
    catalog: &Catalog,
    id: ProjectId,
    name: &str,
) -> Result<Vec<Event>, DomainError> {
    let project = catalog.project(id).ok_or(DomainError::UnknownProject(id))?;
    let name = Name::parse(name)?;
    ensure_project_name_free(catalog, project.client, &name, Some(id))?;
    if project.name == name {
        return Ok(vec![]);
    }
    Ok(vec![Event::ProjectRenamed { id, name }])
}

pub(super) fn create_task(
    catalog: &Catalog,
    ctx: &Ctx,
    project: ProjectId,
    name: &str,
) -> Result<Vec<Event>, DomainError> {
    let task = new_task(catalog, ctx, project, name)?;
    Ok(vec![Event::TaskCreated { task }])
}

/// A new task for `project`, with the next task ID. Also used when an entry command creates
/// its task on the fly (R3).
pub(crate) fn new_task(
    catalog: &Catalog,
    ctx: &Ctx,
    project: ProjectId,
    name: &str,
) -> Result<Task, DomainError> {
    catalog
        .project(project)
        .ok_or(DomainError::UnknownProject(project))?;
    let name = Name::parse(name)?;
    ensure_task_name_free(catalog, project, &name, None)?;
    Ok(Task {
        id: catalog.counters.task(),
        project,
        name,
        archived_at: None,
        created_at: ctx.now,
    })
}

pub(super) fn rename_task(
    catalog: &Catalog,
    id: TaskId,
    name: &str,
) -> Result<Vec<Event>, DomainError> {
    let task = catalog.task(id).ok_or(DomainError::UnknownTask(id))?;
    let name = Name::parse(name)?;
    ensure_task_name_free(catalog, task.project, &name, Some(id))?;
    if task.name == name {
        return Ok(vec![]);
    }
    Ok(vec![Event::TaskRenamed { id, name }])
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

// Uniqueness (R1): `except` is the item being renamed, which may keep its own name in any case.

fn ensure_client_name_free(
    catalog: &Catalog,
    name: &Name,
    except: Option<ClientId>,
) -> Result<(), DomainError> {
    match catalog.client_named(name) {
        Some(c) if Some(c.id) != except => Err(DomainError::DuplicateName {
            existing: ItemRef::Client(c.id),
        }),
        _ => Ok(()),
    }
}

fn ensure_project_name_free(
    catalog: &Catalog,
    client: ClientId,
    name: &Name,
    except: Option<ProjectId>,
) -> Result<(), DomainError> {
    match catalog.project_named(client, name) {
        Some(p) if Some(p.id) != except => Err(DomainError::DuplicateName {
            existing: ItemRef::Project(p.id),
        }),
        _ => Ok(()),
    }
}

fn ensure_task_name_free(
    catalog: &Catalog,
    project: ProjectId,
    name: &Name,
    except: Option<TaskId>,
) -> Result<(), DomainError> {
    match catalog.task_named(project, name) {
        Some(t) if Some(t.id) != except => Err(DomainError::DuplicateName {
            existing: ItemRef::Task(t.id),
        }),
        _ => Ok(()),
    }
}
