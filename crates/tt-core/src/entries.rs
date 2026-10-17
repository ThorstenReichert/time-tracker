//! Time entries: the only part of `State` that grows (OQ-9). Step 3 fills this in; `decide`
//! will reach entries only through named queries that SQL could answer with an index.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entries {}
