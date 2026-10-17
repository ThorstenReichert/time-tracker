//! Read models (R3, R6, R7, R10, R14): pure functions over the `Catalog`, the entries they
//! need, `now`, the zone and the date or query. They take entries as input, never the whole
//! `State` (OQ-9), don't go through `decide` and are never stored.

mod combo;
mod day;
mod day_entries;
mod overview;
mod quick_switch;
mod task_match;

pub use combo::{ComboRow, combo_row, recents};
pub use day::{day_bounds, local_date};
pub use day_entries::{EntryRow, day_entries, entry_row};
pub use overview::{ClientBlock, Figure, Overview, ProjectRow, daily_overview};
pub use quick_switch::{QUICK_SWITCH_LIMIT, quick_switch};
pub use task_match::{TaskMatches, TaskRow, task_matches};
