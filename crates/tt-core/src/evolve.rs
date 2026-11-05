//! `evolve`: applies one event to the state. It holds no rules; `decide` already checked
//! them. In debug builds and tests every result is checked against the invariants.

use crate::{Event, State};

/// Applies `event` to `state`. Panics if the event doesn't fit the state, which means a bug
/// in `decide` (crash-only: nothing has been written yet, see plan.md "Panics").
pub fn evolve(mut state: State, event: &Event) -> State {
    apply(&mut state, event);
    if cfg!(debug_assertions)
        && let Err(violation) = state.check_invariants()
    {
        panic!("{violation} after {event:?}");
    }
    state
}

/// Applies `events` in order.
pub fn evolve_all<'a>(state: State, events: impl IntoIterator<Item = &'a Event>) -> State {
    events.into_iter().fold(state, evolve)
}

fn apply(state: &mut State, event: &Event) {
    let entries = &mut state.entries;
    let catalog = &mut state.catalog;
    let counters = &mut catalog.counters;
    match event {
        Event::ClientCreated { client } => {
            counters.next_client = counters.next_client.max(client.id.get() + 1);
            catalog.clients.insert(client.id, client.clone());
        }
        Event::ClientUpdated {
            id,
            name,
            abbreviation,
            rule,
        } => {
            let client = catalog.clients.get_mut(id).expect("updated client exists");
            client.name = name.clone();
            client.abbreviation = abbreviation.clone();
            client.rule = *rule;
        }
        Event::ClientArchived { id, at } => {
            let client = catalog.clients.get_mut(id).expect("archived client exists");
            client.archived_at = Some(*at);
        }
        Event::ClientUnarchived { id } => {
            let client = catalog
                .clients
                .get_mut(id)
                .expect("unarchived client exists");
            client.archived_at = None;
        }
        Event::ProjectCreated { project } => {
            counters.next_project = counters.next_project.max(project.id.get() + 1);
            catalog.projects.insert(project.id, project.clone());
        }
        Event::ProjectUpdated {
            id,
            name,
            abbreviation,
        } => {
            let project = catalog
                .projects
                .get_mut(id)
                .expect("updated project exists");
            project.name = name.clone();
            project.abbreviation = abbreviation.clone();
        }
        Event::ProjectArchived { id, at } => {
            let project = catalog
                .projects
                .get_mut(id)
                .expect("archived project exists");
            project.archived_at = Some(*at);
        }
        Event::ProjectUnarchived { id } => {
            let project = catalog
                .projects
                .get_mut(id)
                .expect("unarchived project exists");
            project.archived_at = None;
        }
        Event::TaskCreated { task } => {
            counters.next_task = counters.next_task.max(task.id.get() + 1);
            catalog.tasks.insert(task.id, task.clone());
        }
        Event::TaskUpdated {
            id,
            name,
            abbreviation,
        } => {
            let task = catalog.tasks.get_mut(id).expect("updated task exists");
            task.name = name.clone();
            task.abbreviation = abbreviation.clone();
        }
        Event::SettingsChanged { settings } => {
            catalog.settings = settings.clone();
        }
        Event::EntryStarted { entry } | Event::EntryAdded { entry } => {
            counters.next_entry = counters.next_entry.max(entry.id.get() + 1);
            entries.insert(entry.clone());
        }
        Event::EntryStopped {
            id,
            end,
            updated_at,
        } => entries.stop(*id, *end, *updated_at),
        Event::EntryUpdated { entry } => entries.replace(entry.clone()),
        Event::EntryDeleted { id } => {
            entries.remove(*id);
        }
    }
}
