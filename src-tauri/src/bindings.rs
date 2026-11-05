//! The specta builder shared by the app and the bindings test. `ui/src/bindings.ts` is
//! generated from it; regenerate with `UPDATE_BINDINGS=1 cargo test -p timetracker-app bindings`.

#[cfg(test)]
use std::path::Path;

#[cfg(test)]
use specta_typescript::Typescript;
use tauri_specta::{Builder, collect_commands};

use crate::commands;

pub fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::get_status,
            commands::get_today,
            commands::get_catalog,
            commands::create_client,
            commands::update_client,
            commands::create_project,
            commands::update_project,
            commands::archive_project,
            commands::unarchive_project,
            commands::check_project_name,
            commands::archive_client,
            commands::unarchive_client,
            commands::check_client_name,
            commands::create_task,
            commands::update_task,
            commands::start,
            commands::stop,
            commands::add_entry,
            commands::edit_entry,
            commands::delete_entry,
            commands::update_settings,
            commands::get_settings_defaults,
            commands::get_autostart,
            commands::set_autostart,
            commands::get_running,
            commands::get_day_entries,
            commands::get_overview,
            commands::get_archive,
            commands::get_recents,
            commands::quick_switch,
            commands::task_matches,
        ])
        // IDs and Unix seconds stay below 2^53, so they fit a JS number exactly.
        .dangerously_cast_bigints_to_number()
}

/// Only the bindings test writes the file; the app never touches the source tree.
#[cfg(test)]
fn export(path: &Path) -> Result<(), specta_typescript::Error> {
    specta_builder().export(
        Typescript::default().header("// Generated from src-tauri by tauri-specta. Do not edit."),
        path,
    )
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::export;

    fn committed_bindings() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/src/bindings.ts")
    }

    fn read_normalised(path: &Path) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .replace("\r\n", "\n")
    }

    #[test]
    fn bindings_up_to_date() {
        let committed = committed_bindings();
        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            export(&committed).expect("export bindings");
            return;
        }
        let fresh = std::env::temp_dir().join(format!("tt-bindings-{}.ts", std::process::id()));
        export(&fresh).expect("export bindings");
        let (want, have) = (read_normalised(&fresh), read_normalised(&committed));
        let _ = std::fs::remove_file(&fresh);
        assert!(
            want == have,
            "ui/src/bindings.ts is stale; run `UPDATE_BINDINGS=1 cargo test -p timetracker-app bindings`"
        );
    }
}
