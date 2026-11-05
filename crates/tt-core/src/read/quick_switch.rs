//! Quick-switch matching (R6): fuzzy filtering over the active "Client / Project / Task"
//! combos, names and abbreviations alike (R24), ranked by the query words that are exact
//! abbreviations, then match score, then recency, then label. Ranking is tuned during use
//! (OQ-2).

use std::collections::{BTreeMap, BTreeSet};

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use super::combo::is_active;
use super::{ComboRow, combo_row};
use crate::{Catalog, Combo};

/// At most this many matches are returned.
pub const QUICK_SWITCH_LIMIT: usize = 50;

/// How many recent active combos rank the matches (`recents` of [`quick_switch`]).
pub const QUICK_SWITCH_RECENCY: usize = 100;

/// The popup's and the pickers' list for `query`. A blank query gives the recents, then the
/// other active combos by label; otherwise every active combo (each task, plus the project
/// without a task) whose names and abbreviations match all words of the query, best first.
/// A combo with more query words that are exactly one of its abbreviations ranks first
/// (R24). `recents` is [`super::recent_combos`] for [`QUICK_SWITCH_RECENCY`], newest first:
/// all of it ranks typed matches, and its first `recents_n` are the recents of a blank query.
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
    let words = query_words(query);

    let mut buf = Vec::new();
    let mut scored: Vec<(usize, u32, usize, String, ComboRow)> = candidates(catalog)
        .filter_map(|row| {
            let text = row.match_text();
            let score = pattern.score(Utf32Str::new(&text, &mut buf), &mut matcher)?;
            let exact = exact_abbreviations(&words, &row.abbreviation_keys());
            let rank = recency.get(&row.combo).copied().unwrap_or(usize::MAX);
            Some((exact, score, rank, row.label.to_lowercase(), row))
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(b.1.cmp(&a.1))
            .then(a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
            .then(a.4.combo.cmp(&b.4.combo))
    });
    scored
        .into_iter()
        .take(QUICK_SWITCH_LIMIT)
        .map(|(_, _, _, _, row)| row)
        .collect()
}

/// The distinct words of a query, lowercased like the abbreviation keys.
pub(crate) fn query_words(query: &str) -> BTreeSet<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

/// How many of the query words are exactly one of these abbreviation keys (R24).
pub(crate) fn exact_abbreviations(words: &BTreeSet<String>, keys: &[&str]) -> usize {
    words.iter().filter(|w| keys.contains(&w.as_str())).count()
}

/// The recents (the newest `recents_n`, R7) first, then all other active combos by label,
/// so a new catalog is listed before anything is tracked.
fn blank(catalog: &Catalog, recents: &[Combo]) -> Vec<ComboRow> {
    let n = (catalog.settings().recents_n as usize).min(QUICK_SWITCH_LIMIT);
    let mut rows = super::combo::rows(catalog, recents);
    rows.truncate(n);
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
