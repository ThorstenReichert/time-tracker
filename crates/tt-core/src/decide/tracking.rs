//! Tracking commands: start/stop/switch (R4, R5), entries after the fact and edits (R10,
//! R11), validation (R12) and notes (R13).

use jiff::civil;

use super::catalog::new_task;
use crate::{
    Catalog, Combo, Ctx, DomainError, Entries, Entry, EntryId, Event, Instant, Note, ProjectId,
    State, TaskChoice, to_instant,
};

pub(super) fn start(
    state: &State,
    ctx: &Ctx,
    project: ProjectId,
    task: &TaskChoice,
) -> Result<Vec<Event>, DomainError> {
    let resolved = resolve_combo(&state.catalog, ctx, project, task)?;
    let now = ctx.now;
    let mut events = Vec::new();

    let running = state.entries.running();
    if let Some(running) = running {
        if resolved.new_task.is_none() && running.combo() == resolved.combo {
            return Ok(events); // already running on this combo: nothing to do
        }
        events.push(end_running_now(running, now)?);
    }

    // The new entry runs from `now`; nothing else may still be open at or after `now`.
    ensure_no_overlap(&state.entries, now, None, running.map(|r| r.id))?;

    let combo = resolved.combo;
    if let Some(task) = resolved.new_task {
        events.push(Event::TaskCreated { task });
    }
    events.push(Event::EntryStarted {
        entry: Entry {
            id: state.catalog.counters.entry(),
            project: combo.project,
            task: combo.task,
            start: now,
            end: None,
            note: None,
            created_at: now,
            updated_at: now,
        },
    });
    Ok(events)
}

pub(super) fn stop(
    state: &State,
    ctx: &Ctx,
    at: Option<civil::DateTime>,
) -> Result<Vec<Event>, DomainError> {
    let running = state.entries.running().ok_or(DomainError::NotRunning)?;
    match at {
        None => Ok(vec![end_running_now(running, ctx.now)?]),
        // A typed time is never adjusted: it must lie after the start and not in the future.
        Some(local) => {
            let end = to_instant(&ctx.tz, local)?;
            if end > ctx.now {
                return Err(DomainError::EndInFuture);
            }
            if end <= running.start {
                return Err(DomainError::EndNotAfterStart);
            }
            Ok(vec![Event::EntryStopped {
                id: running.id,
                end,
                updated_at: ctx.now,
            }])
        }
    }
}

pub(super) fn add_entry(
    state: &State,
    ctx: &Ctx,
    project: ProjectId,
    task: &TaskChoice,
    (start, end): (civil::DateTime, civil::DateTime),
    note: Option<&str>,
) -> Result<Vec<Event>, DomainError> {
    let resolved = resolve_combo(&state.catalog, ctx, project, task)?;
    let (start, end) = (to_instant(&ctx.tz, start)?, to_instant(&ctx.tz, end)?);
    validate_interval(ctx.now, start, Some(end))?;
    let note = parse_note(note)?;
    ensure_no_overlap(&state.entries, start, Some(end), None)?;

    let mut events = Vec::new();
    if let Some(task) = resolved.new_task {
        events.push(Event::TaskCreated { task });
    }
    events.push(Event::EntryAdded {
        entry: Entry {
            id: state.catalog.counters.entry(),
            project: resolved.combo.project,
            task: resolved.combo.task,
            start,
            end: Some(end),
            note,
            created_at: ctx.now,
            updated_at: ctx.now,
        },
    });
    Ok(events)
}

pub(super) struct Edit<'a> {
    pub id: EntryId,
    pub project: ProjectId,
    pub task: &'a TaskChoice,
    pub start: civil::DateTime,
    pub end: Option<civil::DateTime>,
    pub note: Option<&'a str>,
}

