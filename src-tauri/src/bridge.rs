//! Turns the events tt-app publishes into UI events (spec §2 "State sync"). tt-app calls
//! its subscribers under its lock, so the subscriber only sends on a channel; this thread
//! does the rest outside the lock, where reading the app again is safe.

use std::sync::mpsc::Receiver;

use tauri::{AppHandle, Emitter};
use tt_core::Event;

/// Something changed; windows reload what they show.
pub const DATA_CHANGED: &str = "data-changed";
/// The running timer started, stopped or changed.
pub const TIMER_CHANGED: &str = "timer-changed";
/// A tray menu action failed; the payload is the message. Sent to the main window.
pub const ACTION_FAILED: &str = "action-failed";

pub fn start(app: AppHandle, events: Receiver<Vec<Event>>) {
    std::thread::Builder::new()
        .name("event-bridge".into())
        .spawn(move || {
            for batch in events {
                let _ = app.emit(DATA_CHANGED, ());
                if batch.iter().any(touches_entries) {
                    let _ = app.emit(TIMER_CHANGED, ());
                }
                if batch
                    .iter()
                    .any(|e| matches!(e, Event::SettingsChanged { .. }))
                {
                    // R6: a new hotkey replaces the old one at once.
                    crate::hotkey::reregister(&app);
                }
                // Any change can touch the tray menu (recents, names, the recents count).
                crate::tray::nudge(&app);
            }
        })
        .expect("spawn the event bridge");
}

fn touches_entries(event: &Event) -> bool {
    matches!(
        event,
        Event::EntryStarted { .. }
            | Event::EntryStopped { .. }
            | Event::EntryAdded { .. }
            | Event::EntryUpdated { .. }
            | Event::EntryDeleted { .. }
            // A rename changes the running combo's label.
            | Event::ClientUpdated { .. }
            | Event::ProjectRenamed { .. }
            | Event::TaskRenamed { .. }
    )
}
