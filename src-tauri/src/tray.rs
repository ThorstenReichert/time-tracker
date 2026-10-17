//! Tray icon (R8): idle/running icon and a tooltip with the running combo. Step 10 adds the
//! once-a-minute tooltip refresh, Stop and the recents in the menu.

use tauri::{
    AppHandle, Manager,
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::backend::{Backend, ctx};

const TRAY_ID: &str = "main";
/// Windows truncates tray tooltips longer than this.
const TOOLTIP_MAX_CHARS: usize = 127;
const IDLE_ICON: &[u8] = include_bytes!("../icons/tray-idle.png");
const RUNNING_ICON: &[u8] = include_bytes!("../icons/tray-running.png");

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &separator, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(IDLE_ICON)?)
        .tooltip("ThorSoft TimeTracker – idle")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => crate::show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    refresh(app);
    Ok(())
}

/// Updates icon and tooltip from the running entry.
pub fn refresh(app: &AppHandle) {
    let Some(backend) = app.try_state::<Backend>() else {
        return;
    };
    let ctx = ctx();
    let (running, text) = match backend.app().map(|a| a.running(&ctx)) {
        Ok(Ok(Some(entry))) => (
            true,
            format!("{} · {}", entry.combo.label, elapsed(entry.duration)),
        ),
        Ok(Ok(None)) => (false, "ThorSoft TimeTracker – idle".to_owned()),
        Ok(Err(e)) => (false, format!("ThorSoft TimeTracker – {e}")),
        Err(e) => (false, format!("ThorSoft TimeTracker – {}", e.message)),
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let icon = if running { RUNNING_ICON } else { IDLE_ICON };
        if let Ok(image) = Image::from_bytes(icon) {
            let _ = tray.set_icon(Some(image));
        }
        let _ = tray.set_tooltip(Some(truncate(&text)));
    }
}

pub fn set_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(truncate(text)));
    }
}

fn truncate(text: &str) -> String {
    text.chars().take(TOOLTIP_MAX_CHARS).collect()
}

/// Elapsed time as h:mm.
fn elapsed(seconds: i64) -> String {
    let minutes = seconds.max(0) / 60;
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_is_hours_and_minutes() {
        assert_eq!(elapsed(0), "0:00");
        assert_eq!(elapsed(83 * 60 + 59), "1:23");
        assert_eq!(elapsed(10 * 3600), "10:00");
    }

    #[test]
    fn tooltips_are_cut_to_the_windows_limit() {
        assert_eq!(
            truncate(&"ä".repeat(200)).chars().count(),
            TOOLTIP_MAX_CHARS
        );
    }
}
