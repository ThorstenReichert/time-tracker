//! Global hotkey (R6): shows and focuses the quick-switch popup.

use tauri::{AppHandle, Emitter, Manager, Runtime, plugin::TauriPlugin};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::backend::Backend;
use crate::{QUICK_SWITCH_WINDOW, tray};

/// Default hotkey. Debug builds use another one, so they can run next to the installed app.
pub const DEFAULT_HOTKEY: &str = if cfg!(debug_assertions) {
    "Ctrl+Alt+Shift+Space"
} else {
    "Ctrl+Alt+Space"
};

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            // The handler fires on press and on release; act once.
            if event.state == ShortcutState::Pressed {
                show_quick_switch(app);
            }
        })
        .build()
}

/// The hotkey from the settings, or the default.
pub fn configured(app: &AppHandle) -> String {
    app.try_state::<Backend>()
        .and_then(|b| b.app().ok().and_then(|a| a.catalog().ok()))
        .and_then(|c| c.settings().hotkey.clone())
        .unwrap_or_else(|| DEFAULT_HOTKEY.to_owned())
}

/// Registers the configured hotkey. A failure (AC-1: taken by another app) is shown in the
/// tray tooltip instead of failing silently; step 10 adds the main-window banner.
pub fn register_from_settings(app: &AppHandle) {
    let hotkey = configured(app);
    if let Err(e) = app.global_shortcut().register(hotkey.as_str()) {
        let message = format!("Hotkey {hotkey} unavailable: {e}");
        eprintln!("{message}");
        tray::set_tooltip(app, &message);
    }
}

fn show_quick_switch<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(QUICK_SWITCH_WINDOW) {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit_to(QUICK_SWITCH_WINDOW, "quick-switch-shown", ());
    }
}
