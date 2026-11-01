#![allow(dead_code)] // each test binary uses a different part

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tt_app::{App, AppConfig};
use tt_core::testing::ctx_berlin;
use tt_core::{ClientId, Command, Ctx, ProjectId, RoundingRule};

pub const WEB: ProjectId = ProjectId::new(1);
pub const API: ProjectId = ProjectId::new(2);

pub fn ctx(local: &str) -> Ctx {
    ctx_berlin(local)
}

pub struct Scratch {
    pub dir: TempDir,
}

impl Scratch {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    pub fn db(&self) -> PathBuf {
        self.dir.path().join("timetracker.db")
    }

    pub fn backups(&self) -> PathBuf {
        self.dir.path().join("backups")
    }

    pub fn open(&self, local_now: &str) -> App {
        App::open(AppConfig::new(self.db()), &ctx(local_now)).unwrap()
    }
}

/// Acme (1) / Web (1); Beta (2) / API (2).
pub fn setup_catalog(app: &App) {
    let c = ctx("2026-10-17T08:00");
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
        Command::CreateClient {
            name: "Beta".into(),
            abbreviation: None,
            rule: RoundingRule::default(),
        },
        Command::CreateProject {
            client: ClientId::new(2),
            name: "API".into(),
            abbreviation: None,
        },
    ] {
        app.execute(&c, cmd).unwrap();
    }
}

/// A raw connection to the same file, for fault injection and out-of-band edits.
pub fn raw(db: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(db).unwrap()
}

pub fn files_in(dir: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}
