//! Quick-switch matching (R6): fuzzy filtering over the active "Client / Project / Task"
//! combos, ranked by match score, then recency, then label. Ranking is tuned during use
//! (OQ-2).

use std::collections::{BTreeMap, BTreeSet};

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use super::combo::is_active;
use super::{ComboRow, combo_row};
use crate::{Catalog, Combo};

/// At most this many matches are returned.
pub const QUICK_SWITCH_LIMIT: usize = 50;

/// The popup's and the pickers' list for `query`. A blank query gives the recents, then the
/// other active combos by label; otherwise every active combo (each task, plus the project
/// without a task) whose label matches all words of the query, best first. `recents` is
/// newest first.
pub fn quick_switch(catalog: &Catalog, recents: &[Combo], query: &str) -> Vec<ComboRow> {
    if query.trim().is_empty() {
        return blank(catalog, recents);
    }
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut matcher = Matcher::new(Config::DEFAULT);
    let recency: BTreeMap<Combo, usize> =
        recents.iter().enumerate().map(|(i, c)| (*c, i)).collect();

    let mut buf = Vec::new();
    let mut scored: Vec<(u32, usize, String, ComboRow)> = candidates(catalog)
        .filter_map(|row| {
            let score = pattern.score(Utf32Str::new(&row.label, &mut buf), &mut matcher)?;
            let rank = recency.get(&row.combo).copied().unwrap_or(usize::MAX);
            Some((score, rank, row.label.to_lowercase(), row))
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
            .then(a.3.combo.cmp(&b.3.combo))
    });
    scored
        .into_iter()
        .take(QUICK_SWITCH_LIMIT)
        .map(|(_, _, _, row)| row)
        .collect()
}

/// The recents first, then the combos never or not recently used, so a new catalog is
/// listed before anything is tracked.
fn blank(catalog: &Catalog, recents: &[Combo]) -> Vec<ComboRow> {
    let mut rows = super::recents(catalog, recents, QUICK_SWITCH_LIMIT);
    let listed: BTreeSet<Combo> = rows.iter().map(|r| r.combo).collect();
    let mut rest: Vec<(String, ComboRow)> = candidates(catalog)
        .filter(|row| !listed.contains(&row.combo))
        .map(|row| (row.label.to_lowercase(), row))
        .collect();
    rest.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.combo.cmp(&b.1.combo)));
    let room = QUICK_SWITCH_LIMIT.saturating_sub(rows.len());
    rows.extend(rest.into_iter().take(room).map(|(_, row)| row));
    rows
}

/// All active combos: per active project, the project alone and each of its active tasks.
fn candidates(catalog: &Catalog) -> impl Iterator<Item = ComboRow> + '_ {
    catalog
        .projects()
        .flat_map(move |p| {
            std::iter::once(Combo {
                project: p.id,
                task: None,
            })
            .chain(catalog.tasks_of(p.id).map(move |t| Combo {
                project: p.id,
                task: Some(t.id),
            }))
        })
        .filter(move |c| is_active(catalog, *c))
        .filter_map(move |c| combo_row(catalog, c))
}
