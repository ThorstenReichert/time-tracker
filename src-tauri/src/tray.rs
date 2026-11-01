//! Tray icon (R8): idle/running icon, a tooltip with the running combo and its elapsed time,
//! and a menu with Stop, the recents, Open window and Quit. One thread refreshes the tray: on
//! every nudge (data changed, hotkey status changed), and each time the elapsed minutes change
//! while a timer runs (spec §2 "Tray elapsed time").

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use tauri::{
    AppHandle, Emitter, Manager,
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tt_core::read::{ComboRow, EntryRow};
use tt_core::{Combo, Command, ProjectId, TaskChoice, TaskId};

use crate::backend::{Backend, ctx};
use crate::hotkey::HotkeyStatus;
use crate::{MAIN_WINDOW, bridge, commands};

const TRAY_ID: &str = "main";
/// Windows keeps at most this many UTF-16 units of a tray tooltip, plus the final NUL
/// (`szTip` holds 128).
const TOOLTIP_MAX_UNITS: usize = 127;
const IDLE_ICON: &[u8] = include_bytes!("../icons/tray-idle.png");
const RUNNING_ICON: &[u8] = include_bytes!("../icons/tray-running.png");
const IDLE_TOOLTIP: &str = "ThorSoft TimeTracker – idle";

const OPEN: &str = "open";
const QUIT: &str = "quit";
const STOP: &str = "stop";
/// Menu IDs of the recents: `start:<project>:<task or ->`.
const START_PREFIX: &str = "start:";

/// Wakes the tray thread.
struct Nudge(Sender<()>);

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(IDLE_ICON)?)
        .tooltip(IDLE_TOOLTIP)
        .menu(&menu(app, None, &[])?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu_event(app, event.id.as_ref()))
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
    let (nudges, rx) = mpsc::channel();
    app.manage(Nudge(nudges));
    start_thread(app.clone(), rx);
    Ok(())
}

/// Refreshes icon, tooltip and menu soon.
pub fn nudge(app: &AppHandle) {
    if let Some(nudge) = app.try_state::<Nudge>() {
        let _ = nudge.0.send(());
    }
}

fn start_thread(app: AppHandle, nudges: Receiver<()>) {
    std::thread::Builder::new()
        .name("tray".into())
        .spawn(move || {
            let mut with_menu = true;
            loop {
                // Only a nudge rebuilds the menu, so a menu the user has open stays put.
                let elapsed = refresh(&app, with_menu);
                let woken = match next_tick(elapsed) {
                    Some(wait) => nudges.recv_timeout(wait),
                    None => nudges.recv().map_err(|_| RecvTimeoutError::Disconnected),
                };
                with_menu = match woken {
                    Ok(()) => true,
                    Err(RecvTimeoutError::Timeout) => false,
                    Err(RecvTimeoutError::Disconnected) => return,
                };
            }
        })
        .expect("spawn the tray thread");
}

/// When the tooltip's h:mm changes next: at the running entry's next full minute. Idle, it
/// never changes on its own.
fn next_tick(elapsed: Option<i64>) -> Option<Duration> {
    elapsed.map(|seconds| Duration::from_secs((60 - seconds.rem_euclid(60)) as u64))
}

/// Updates icon and tooltip, and the menu if `with_menu`. Returns the running entry's elapsed
/// seconds.
fn refresh(app: &AppHandle, with_menu: bool) -> Option<i64> {
    let tray = app.tray_by_id(TRAY_ID)?;
    let backend = app.try_state::<Backend>()?;
    let ctx = ctx();
    let running = backend
        .app()
        .map_err(|e| e.message)
        .and_then(|a| a.running(&ctx).map_err(|e| e.to_string()));

    // The hotkey warning comes first, so a long combo label can't cut it off (AC-1).
    let mut lines: Vec<String> = app
        .try_state::<HotkeyStatus>()
        .and_then(|s| s.tooltip())
        .into_iter()
        .collect();
    lines.push(match &running {
        Ok(Some(entry)) => format!("{} · {}", entry.combo.short_label, elapsed(entry.duration)),
        Ok(None) => IDLE_TOOLTIP.to_owned(),
        Err(e) => format!("ThorSoft TimeTracker – {e}"),
    });
    let running = running.ok().flatten();

    let icon = if running.is_some() {
        RUNNING_ICON
    } else {
        IDLE_ICON
    };
    if let Ok(image) = Image::from_bytes(icon) {
        let _ = tray.set_icon(Some(image));
    }
    let _ = tray.set_tooltip(Some(truncate(&lines.join("\n"))));

    if with_menu {
        let recents = backend
            .app()
            .ok()
            .and_then(|a| a.recents().ok())
            .unwrap_or_default();
        match menu(app, running.as_ref(), &recents) {
            Ok(menu) => {
                let _ = tray.set_menu(Some(menu));
            }
            Err(e) => eprintln!("tray menu: {e}"),
        }
    }
    running.map(|entry| entry.duration)
}

