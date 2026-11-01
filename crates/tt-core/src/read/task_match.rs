//! Quick task creation (R3): match a project's active tasks (names and abbreviations, R24) as
//! you type, and offer `Create "<text>"` when no task is called exactly that. A task whose
//! abbreviation is exactly a typed word ranks first.

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};

use super::quick_switch::{exact_abbreviations, query_words};
use crate::{Abbreviation, Catalog, Name, ProjectId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRow {
    pub id: TaskId,
    pub name: Name,
    pub abbreviation: Option<Abbreviation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskMatches {
    /// Matching active tasks, best first (all of them, by name, for blank text).
    pub tasks: Vec<TaskRow>,
    /// The name to offer as `Create "<name>"`: set when the text is a valid name and no task
    /// of the project has it as its name or abbreviation, ignoring case (R24). Archived
    /// tasks count too (OQ-7); offering to unarchive one comes with archiving (R2, slice 2).
    pub offer_create: Option<Name>,
}

pub fn task_matches(catalog: &Catalog, project: ProjectId, text: &str) -> TaskMatches {
    let active = catalog
        .tasks_of(project)
        .filter(|t| catalog.is_task_active(t.id));
    let words = query_words(text);
    let mut tasks: Vec<(usize, u32, &str, TaskRow)> = if text.trim().is_empty() {
        active.map(|t| (0, 0, t.name.key(), row(t))).collect()
    } else {
        let pattern = Pattern::new(
            text,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );
        let mut matcher = Matcher::new(Config::DEFAULT);
        let mut buf = Vec::new();
        active
            .filter_map(|t| {
                let text = match &t.abbreviation {
                    Some(a) => format!("{} {a}", t.name),
                    None => t.name.to_string(),
                };
                let score = pattern.score(Utf32Str::new(&text, &mut buf), &mut matcher)?;
                let keys: Vec<&str> = t.abbreviation.iter().map(|a| a.key()).collect();
                Some((
                    exact_abbreviations(&words, &keys),
                    score,
                    t.name.key(),
                    row(t),
                ))
            })
            .collect()
    };
    tasks.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(b.1.cmp(&a.1))
            .then(a.2.cmp(b.2))
            .then(a.3.id.cmp(&b.3.id))
    });

    let offer_create = Name::parse(text)
        .ok()
        .filter(|name| catalog.task_called(project, name.key()).is_none());
    TaskMatches {
        tasks: tasks.into_iter().map(|(_, _, _, row)| row).collect(),
        offer_create,
    }
}

fn row(task: &crate::Task) -> TaskRow {
    TaskRow {
        id: task.id,
        name: task.name.clone(),
        abbreviation: task.abbreviation.clone(),
    }
}
