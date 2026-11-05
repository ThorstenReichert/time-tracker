use serde::{Deserialize, Serialize};

use crate::{Abbreviation, Catalog, ClientId, Combo, Entries, Name};

/// A combo with its names, for lists and matching. `label` is "Client / Project / Task"
/// (or "Client / Project" without a task). `short_label` has the same shape with each
/// level's abbreviation where it has one (R24).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboRow {
    pub combo: Combo,
    pub client: ClientId,
    pub client_name: Name,
    pub client_abbreviation: Option<Abbreviation>,
    pub project_name: Name,
    pub project_abbreviation: Option<Abbreviation>,
    pub task_name: Option<Name>,
    pub task_abbreviation: Option<Abbreviation>,
    pub label: String,
    pub short_label: String,
}

impl ComboRow {
    /// The lowercase keys of the abbreviations of every level that has one (R24).
    pub(crate) fn abbreviation_keys(&self) -> Vec<&str> {
        [
            &self.client_abbreviation,
            &self.project_abbreviation,
            &self.task_abbreviation,
        ]
        .into_iter()
        .flatten()
        .map(Abbreviation::key)
        .collect()
    }

    /// What typed text is matched against (R3, R6, R24): the names and abbreviations of
    /// every level, so a query can mix them ("nwi calibration").
    pub(crate) fn match_text(&self) -> String {
        let level = |name: &Name, abbreviation: Option<&Abbreviation>| match abbreviation {
            Some(a) => format!("{name} {a}"),
            None => name.to_string(),
        };
        let mut text = format!(
            "{} / {}",
            level(&self.client_name, self.client_abbreviation.as_ref()),
            level(&self.project_name, self.project_abbreviation.as_ref())
        );
        if let Some(task) = &self.task_name {
            text.push_str(" / ");
            text.push_str(&level(task, self.task_abbreviation.as_ref()));
        }
        text
    }
}

/// The names for `combo`, or `None` if an item doesn't exist.
pub fn combo_row(catalog: &Catalog, combo: Combo) -> Option<ComboRow> {
    let project = catalog.project(combo.project)?;
    let client = catalog.client(project.client)?;
    let task = match combo.task {
        Some(id) => Some(catalog.task(id).filter(|t| t.project == project.id)?),
        None => None,
    };
    let mut label = format!("{} / {}", client.name, project.name);
    let mut short_label = format!("{} / {}", client.short_name(), project.short_name());
    if let Some(task) = task {
        label.push_str(" / ");
        label.push_str(task.name.as_str());
        short_label.push_str(" / ");
        short_label.push_str(task.short_name());
    }
    Some(ComboRow {
        combo,
        client: client.id,
        client_name: client.name.clone(),
        client_abbreviation: client.abbreviation.clone(),
        project_name: project.name.clone(),
        project_abbreviation: project.abbreviation.clone(),
        task_name: task.map(|t| t.name.clone()),
        task_abbreviation: task.and_then(|t| t.abbreviation.clone()),
        label,
        short_label,
    })
}

/// Whether the combo can be used for a new entry: it exists and nothing in it is archived.
pub(crate) fn is_active(catalog: &Catalog, combo: Combo) -> bool {
    catalog.is_project_active(combo.project) && combo.task.is_none_or(|t| catalog.is_task_active(t))
}

/// The newest `n` distinct active combos, newest first: the read set of the recents (R7)
/// and of the quick-switch ranking. Archived combos don't use up any of the `n`.
pub fn recent_combos(catalog: &Catalog, entries: &Entries, n: usize) -> Vec<Combo> {
    entries.recent_combos_where(n, |c| is_active(catalog, c))
}

/// The recents (R7): the newest `n` distinct active combos with their names.
pub fn recents(catalog: &Catalog, entries: &Entries, n: usize) -> Vec<ComboRow> {
    rows(catalog, &recent_combos(catalog, entries, n))
}

/// The rows of `combos` that are active, in order.
pub(crate) fn rows(catalog: &Catalog, combos: &[Combo]) -> Vec<ComboRow> {
    combos
        .iter()
        .filter(|c| is_active(catalog, **c))
        .filter_map(|c| combo_row(catalog, *c))
        .collect()
}