fn menu(
    app: &AppHandle,
    running: Option<&EntryRow>,
    recents: &[ComboRow],
) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let stop = match running {
        Some(entry) => format!("Stop {}", entry.combo.short_label),
        None => "Stop (no timer running)".to_owned(),
    };
    menu.append(&MenuItem::with_id(
        app,
        STOP,
        menu_text(&stop),
        running.is_some(),
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    for row in recents {
        // The running combo is greyed out: starting it again changes nothing (R5).
        let idle = running.is_none_or(|entry| entry.combo.combo != row.combo);
        menu.append(&MenuItem::with_id(
            app,
            start_id(row.combo),
            menu_text(&row.short_label),
            idle,
            None::<&str>,
        )?)?;
    }
    if !recents.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    menu.append(&MenuItem::with_id(
        app,
        OPEN,
        "Open window",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?)?;
    Ok(menu)
}

fn on_menu_event(app: &AppHandle, id: &str) {
    match id {
        OPEN => crate::show_main_window(app),
        QUIT => app.exit(0),
        STOP => run(app, Command::Stop { at: None }),
        _ => {
            if let Some(command) = parse_start_id(id) {
                run(app, command);
            }
        }
    }
}

/// Executes a menu command off the main thread, like the Tauri commands. A failure shows the
/// main window with the message, so it isn't lost.
fn run(app: &AppHandle, command: Command) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let backend = app.state::<Backend>();
        if let Err(e) = commands::execute(&backend, &ctx(), command) {
            crate::show_main_window(&app);
            let _ = app.emit_to(MAIN_WINDOW, bridge::ACTION_FAILED, e.message);
        }
    });
}

fn start_id(combo: Combo) -> String {
    let task = combo
        .task
        .map_or_else(|| "-".to_owned(), |t| t.get().to_string());
    format!("{START_PREFIX}{}:{task}", combo.project.get())
}

fn parse_start_id(id: &str) -> Option<Command> {
    let (project, task) = id.strip_prefix(START_PREFIX)?.split_once(':')?;
    let project = ProjectId::new(project.parse().ok()?);
    let task = match task {
        "-" => TaskChoice::NoTask,
        id => TaskChoice::Existing(TaskId::new(id.parse().ok()?)),
    };
    Some(Command::Start { project, task })
}

/// Windows menus treat `&` as a mnemonic marker and a tab as the start of the accelerator.
fn menu_text(label: &str) -> String {
    label.replace('&', "&&").replace('\t', " ")
}

fn truncate(text: &str) -> String {
    let mut units = 0;
    text.chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= TOOLTIP_MAX_UNITS
        })
        .collect()
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
            TOOLTIP_MAX_UNITS
        );
        // A character outside the BMP takes two units and is never split.
        assert_eq!(
            truncate(&"😀".repeat(100)).encode_utf16().count(),
            TOOLTIP_MAX_UNITS - 1
        );
        assert_eq!(truncate("short"), "short");
    }

    #[test]
    fn the_tooltip_refreshes_when_the_minute_changes() {
        assert_eq!(next_tick(None), None);
        assert_eq!(next_tick(Some(0)), Some(Duration::from_secs(60)));
        assert_eq!(next_tick(Some(83 * 60 + 59)), Some(Duration::from_secs(1)));
        assert_eq!(next_tick(Some(61)), Some(Duration::from_secs(59)));
    }

    #[test]
    fn start_ids_round_trip() {
        let project = ProjectId::new(7);
        for (task, choice) in [
            (None, TaskChoice::NoTask),
            (Some(TaskId::new(3)), TaskChoice::Existing(TaskId::new(3))),
        ] {
            let id = start_id(Combo { project, task });
            assert_eq!(
                parse_start_id(&id),
                Some(Command::Start {
                    project,
                    task: choice
                })
            );
        }
        assert_eq!(parse_start_id(OPEN), None);
        assert_eq!(parse_start_id("start:x:-"), None);
    }

    #[test]
    fn menu_labels_show_ampersands() {
        assert_eq!(menu_text("R&D / Web\tx"), "R&&D / Web x");
    }
}
