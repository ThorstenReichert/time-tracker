use serde::{Deserialize, Serialize};

use crate::{Catalog, ClientId, Combo, Name};

/// A combo with its names, for lists and matching. `label` is "Client / Project / Task"
/// (or "Client / Project" without a task).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboRow {
    pub combo: Combo,
    pub client: ClientId,
    pub client_name: Name,
    pub project_name: Name,
    pub task_name: Option<Name>,
    pub label: String,
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
    if let Some(task) = task {
        label.push_str(" / ");
        label.push_str(task.name.as_str());
    }
    Some(ComboRow {
        combo,
        client: client.id,
        client_name: client.name.clone(),
        project_name: project.name.clone(),
        task_name: task.map(|t| t.name.clone()),
        label,
    })
}

/// Whether the combo can be used for a new entry: it exists and nothing in it is archived.
pub(crate) fn is_active(catalog: &Catalog, combo: Combo) -> bool {
    catalog.is_project_active(combo.project) && combo.task.is_none_or(|t| catalog.is_task_active(t))
}

/// The recents (R7): the newest distinct combos, active ones only, at most `n`. `combos` is
/// `Entries::recent_combos`, newest first; ask it for more than `n` if some may be inactive.
pub fn recents(catalog: &Catalog, combos: &[Combo], n: usize) -> Vec<ComboRow> {
    combos
        .iter()
        .filter(|c| is_active(catalog, **c))
        .filter_map(|c| combo_row(catalog, *c))
        .take(n)
        .collect()
}
