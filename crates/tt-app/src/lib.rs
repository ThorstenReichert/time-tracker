//! The imperative shell around the core: owns `State` and the store behind one lock,
//! executes `Command`s and publishes `Event`s. Shared by every driver. No business rules
//! here: tt-app only sequences decide, evolve, the database and the subscribers. The driver
//! builds `Ctx`; tt-app never reads the clock or the time zone.

mod error;
mod paths;
mod startup;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use jiff::civil;
use tt_core::read::{self, ArchivedClient, ComboRow, EntryRow, NameCheck, Overview, TaskMatches};
use tt_core::{
    Catalog, ClientId, Command, Ctx, EntryId, Event, ProjectId, State, decide, evolve_all,
};
use tt_store::{MIGRATIONS, Migration, Store};

pub use error::AppError;
pub use paths::{app_data_dir, default_backup_dir, live_db_path};

/// Where and how to open the database.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db_path: PathBuf,
    /// Overrides the backup folder (otherwise the `backup_dir` setting, else
    /// `<db dir>/backups`). tt-cli sets it, so scratch databases never back up elsewhere.
    pub backup_dir_override: Option<PathBuf>,
    /// The migrations to apply; tests pass extra ones.
    pub migrations: &'static [Migration],
}

impl AppConfig {
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
            backup_dir_override: None,
            migrations: MIGRATIONS,
        }
    }
}

/// Whether the app accepts writes, and anything the user should know about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppStatus {
    /// `None` when ready; otherwise why writes are refused.
    pub broken: Option<String>,
    /// Problems that don't stop the app, e.g. a backup that failed.
    pub warnings: Vec<String>,
}

type Subscriber = Box<dyn Fn(&[Event]) + Send + Sync>;

#[expect(
    clippy::large_enum_variant,
    reason = "there is one Inner per app, behind a Mutex"
)]
enum Inner {
    Ready { state: State, store: Store },
    Broken { reason: String },
}

pub struct App {
    inner: Mutex<Inner>,
    warnings: Vec<String>,
    subscribers: Mutex<Vec<Subscriber>>,
    // Held for the lifetime of the app: one process per database file.
    _lock: tt_store::DbLock,
}

impl App {
    /// Runs the startup sequence (spec §2): lock → backup if the database exists →
    /// migrations → load State and check its invariants. Only a lock held by another
    /// process is an error; any later failure gives an app that is [`AppStatus::broken`].
    pub fn open(config: AppConfig, ctx: &Ctx) -> Result<App, AppError> {
        startup::open(config, ctx)
    }

    /// Executes a command through the single write path: decide → evolve a copy into a
    /// candidate → apply the events in one transaction → swap in the candidate → publish.
    /// On a store error the candidate is discarded and State is reloaded from the database
    /// (it is the authority); if that fails too, the app stops accepting writes.
    pub fn execute(&self, ctx: &Ctx, cmd: Command) -> Result<Vec<Event>, AppError> {
        let mut inner = self.lock();
        let Inner::Ready { state, store } = &mut *inner else {
            return Err(self.broken_error(&inner));
        };
        let events = decide(state, ctx, &cmd)?;
        if events.is_empty() {
            return Ok(events);
        }
        // evolve checks the invariants after every event in debug builds and tests; a bug
        // panics here, before anything is written (crash-only, plan.md "Panics").
        let candidate = evolve_all(state.clone(), &events);
        if let Err(write_error) = store.apply(&events) {
            match store
                .load()
                .map_err(|e| e.to_string())
                .and_then(|s| s.check_invariants().map(|()| s).map_err(|v| v.to_string()))
            {
                Ok(reloaded) => *state = reloaded,
                Err(reason) => {
                    *inner = Inner::Broken {
                        reason: format!("reload after a failed save failed: {reason}"),
                    }
                }
            }
            return Err(AppError::Store(write_error));
        }
        *state = candidate;
        for subscriber in self.subscribers.lock().expect("subscribers lock").iter() {
            subscriber(&events);
        }
        Ok(events)
    }

    /// Registers a callback for the events of every successful command. It is called under
    /// the app lock, in command order, so it must be quick (e.g. send on a channel) and must
    /// not call back into the app.
    pub fn subscribe(&self, subscriber: impl Fn(&[Event]) + Send + Sync + 'static) {
        self.subscribers
            .lock()
            .expect("subscribers lock")
            .push(Box::new(subscriber));
    }

