//! `decide`: the only place where business rules turn a command into events. It never
//! changes `State`; the caller evolves a copy with the events it returns.

mod catalog;
mod tracking;

pub(crate) use catalog::validate_settings;

use crate::{Command, Ctx, DomainError, Event, State};

/// Decides what `cmd` changes, given `state` and `ctx`. Same inputs, same output.
pub fn decide(state: &State, ctx: &Ctx, cmd: &Command) -> Result<Vec<Event>, DomainError> {
    let catalog = &state.catalog;
    match cmd {
        Command::CreateClient {
            name,
            abbreviation,
            rule,
        } => catalog::create_client(catalog, ctx, (name, abbreviation.as_deref()), *rule),
        Command::UpdateClient {
            id,
            name,
            abbreviation,
            rule,
        } => catalog::update_client(catalog, *id, (name, abbreviation.as_deref()), *rule),
        Command::CreateProject {
            client,
            name,
            abbreviation,
        } => catalog::create_project(catalog, ctx, *client, (name, abbreviation.as_deref())),
        Command::UpdateProject {
            id,
            name,
            abbreviation,
        } => catalog::update_project(catalog, *id, (name, abbreviation.as_deref())),
        Command::ArchiveProject { id } => catalog::archive_project(state, ctx, *id),
        Command::UnarchiveProject { id } => catalog::unarchive_project(catalog, *id),
        Command::CreateTask {
            project,
            name,
            abbreviation,
        } => catalog::create_task(catalog, ctx, *project, (name, abbreviation.as_deref())),
        Command::UpdateTask {
            id,
            name,
            abbreviation,
        } => catalog::update_task(catalog, *id, (name, abbreviation.as_deref())),
        Command::UpdateSettings { settings } => catalog::update_settings(settings),
        Command::Start { project, task } => tracking::start(state, ctx, *project, task),
        Command::Stop { at } => tracking::stop(state, ctx, *at),
        Command::AddEntry {
            project,
            task,
            start,
            end,
            note,
        } => tracking::add_entry(state, ctx, *project, task, (*start, *end), note.as_deref()),
        Command::EditEntry {
            id,
            project,
            task,
            start,
            end,
            note,
        } => tracking::edit_entry(
            state,
            ctx,
            tracking::Edit {
                id: *id,
                project: *project,
                task,
                start: *start,
                end: *end,
                note: note.as_deref(),
            },
        ),
        Command::DeleteEntry { id } => tracking::delete_entry(state, *id),
    }
}
