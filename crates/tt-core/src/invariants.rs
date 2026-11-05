//! The invariants the core guarantees (R1, R2a, R4, R12, R24). They run after every `evolve` in
//! debug builds and tests, and on every load in all builds.

use std::collections::BTreeSet;

use crate::decide::validate_settings;
use crate::{Abbreviation, InvariantViolation, Name, State};

impl State {
    pub fn check_invariants(&self) -> Result<(), InvariantViolation> {
        check_catalog(self)?;
        check_entries(self)
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

    // Names and abbreviations share one namespace per parent, ignoring case (R1, R24). An
    // item's own name and abbreviation may be equal.
    let mut seen = BTreeSet::new();
    for client in c.clients() {
        for key in keys(&client.name, client.abbreviation.as_ref()) {
            if !seen.insert(key) {
                return violation(format!("duplicate client name or abbreviation {key:?}"));
            }
        }
    }
    let mut seen = BTreeSet::new();
    for project in c.projects() {
        for key in keys(&project.name, project.abbreviation.as_ref()) {
            if !seen.insert((project.client, key)) {
                return violation(format!(
                    "duplicate project name or abbreviation {key:?} in client {}",
                    project.client
                ));
            }
        }
    }
    let mut seen = BTreeSet::new();
    for task in c.tasks() {
        for key in keys(&task.name, task.abbreviation.as_ref()) {
            if !seen.insert((task.project, key)) {
                return violation(format!(
                    "duplicate task name or abbreviation {key:?} in project {}",
                    task.project
                ));
            }
        }
    }

    if let Err(problem) = validate_settings(&c.settings) {
        return violation(format!("settings: {problem}"));
    }
    Ok(())
}

fn check_entries(state: &State) -> Result<(), InvariantViolation> {
    let (c, entries) = (&state.catalog, &state.entries);
    if !entries.indexes_consistent() {
        return violation("entry indexes disagree with the entries");
    }
    // R4: at most one running entry.
    if entries.running_count() > 1 {
        return violation(format!("{} running entries", entries.running_count()));
    }
    // R2a: the running entry's combo is active (archiving its project is rejected).
    if let Some(running) = entries.running()
        && let Some(item) = c.archived_in(running.project, running.task)
    {
        return violation(format!(
            "the running entry {} is on archived {item}",
            running.id
        ));
    }
    let mut previous: Option<&crate::Entry> = None;
    for entry in entries.all() {
        let id = entry.id;
        if id.get() >= c.counters.next_entry {
            return violation(format!("entry {id} is not below the entry counter"));
        }
        if c.project(entry.project).is_none() {
            return violation(format!(
                "entry {id} refers to missing project {}",
                entry.project
            ));
        }
        if let Some(task) = entry.task {
            match c.task(task) {
                None => return violation(format!("entry {id} refers to missing task {task}")),
                Some(t) if t.project != entry.project => {
                    return violation(format!(
                        "entry {id}: task {task} belongs to project {}, not {}",
                        t.project, entry.project
                    ));
                }
                Some(_) => {}
            }
        }
        // R12: the end is after the start.
        if entry.end.is_some_and(|end| end <= entry.start) {
            return violation(format!("entry {id} doesn't end after its start"));
        }
        // R12: no overlaps. Ordered by start, each entry must end (a running one never
        // does) at or before the next one starts.
        if let Some(prev) = previous {
            match prev.end {
                None => {
                    return violation(format!(
                        "entry {id} starts after the running entry {}",
                        prev.id
                    ));
                }
                Some(end) if end > entry.start => {
                    return violation(format!("entries {} and {id} overlap", prev.id));
                }
                Some(_) => {}
            }
        }
        previous = Some(entry);
    }
    Ok(())
}

/// The keys an item is called by: its name, and its abbreviation if that differs.
fn keys<'a>(name: &'a Name, abbreviation: Option<&'a Abbreviation>) -> Vec<&'a str> {
    let mut keys = vec![name.key()];
    if let Some(a) = abbreviation.filter(|a| a.key() != name.key()) {
        keys.push(a.key());
    }
    keys
}