pub(super) fn edit_entry(
    state: &State,
    ctx: &Ctx,
    edit: Edit<'_>,
) -> Result<Vec<Event>, DomainError> {
    let current = state
        .entries
        .get(edit.id)
        .ok_or(DomainError::UnknownEntry(edit.id))?;
    let resolved = resolve_combo(&state.catalog, ctx, edit.project, edit.task)?;
    let start = to_instant(&ctx.tz, edit.start)?;
    let end = edit.end.map(|e| to_instant(&ctx.tz, e)).transpose()?;
    // Only the running entry may stay open; giving it an end stops it (R11).
    if end.is_none() && !current.is_running() {
        return Err(DomainError::EndRequired);
    }
    validate_interval(ctx.now, start, end)?;
    let note = parse_note(edit.note)?;
    ensure_no_overlap(&state.entries, start, end, Some(edit.id))?;

    let combo = resolved.combo;
    let unchanged = resolved.new_task.is_none()
        && current.combo() == combo
        && (current.start, current.end) == (start, end)
        && current.note == note;
    if unchanged {
        return Ok(vec![]);
    }
    let mut events = Vec::new();
    if let Some(task) = resolved.new_task {
        events.push(Event::TaskCreated { task });
    }
    events.push(Event::EntryUpdated {
        entry: Entry {
            id: edit.id,
            project: combo.project,
            task: combo.task,
            start,
            end,
            note,
            created_at: current.created_at,
            updated_at: ctx.now,
        },
    });
    Ok(events)
}

pub(super) fn delete_entry(state: &State, id: EntryId) -> Result<Vec<Event>, DomainError> {
    state.entries.get(id).ok_or(DomainError::UnknownEntry(id))?;
    Ok(vec![Event::EntryDeleted { id }])
}

/// A combo checked against the catalog. `new_task` is set when the command creates the task.
struct Resolved {
    combo: Combo,
    new_task: Option<crate::Task>,
}

fn resolve_combo(
    catalog: &Catalog,
    ctx: &Ctx,
    project: ProjectId,
    task: &TaskChoice,
) -> Result<Resolved, DomainError> {
    catalog
        .project(project)
        .ok_or(DomainError::UnknownProject(project))?;
    let (task_id, new_task) = match task {
        TaskChoice::NoTask => (None, None),
        TaskChoice::Existing(id) => {
            let task = catalog.task(*id).ok_or(DomainError::UnknownTask(*id))?;
            if task.project != project {
                return Err(DomainError::TaskNotInProject { task: *id, project });
            }
            (Some(*id), None)
        }
        TaskChoice::New(name) => {
            let task = new_task(catalog, ctx, project, (name, None))?;
            (Some(task.id), Some(task))
        }
    };
    Ok(Resolved {
        combo: Combo {
            project,
            task: task_id,
        },
        new_task,
    })
}

/// Ends the running entry at `now`. An entry started in this very second is discarded
/// instead of being saved with zero length (plan.md "Zero-length running entry").
fn end_running_now(running: &Entry, now: Instant) -> Result<Event, DomainError> {
    if now < running.start {
        // The clock went back: stopping now would put the end before the start.
        return Err(DomainError::EndNotAfterStart);
    }
    if now == running.start {
        return Ok(Event::EntryDeleted { id: running.id });
    }
    Ok(Event::EntryStopped {
        id: running.id,
        end: now,
        updated_at: now,
    })
}

/// R12: no start or end in the future, and the end after the start.
fn validate_interval(
    now: Instant,
    start: Instant,
    end: Option<Instant>,
) -> Result<(), DomainError> {
    if start > now {
        return Err(DomainError::StartInFuture);
    }
    if let Some(end) = end {
        if end > now {
            return Err(DomainError::EndInFuture);
        }
        if end <= start {
            return Err(DomainError::EndNotAfterStart);
        }
    }
    Ok(())
}

/// R12: the interval may not overlap another entry; the error names the earliest conflict.
fn ensure_no_overlap(
    entries: &Entries,
    start: Instant,
    end: Option<Instant>,
    except: Option<EntryId>,
) -> Result<(), DomainError> {
    match entries.overlapping(start, end, except).first() {
        Some(other) => Err(DomainError::Overlap {
            with: other.id,
            start: other.start,
            end: other.end,
        }),
        None => Ok(()),
    }
}

fn parse_note(note: Option<&str>) -> Result<Option<Note>, DomainError> {
    note.map(Note::parse).transpose().map(Option::flatten)
}
