//! `decide`: the only place where business rules turn a command into events. It never
//! changes `State`; the caller evolves a copy with the events it returns.

mod catalog;

pub(crate) use catalog::validate_settings;

use crate::{Command, Ctx, DomainError, Event, State};

/// Decides what `cmd` changes, given `state` and `ctx`. Same inputs, same output.
pub fn decide(state: &State, ctx: &Ctx, cmd: &Command) -> Result<Vec<Event>, DomainError> {
    let catalog = &state.catalog;
    match cmd {
        Command::CreateClient { name, rule } => catalog::create_client(catalog, ctx, name, *rule),
        Command::UpdateClient { id, name, rule } => {
            catalog::update_client(catalog, *id, name, *rule)
        }
        Command::CreateProject { client, name } => {
            catalog::create_project(catalog, ctx, *client, name)
        }
        Command::RenameProject { id, name } => catalog::rename_project(catalog, *id, name),
        Command::CreateTask { project, name } => catalog::create_task(catalog, ctx, *project, name),
        Command::RenameTask { id, name } => catalog::rename_task(catalog, *id, name),
        Command::UpdateSettings { settings } => catalog::update_settings(settings),
    }
}
