//! GUI driver: translates UI calls, the tray and the global hotkey into tt-app calls.
//! No business rules live here (CLAUDE.md "Drivers").

mod bindings;
mod commands;
mod hotkey;
mod tray;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};

/// Label of the main window (tauri.conf.json).
pub const MAIN_WINDOW: &str = "main";
/// Label of the quick-switch popup (tauri.conf.json).
pub const QUICK_SWITCH_WINDOW: &str = "quick-switch";
/// Argument the autostart entry passes, so the app starts hidden in the tray.
const AUTOSTART_ARG: &str = "--autostart";

pub fn run() {
    let specta = bindings::specta_builder();

    tauri::Builder::default()
        // Must be the first plugin: a second launch is forwarded here before anything else
        // (in particular the database lock) runs.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(hotkey::plugin())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(specta.invoke_handler())
        .setup(move |app| {
            specta.mount_events(app);
            tray::create(app.handle())?;
            hotkey::register_default(app.handle());
            enable_autostart_in_release(app.handle());
            if !std::env::args().any(|a| a == AUTOSTART_ARG) {
                show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            // Closing the main window hides it to the tray; the app quits only via the tray (R8).
            (MAIN_WINDOW, WindowEvent::CloseRequested { api, .. }) => {
                api.prevent_close();
                let _ = window.hide();
            }
            // The popup hides when it loses focus (spec §2 "Windows").
            (QUICK_SWITCH_WINDOW, WindowEvent::Focused(false)) => {
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}

pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Step 0 spike for AC-2: the installed build registers itself for autostart. Debug builds
/// never do, so `cargo tauri dev` can't point the Run key at `target\debug`. Step 8 moves
/// this to "first run only", and step 13 adds the toggle.
fn enable_autostart_in_release(app: &tauri::AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    let autolaunch = app.autolaunch();
    if !autolaunch.is_enabled().unwrap_or(false)
        && let Err(e) = autolaunch.enable()
    {
        eprintln!("autostart could not be enabled: {e}");
    }
}
