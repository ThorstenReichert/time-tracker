//! The invariants the core guarantees (R1, R2, R4, R12). They run after every `evolve` in
//! debug builds and tests, and on every load in all builds.

use std::collections::BTreeSet;

use crate::decide::validate_settings;
use crate::{InvariantViolation, State};

impl State {
    pub fn check_invariants(&self) -> Result<(), InvariantViolation> {
        check_catalog(self)
    }
}

fn violation(message: impl Into<String>) -> Result<(), InvariantViolation> {
    Err(InvariantViolation(message.into()))
}

fn check_catalog(state: &State) -> Result<(), InvariantViolation> {
    let c = &state.catalog;

    // Map keys match the IDs, and the counters are ahead of every ID (IDs are never reused).
    for (id, client) in &c.clients {
        if *id != client.id {
            return violation(format!("client stored under {id} has id {}", client.id));
        }
        if client.id.get() >= c.counters.next_client {
            return violation(format!("client {id} is not below the client counter"));
        }
    }
    for (id, project) in &c.projects {
        if *id != project.id {
            return violation(format!("project stored under {id} has id {}", project.id));
        }
        if project.id.get() >= c.counters.next_project {
            return violation(format!("project {id} is not below the project counter"));
        }
        if c.client(project.client).is_none() {
            return violation(format!(
                "project {id} refers to missing client {}",
                project.client
            ));
        }
    }
    for (id, task) in &c.tasks {
        if *id != task.id {
            return violation(format!("task stored under {id} has id {}", task.id));
        }
        if task.id.get() >= c.counters.next_task {
            return violation(format!("task {id} is not below the task counter"));
        }
        if c.project(task.project).is_none() {
            return violation(format!(
                "task {id} refers to missing project {}",
                task.project
            ));
        }
    }

    // Names are unique within their parent, ignoring case (R1).
    let mut seen = BTreeSet::new();
    for client in c.clients() {
        if !seen.insert(client.name.key()) {
            return violation(format!("duplicate client name {:?}", client.name.as_str()));
        }
    }
    let mut seen = BTreeSet::new();
    for project in c.projects() {
        if !seen.insert((project.client, project.name.key())) {
            return violation(format!(
                "duplicate project name {:?} in client {}",
                project.name.as_str(),
                project.client
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for task in c.tasks() {
        if !seen.insert((task.project, task.name.key())) {
            return violation(format!(
                "duplicate task name {:?} in project {}",
                task.name.as_str(),
                task.project
            ));
        }
    }

    if let Err(problem) = validate_settings(&c.settings) {
        return violation(format!("settings: {problem}"));
    }
    Ok(())
}
