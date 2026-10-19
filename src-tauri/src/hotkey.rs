//! Global hotkey (R6): shows and focuses the quick-switch popup.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, Runtime, plugin::TauriPlugin};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::backend::Backend;
use crate::{QUICK_SWITCH_WINDOW, bridge, tray};

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

/// Why the configured hotkey isn't registered, if it isn't (AC-1). Managed state, read by the
/// main window's status banner and the tray tooltip.
#[derive(Default)]
pub struct HotkeyStatus(Mutex<Option<Failure>>);

struct Failure {
    hotkey: String,
    reason: String,
}

impl HotkeyStatus {
    /// The banner text for the main window.
    pub fn warning(&self) -> Option<String> {
        self.with(|f| {
            format!(
                "The hotkey {} is not available: {}. Choose another one in Settings.",
                f.hotkey, f.reason
            )
        })
    }

    /// The short line for the tray tooltip.
    pub fn tooltip(&self) -> Option<String> {
        self.with(|f| format!("Hotkey {} unavailable", f.hotkey))
    }

    fn with(&self, f: impl FnOnce(&Failure) -> String) -> Option<String> {
        self.0.lock().expect("hotkey status").as_ref().map(f)
    }

    fn set(&self, failure: Option<Failure>) {
        *self.0.lock().expect("hotkey status") = failure;
    }
}

/// The hotkey from the settings, or the default.
pub fn configured(app: &AppHandle) -> String {
    app.try_state::<Backend>()
        .and_then(|b| b.app().ok().and_then(|a| a.catalog().ok()))
        .and_then(|c| c.settings().hotkey.clone())
        .unwrap_or_else(|| DEFAULT_HOTKEY.to_owned())
}

/// Whether the global-shortcut plugin can read `hotkey`, e.g. `Ctrl+Alt+Space`.
pub fn is_valid(hotkey: &str) -> bool {
    hotkey.parse::<Shortcut>().is_ok()
}

/// Registers the hotkey again after the settings changed. The old one is freed first, so it's
/// free for other apps, and saving the same hotkey retries one that Windows refused before.
pub fn reregister(app: &AppHandle) {
    if let Err(e) = app.global_shortcut().unregister_all() {
        eprintln!("hotkeys not unregistered: {e}");
    }
    register_from_settings(app);
}

/// Registers the configured hotkey. A failure (AC-1: taken by another app) is shown in the
/// main window's banner and the tray tooltip instead of failing silently.
pub fn register_from_settings(app: &AppHandle) {
    let hotkey = configured(app);
    // Parsed here, so a refusal by Windows can be told apart from a malformed setting.
    let reason = match hotkey.parse::<Shortcut>() {
        Err(e) => Some(format!("it isn't a valid hotkey ({e})")),
        Ok(shortcut) => app.global_shortcut().register(shortcut).err().map(|e| {
            eprintln!("hotkey {hotkey} not registered: {e}");
            "Windows refused it, so another app probably uses it".to_owned()
        }),
    };
    let failure = reason.map(|reason| Failure {
        hotkey: hotkey.clone(),
        reason,
    });
    app.state::<HotkeyStatus>().set(failure);
    tray::nudge(app);
    // The main window may have loaded its status before this ran.
    let _ = app.emit(bridge::DATA_CHANGED, ());
}

fn show_quick_switch<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(QUICK_SWITCH_WINDOW) {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit_to(QUICK_SWITCH_WINDOW, "quick-switch-shown", ());
    }
}
