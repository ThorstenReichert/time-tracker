use jiff::{civil, tz::TimeZone};
use serde::{Deserialize, Serialize};

use super::{ComboRow, combo_row, local_date};
use crate::{Catalog, Entry, EntryId, Instant, Note};

/// One line of the entry list for a day (R10, R13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryRow {
    pub id: EntryId,
    pub combo: ComboRow,
    pub start: Instant,
    pub end: Option<Instant>,
    /// Seconds; a running entry counts up to `now`.
    pub duration: i64,
    pub running: bool,
    pub note: Option<Note>,
}

/// The entries that start on `date`, newest first, so the running entry is on top (R10).
/// `entries` are the day's entries (callers pass `Entries::started_between(day_bounds(date))`);
/// others are ignored.
pub fn day_entries(
    catalog: &Catalog,
    entries: &[&Entry],
    now: Instant,
    tz: &TimeZone,
    date: civil::Date,
) -> Vec<EntryRow> {
    let mut rows: Vec<EntryRow> = entries
        .iter()
        .filter(|e| local_date(e.start, tz) == date)
        .filter_map(|e| entry_row(catalog, e, now))
        .collect();
    rows.sort_by_key(|r| std::cmp::Reverse((r.start, r.id)));
    rows
}

/// One entry with its names, e.g. the running entry for the timer display. `None` if its
/// combo doesn't exist in the catalog.
pub fn entry_row(catalog: &Catalog, entry: &Entry, now: Instant) -> Option<EntryRow> {
    Some(EntryRow {
        id: entry.id,
        combo: combo_row(catalog, entry.combo())?,
        start: entry.start,
        end: entry.end,
        duration: entry.duration_until(now),
        running: entry.is_running(),
        note: entry.note.clone(),
    })
}
