//! Typed IDs. The core assigns them from counters in `State`, so they are never reused.

use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            /// Wraps a raw ID, e.g. one loaded from the database.
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(
    /// Identifies a client.
    ClientId
);
id_type!(
    /// Identifies a project.
    ProjectId
);
id_type!(
    /// Identifies a task.
    TaskId
);
id_type!(
    /// Identifies a time entry.
    EntryId
);
