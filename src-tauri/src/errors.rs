//! Errors as the UI sees them. Domain errors keep the core's wording; an overlap is rendered
//! with the conflicting entry's combo and local times, as R12 asks.

use serde::Serialize;
use specta::Type;
use tt_app::{App, AppError};
use tt_core::{Catalog, Ctx, DomainError, ItemRef, to_local};

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
    if let DomainError::DuplicateAbbreviation { existing } = error
        && let Some(item) = item_name(app, *existing)
    {
        return format!("The abbreviation is already used by {item}");
    }
    // R2d: the archived item is named without its abbreviation or the word "archived".
    if let DomainError::Archived { item } = error
        && let Some(item) = app.catalog().ok().and_then(|c| item_parts(&c, *item))
    {
        return format!("{} \"{}\" is archived", capitalise(item.kind), item.name);
    }
    if let DomainError::ProjectRunning { .. } = error {
        return "The timer is running on this project; stop it first".to_owned();
    }
    if let DomainError::ClientRunning { .. } = error {
        return "The timer is running on a project of this client; stop it first".to_owned();
    }
    capitalise(&error.to_string())
}

struct ItemParts {
    kind: &'static str,
    name: String,
    abbreviation: Option<String>,
    archived: bool,
}

fn item_parts(catalog: &Catalog, item: ItemRef) -> Option<ItemParts> {
    let (kind, name, abbreviation, archived_at) = match item {
        ItemRef::Client(id) => {
            let c = catalog.client(id)?;
            ("client", &c.name, &c.abbreviation, c.archived_at)
        }
        ItemRef::Project(id) => {
            let p = catalog.project(id)?;
            ("project", &p.name, &p.abbreviation, p.archived_at)
        }
        ItemRef::Task(id) => {
            let t = catalog.task(id)?;
            ("task", &t.name, &t.abbreviation, t.archived_at)
        }
    };
    Some(ItemParts {
        kind,
        name: name.to_string(),
        abbreviation: abbreviation.as_ref().map(ToString::to_string),
        archived: archived_at.is_some(),
    })
}

fn item_name(app: &App, item: ItemRef) -> Option<String> {
    item_label(&app.catalog().ok()?, item)
}

/// `client "Acme"`, or `client "Acme" (AC)` with its abbreviation (R24), with "archived "
/// in front for an archived item (R2d), or `None` if the item is gone. The New project
/// dialog's name check shows the same labels.
pub fn item_label(catalog: &Catalog, item: ItemRef) -> Option<String> {
    let ItemParts {
        kind,
        name,
        abbreviation,
        archived,
    } = item_parts(catalog, item)?;
    let archived = if archived { "archived " } else { "" };
    Some(match abbreviation {
        Some(a) => format!("{archived}{kind} \"{name}\" ({a})"),
        None => format!("{archived}{kind} \"{name}\""),
    })
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
                abbreviation: None,
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: ClientId::new(1),
                name: "Web".into(),
                abbreviation: None,
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
                    abbreviation: None,
                    rule: RoundingRule::default(),
                },
            )
            .unwrap_err();
        assert_eq!(
            UiError::from_app(err, &app, &ctx).message,
            "The name is already used by client \"Acme\""
        );

        // An abbreviation clash names the item with its abbreviation (R24).
        app.execute(
            &ctx,
            Command::UpdateProject {
                id: ProjectId::new(1),
                name: "Web".into(),
                abbreviation: Some("W".into()),
            },
        )
        .unwrap();
        let err = app
            .execute(
                &ctx,
                Command::CreateProject {
                    client: ClientId::new(1),
                    name: "Mobile".into(),
                    abbreviation: Some("w".into()),
                },
            )
            .unwrap_err();
        assert_eq!(
            UiError::from_app(err, &app, &ctx).message,
            "The abbreviation is already used by project \"Web\" (W)"
        );

        let err = app.execute(&ctx, Command::Stop { at: None }).unwrap_err();
        assert_eq!(
            UiError::from_app(err, &app, &ctx).message,
            "No timer is running"
        );
    }

    #[test]
    fn archive_errors_name_the_archived_project() {
        // R2d (changes/2026-11-05-archive-projects).
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = ctx_berlin("2026-10-17T18:00");
        let app = App::open(AppConfig::new(dir.path().join("t.db")), &ctx).unwrap();
        let web = ProjectId::new(1);
        let start = |project| Command::Start {
            project,
            task: TaskChoice::NoTask,
        };
        for cmd in [
            Command::CreateClient {
                name: "Acme".into(),
                abbreviation: None,
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: ClientId::new(1),
                name: "Web".into(),
                abbreviation: Some("W".into()),
            },
            start(web),
        ] {
            app.execute(&ctx, cmd).unwrap();
        }
        let message =
            |cmd| UiError::from_app(app.execute(&ctx, cmd).unwrap_err(), &app, &ctx).message;
        assert_eq!(
            message(Command::ArchiveProject { id: web }),
            "The timer is running on this project; stop it first"
        );
        app.execute(&ctx, Command::Stop { at: None }).unwrap();
        app.execute(&ctx, Command::ArchiveProject { id: web })
            .unwrap();
        assert_eq!(message(start(web)), "Project \"Web\" is archived");
        // The Edit dialog's rename clash says the project is archived (no unarchive there).
        app.execute(
            &ctx,
            Command::CreateProject {
                client: ClientId::new(1),
                name: "Mobile".into(),
                abbreviation: None,
            },
        )
        .unwrap();
        let rename = Command::UpdateProject {
            id: ProjectId::new(2),
            name: "web".into(),
            abbreviation: None,
        };
        assert_eq!(
            message(rename),
            "The name is already used by archived project \"Web\" (W)"
        );
    }

    #[test]
    fn archive_errors_name_the_archived_client() {
        // R2k (changes/2026-11-05-archive-clients).
        let dir = tempfile::TempDir::new().unwrap();
        let ctx = ctx_berlin("2026-10-17T18:00");
        let app = App::open(AppConfig::new(dir.path().join("t.db")), &ctx).unwrap();
        let acme = ClientId::new(1);
        let web = ProjectId::new(1);
        let start = |project| Command::Start {
            project,
            task: TaskChoice::NoTask,
        };
        for cmd in [
            Command::CreateClient {
                name: "Acme".into(),
                abbreviation: Some("AC".into()),
                rule: RoundingRule::default(),
            },
            Command::CreateProject {
                client: acme,
                name: "Web".into(),
                abbreviation: None,
            },
            start(web),
        ] {
            app.execute(&ctx, cmd).unwrap();
        }
        let message =
            |cmd| UiError::from_app(app.execute(&ctx, cmd).unwrap_err(), &app, &ctx).message;
        assert_eq!(
            message(Command::ArchiveClient { id: acme }),
            "The timer is running on a project of this client; stop it first"
        );
        app.execute(&ctx, Command::Stop { at: None }).unwrap();
        app.execute(&ctx, Command::ArchiveClient { id: acme })
            .unwrap();
        assert_eq!(message(start(web)), "Client \"Acme\" is archived");
        assert_eq!(
            message(Command::CreateClient {
                name: "acme".into(),
                abbreviation: None,
                rule: RoundingRule::default(),
            }),
            "The name is already used by archived client \"Acme\" (AC)"
        );
    }
}
