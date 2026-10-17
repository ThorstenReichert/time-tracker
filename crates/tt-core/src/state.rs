use serde::{Deserialize, Serialize};

use crate::{Catalog, Entries};

/// The whole world, held in memory and loaded at startup (spec §2). Only `evolve` changes it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub(crate) catalog: Catalog,
    pub(crate) entries: Entries,
}

impl State {
    /// Builds a state from loaded parts. Run [`State::check_invariants`] on the result.
    pub fn from_parts(catalog: Catalog, entries: Entries) -> Self {
        Self { catalog, entries }
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn entries(&self) -> &Entries {
        &self.entries
    }
}
