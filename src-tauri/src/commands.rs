//! Tauri commands: thin mappings from UI calls to tt-app. All commands are `async`, so they
//! run off the main thread.

use serde::Serialize;
use specta::Type;

/// Step 0 spike: proves the specta bindings, including a 64-bit integer field.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Pong {
    pub message: String,
    pub build_id: u64,
}

#[tauri::command]
#[specta::specta]
pub async fn ping() -> Pong {
    Pong {
        message: "pong".to_owned(),
        // 2^53 − 1: the largest integer a JS number holds exactly.
        build_id: 9_007_199_254_740_991,
    }
}
