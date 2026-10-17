//! What a driver asks for. Commands are plain data; names arrive as raw text and are parsed
//! by `decide`, so an invalid name is a `DomainError`, not a deserialisation failure.

use serde::{Deserialize, Serialize};

use crate::{ClientId, ProjectId, RoundingRule, Settings, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    CreateClient {
        name: String,
        rule: RoundingRule,
    },
    /// Renames a client and/or changes its rounding rule (R17, R19).
    UpdateClient {
        id: ClientId,
        name: String,
        rule: RoundingRule,
    },
    CreateProject {
        client: ClientId,
        name: String,
    },
    RenameProject {
        id: ProjectId,
        name: String,
    },
    CreateTask {
        project: ProjectId,
        name: String,
    },
    RenameTask {
        id: TaskId,
        name: String,
    },
    UpdateSettings {
        settings: Settings,
    },
}
