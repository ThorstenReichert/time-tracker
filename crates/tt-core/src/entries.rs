//! Time entries: the only part of `State` that grows (OQ-9). `decide` reaches them only
//! through the named queries below, each of which an indexed SQL query could answer, never
//! by iterating all entries.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{EntryId, Instant, Note, ProjectId, TaskId};

/// A tracked interval `[start, end)` (half-open, so a switch shares a boundary without
/// overlapping). `end` is `None` while the entry is running.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: EntryId,
    pub project: ProjectId,
    pub task: Option<TaskId>,
    pub start: Instant,
    pub end: Option<Instant>,
    pub note: Option<Note>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl Entry {
    pub fn combo(&self) -> Combo {
        Combo {
            project: self.project,
            task: self.task,
        }
    }

    pub fn is_running(&self) -> bool {
        self.end.is_none()
    }

    /// Tracked seconds; a running entry counts up to `now` (R14).
    pub fn duration_until(&self, now: Instant) -> i64 {
        self.end.unwrap_or(now).seconds_since(self.start).max(0)
    }
}

/// What an entry is booked on: a project and optionally one of its tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Combo {
    pub project: ProjectId,
    pub task: Option<TaskId>,
}

/// All entries, with the indexes the named queries need. Serialised as the plain list of
/// entries; the indexes are rebuilt on load.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<Entry>", into = "Vec<Entry>")]
pub struct Entries {
    by_id: BTreeMap<EntryId, Entry>,
    by_start: BTreeSet<(Instant, EntryId)>,
    running: Option<EntryId>,
}

impl Entries {
    /// Builds the entries from loaded rows. Run `State::check_invariants` on the result.
    pub fn from_parts(entries: impl IntoIterator<Item = Entry>) -> Self {
        let mut all = Self::default();
        for entry in entries {
            all.insert(entry);
        }
        all
    }

    pub fn get(&self, id: EntryId) -> Option<&Entry> {
        self.by_id.get(&id)
    }

    /// The running entry, if any (R4: at most one).
    pub fn running(&self) -> Option<&Entry> {
        self.running.and_then(|id| self.by_id.get(&id))
    }

    /// Entries whose interval overlaps `[start, end)`, ordered by start. `end: None` means
    /// open-ended, and a running entry counts as open-ended too; with no start or end in the
    /// future (R12), that's the same as "running until now". `except` is left out (the
    /// entry being edited or stopped).
    ///
    /// Uses the start index: because entries never overlap, ordering by start also orders
    /// them by end, so the scan walks back from the last entry starting before `end` and
    /// stops at the first one that ends at or before `start`.
    pub fn overlapping(
        &self,
        start: Instant,
        end: Option<Instant>,
        except: Option<EntryId>,
    ) -> Vec<&Entry> {
        let before_end = match end {
            Some(end) => self.by_start.range(..(end, EntryId::new(0))),
            None => self.by_start.range(..),
        };
        let mut found = Vec::new();
        for (_, id) in before_end.rev() {
            if Some(*id) == except {
                continue;
            }
            let entry = &self.by_id[id];
            if entry.end.is_some_and(|e| e <= start) {
                break;
            }
            found.push(entry);
        }
        found.reverse();
        found
    }

    /// Entries that start in `[from, to)`, ordered by start (the day assignment of R18
    /// step 1 uses the start only).
    pub fn started_between(&self, from: Instant, to: Instant) -> Vec<&Entry> {
        if to <= from {
            return Vec::new();
        }
        self.by_start
            .range((from, EntryId::new(0))..(to, EntryId::new(0)))
            .map(|(_, id)| &self.by_id[id])
            .collect()
    }

    /// The last `n` distinct combos, newest start first (R7 recents).
    pub fn recent_combos(&self, n: usize) -> Vec<Combo> {
        self.recent_combos_where(n, |_| true)
    }

    /// The last `n` distinct combos that `keep` accepts, newest start first. A named query:
    /// SQL could answer it from the start index with a join on `archived_at` (R7, OQ-9).
    pub fn recent_combos_where(&self, n: usize, mut keep: impl FnMut(Combo) -> bool) -> Vec<Combo> {
        let mut seen = BTreeSet::new();
        self.by_start
            .iter()
            .rev()
            .map(|(_, id)| self.by_id[id].combo())
            .filter(|combo| seen.insert(*combo) && keep(*combo))
            .take(n)
            .collect()
    }

    /// All entries ordered by start. Only for invariant checks, never for `decide`.
    pub(crate) fn all(&self) -> impl Iterator<Item = &Entry> {
        self.by_start.iter().map(|(_, id)| &self.by_id[id])
    }

    /// Whether the indexes agree with the entries (always, unless there's a bug here).
    pub(crate) fn indexes_consistent(&self) -> bool {
        let starts: BTreeSet<_> = self.by_id.values().map(|e| (e.start, e.id)).collect();
        let running: Vec<_> = self.by_id.values().filter(|e| e.is_running()).collect();
        starts == self.by_start
            && match running.as_slice() {
                [] => self.running.is_none(),
                [one] => self.running == Some(one.id),
                _ => true, // more than one running entry is reported separately
            }
    }

    pub(crate) fn running_count(&self) -> usize {
        self.by_id.values().filter(|e| e.is_running()).count()
    }

    pub(crate) fn insert(&mut self, entry: Entry) {
        self.by_start.insert((entry.start, entry.id));
        if entry.is_running() && self.running.is_none() {
            self.running = Some(entry.id);
        }
        if let Some(old) = self.by_id.insert(entry.id, entry) {
            panic!("entry {} inserted twice", old.id);
        }
    }

    pub(crate) fn replace(&mut self, entry: Entry) {
        self.remove(entry.id);
        self.insert(entry);
    }

    pub(crate) fn remove(&mut self, id: EntryId) -> Entry {
        let entry = self.by_id.remove(&id).expect("removed entry exists");
        self.by_start.remove(&(entry.start, id));
        if self.running == Some(id) {
            self.running = None;
        }
        entry
    }

    pub(crate) fn stop(&mut self, id: EntryId, end: Instant, updated_at: Instant) {
        let entry = self.by_id.get_mut(&id).expect("stopped entry exists");
        entry.end = Some(end);
        entry.updated_at = updated_at;
        if self.running == Some(id) {
            self.running = None;
        }
    }
}

impl From<Vec<Entry>> for Entries {
    fn from(entries: Vec<Entry>) -> Self {
        Self::from_parts(entries)
    }
}

impl From<Entries> for Vec<Entry> {
    fn from(entries: Entries) -> Self {
        entries.by_id.into_values().collect()
    }
}
