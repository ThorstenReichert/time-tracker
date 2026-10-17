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

/// The entries that start on `date`, ordered by start. `entries` are the day's entries
/// (callers pass `Entries::started_between(day_bounds(date))`); others are ignored.
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
        .filter_map(|e| {
            Some(EntryRow {
                id: e.id,
                combo: combo_row(catalog, e.combo())?,
                start: e.start,
                end: e.end,
                duration: e.duration_until(now),
                running: e.is_running(),
                note: e.note.clone(),
            })
        })
        .collect();
    rows.sort_by_key(|r| (r.start, r.id));
    rows
}
