//! Errors as the UI sees them. Domain errors keep the core's wording; an overlap is rendered
//! with the conflicting entry's combo and local times, as R12 asks.

use serde::Serialize;
use specta::Type;
use tt_app::{App, AppError};
use tt_core::{Ctx, DomainError, ItemRef, to_local};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ErrorKind {
    /// Rejected by a business rule; nothing changed.
    Domain,
    /// Saving failed; the app reloaded its data from the database.
    Store,
    /// The app accepts no changes (see the status).
    Broken,
    /// The UI sent something unreadable (a bug in the UI).
    Input,
    /// Windows refused (e.g. the autostart entry); nothing changed.
    System,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UiError {
    pub kind: ErrorKind,
    pub message: String,
}

impl UiError {
    pub fn input(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Input,
            message: message.into(),
        }
    }

    /// A value the driver can't use, e.g. a hotkey it can't parse; nothing changed.
    pub fn rejected(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Domain,
            message: message.into(),
        }
    }

    pub fn system(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::System,
            message: message.into(),
        }
    }

    pub fn broken(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Broken,
            message: message.into(),
        }
    }

    /// Maps an app error; `app` and `ctx` are used to describe an overlapped entry.
    pub fn from_app(error: AppError, app: &App, ctx: &Ctx) -> Self {
        match error {
            AppError::Domain(e) => Self {
                kind: ErrorKind::Domain,
                message: domain_message(&e, app, ctx),
            },
            AppError::Store(e) => Self {
                kind: ErrorKind::Store,
                message: format!("Saving failed, nothing was changed: {e}"),
            },
            AppError::Broken(reason) => Self::broken(reason),
            AppError::Locked(path) => Self::broken(format!(
                "The database {} is in use by another process",
                path.display()
            )),
        }
    }
}

fn domain_message(error: &DomainError, app: &App, ctx: &Ctx) -> String {
    if let DomainError::Overlap { with, start, end } = error {
        let hm = |i| to_local(&ctx.tz, i).strftime("%H:%M").to_string();
        let until = end.map(hm).unwrap_or_else(|| "now".to_owned());
        let label = app
            .entry(ctx, *with)
            .ok()
            .flatten()
            .map(|row| row.combo.label)
            .unwrap_or_else(|| format!("entry {with}"));
        return format!("Overlaps {label} {}–{until}", hm(*start));
    }
    if let DomainError::DuplicateName { existing } = error
        && let Some(item) = item_name(app, *existing)
    {
        return format!("The name is already used by {item}");
    }
    capitalise(&error.to_string())
}

/// `client "Acme"`, or `None` if the item is gone.
fn item_name(app: &App, item: ItemRef) -> Option<String> {
    let catalog = app.catalog().ok()?;
    let (kind, name) = match item {
        ItemRef::Client(id) => ("client", catalog.client(id)?.name.to_string()),
        ItemRef::Project(id) => ("project", catalog.project(id)?.name.to_string()),
        ItemRef::Task(id) => ("task", catalog.task(id)?.name.to_string()),
    };
    Some(format!("{kind} \"{name}\""))
}

fn capitalise(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use tt_app::AppConfig;
    use tt_core::testing::ctx_berlin;
    use tt_core::{ClientId, Command, ProjectId, RoundingRule, TaskChoice};

    use super::*;

    #[test]
    fn errors_name_the_conflicting_item() {
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = ctx_berlin("2026-10-17T18:00");
        let app = App::open(AppConfig::new(dir.path().join("t.db")), &ctx).unwrap();
        let add = |start: &str, end: &str| Command::AddEntry {
            project: ProjectId::new(1),
            task: TaskChoice::NoTask,
            start: start.parse().unwrap(),
            end: end.parse().unwrap(),
            note: None,
        };
        for cmd in [
            Command::CreateClient {
                name: "Acme".into(),
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: ClientId::new(1),
                name: "Web".into(),
            },
            add("2026-10-17T09:00", "2026-10-17T10:22"),
        ] {
            app.execute(&ctx, cmd).unwrap();
        }
        let err = app
            .execute(&ctx, add("2026-10-17T10:00", "2026-10-17T11:00"))
            .unwrap_err();
        let ui = UiError::from_app(err, &app, &ctx);
        assert_eq!(ui.kind, ErrorKind::Domain);
        assert_eq!(ui.message, "Overlaps Acme / Web 09:00–10:22");

        let err = app
            .execute(
                &ctx,
                Command::CreateClient {
                    name: "ACME".into(),
                    rule: RoundingRule::default(),
                },
            )
            .unwrap_err();
        assert_eq!(
            UiError::from_app(err, &app, &ctx).message,
            "The name is already used by client \"Acme\""
        );

        let err = app.execute(&ctx, Command::Stop { at: None }).unwrap_err();
        assert_eq!(
            UiError::from_app(err, &app, &ctx).message,
            "No timer is running"
        );
    }
}