    pub fn status(&self) -> AppStatus {
        let broken = match &*self.lock() {
            Inner::Ready { .. } => None,
            Inner::Broken { reason } => Some(reason.clone()),
        };
        AppStatus {
            broken,
            warnings: self.warnings.clone(),
        }
    }

    // Reads take the same lock, so they never see a half-applied command.

    pub fn catalog(&self) -> Result<Catalog, AppError> {
        self.read(|state| state.catalog().clone())
    }

    /// The whole state (tests and the tt-cli `state` command).
    pub fn snapshot(&self) -> Result<State, AppError> {
        self.read(State::clone)
    }

    /// The running entry with its names; its duration counts up to `ctx.now`.
    pub fn running(&self, ctx: &Ctx) -> Result<Option<EntryRow>, AppError> {
        self.read(|state| {
            state
                .entries()
                .running()
                .and_then(|e| read::entry_row(state.catalog(), e, ctx.now))
        })
    }

    /// One entry with its names, e.g. the one an overlap error names.
    pub fn entry(&self, ctx: &Ctx, id: EntryId) -> Result<Option<EntryRow>, AppError> {
        self.read(|state| {
            state
                .entries()
                .get(id)
                .and_then(|e| read::entry_row(state.catalog(), e, ctx.now))
        })
    }

    pub fn daily_overview(&self, ctx: &Ctx, date: civil::Date) -> Result<Overview, AppError> {
        let (from, to) = read::day_bounds(date, &ctx.tz)?;
        self.read(|state| {
            let entries = state.entries().started_between(from, to);
            read::daily_overview(state.catalog(), &entries, ctx.now, &ctx.tz, date)
        })
    }

    pub fn day_entries(&self, ctx: &Ctx, date: civil::Date) -> Result<Vec<EntryRow>, AppError> {
        let (from, to) = read::day_bounds(date, &ctx.tz)?;
        self.read(|state| {
            let entries = state.entries().started_between(from, to);
            read::day_entries(state.catalog(), &entries, ctx.now, &ctx.tz, date)
        })
    }

    pub fn recents(&self) -> Result<Vec<ComboRow>, AppError> {
        self.read(|state| {
            let n = state.catalog().settings().recents_n as usize;
            read::recents(state.catalog(), state.entries(), n)
        })
    }

    pub fn quick_switch(&self, query: &str) -> Result<Vec<ComboRow>, AppError> {
        self.read(|state| {
            let recents =
                read::recent_combos(state.catalog(), state.entries(), read::QUICK_SWITCH_RECENCY);
            read::quick_switch(state.catalog(), &recents, query)
        })
    }

    /// The New project dialog's check while typing (R2c).
    pub fn project_name_check(
        &self,
        client: ClientId,
        name: &str,
        abbreviation: Option<&str>,
    ) -> Result<NameCheck<ProjectId>, AppError> {
        self.read(|state| read::project_name_check(state.catalog(), client, name, abbreviation))
    }

    /// The New client dialog's check while typing (R2i).
    pub fn client_name_check(
        &self,
        name: &str,
        abbreviation: Option<&str>,
    ) -> Result<NameCheck<ClientId>, AppError> {
        self.read(|state| read::client_name_check(state.catalog(), name, abbreviation))
    }

    /// The Archive view: archived clients and archived projects by client, with the projects'
    /// active periods (R2e, R2j).
    pub fn archived_projects(&self, ctx: &Ctx) -> Result<Vec<ArchivedClient>, AppError> {
        self.read(|state| read::archived_projects(state.catalog(), state.entries(), &ctx.tz))
    }

    pub fn task_matches(&self, project: ProjectId, text: &str) -> Result<TaskMatches, AppError> {
        self.read(|state| read::task_matches(state.catalog(), project, text))
    }

    fn read<T>(&self, f: impl FnOnce(&State) -> T) -> Result<T, AppError> {
        let inner = self.lock();
        match &*inner {
            Inner::Ready { state, .. } => Ok(f(state)),
            Inner::Broken { .. } => Err(self.broken_error(&inner)),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        // With panic = "abort" a panic never unwinds, so the lock can't be poisoned.
        self.inner.lock().expect("app lock")
    }

    fn broken_error(&self, inner: &Inner) -> AppError {
        match inner {
            Inner::Broken { reason } => AppError::Broken(reason.clone()),
            Inner::Ready { .. } => unreachable!("only called when broken"),
        }
    }
}
