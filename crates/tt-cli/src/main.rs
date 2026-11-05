//! Headless driver for scenario tests (OQ-5). Requires `--db <path>` and never opens the
//! live database. Like every driver it only translates: arguments → `Command` or read, and
//! the result → JSON on stdout. Exit codes: 0 ok, 1 other error, 2 usage (clap), 3 rejected
//! by a business rule (JSON on stderr).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use jiff::{Timestamp, civil, tz::TimeZone};
use serde_json::{Value, json};
use tt_app::{App, AppConfig, AppError};
use tt_core::{
    ClientId, Command, Ctx, DecimalSeparator, EntryId, Instant, ProjectId, RoundingDirection,
    RoundingLevel, RoundingRule, RoundingUnit, TaskChoice, TaskId, read, to_instant,
};

#[derive(Parser)]
#[command(
    name = "tt-cli",
    about = "Drive the time tracker headless, on a scratch database"
)]
struct Cli {
    /// The scratch database to use (created if missing). The live database is refused.
    #[arg(long, value_name = "PATH")]
    db: PathBuf,
    /// The current time: a local time like 2026-10-17T09:00 (in --tz) or RFC 3339 with an
    /// offset. Default: the system clock.
    #[arg(long, value_name = "TIME")]
    now: Option<String>,
    /// The time zone, as an IANA name like Europe/Berlin. Default: the system zone.
    #[arg(long, value_name = "IANA")]
    tz: Option<String>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(subcommand)]
    Client(ClientCmd),
    #[command(subcommand)]
    Project(ProjectCmd),
    #[command(subcommand)]
    Task(TaskCmd),
    /// Start a timer (or switch to it) at --now.
    Start {
        #[arg(long)]
        project: u64,
        #[command(flatten)]
        task: TaskArgs,
    },
    /// Stop the running timer at --now, or at a typed local time.
    Stop {
        #[arg(long, value_name = "LOCAL")]
        at: Option<civil::DateTime>,
    },
    #[command(subcommand)]
    Entry(EntryCmd),
    /// The entries that start on a day (default: today).
    Entries {
        #[arg(long)]
        date: Option<civil::Date>,
    },
    /// The daily billing overview (default: today).
    Overview {
        #[arg(long)]
        date: Option<civil::Date>,
    },
    /// The recent combos.
    Recents,
    /// Quick-switch matches for a query.
    QuickSwitch { query: String },
    #[command(subcommand)]
    Settings(SettingsCmd),
    /// The whole state as JSON.
    State,
}

#[derive(Subcommand)]
enum ClientCmd {
    Add {
        name: String,
        /// An optional abbreviation (R24).
        #[arg(long)]
        abbr: Option<String>,
        #[command(flatten)]
        rule: RuleArgs,
    },
    /// Rename a client, set its abbreviation or change its rule; omitted options keep their
    /// current value, and `--abbr ""` removes the abbreviation.
    Edit {
        id: u64,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        abbr: Option<String>,
        #[command(flatten)]
        rule: RuleArgs,
    },
}

#[derive(Args)]
struct RuleArgs {
    /// Rounding unit in minutes: 6, 12, 15, 30 or 60.
    #[arg(long)]
    unit: Option<u32>,
    /// up, nearest or down.
    #[arg(long)]
    direction: Option<String>,
    /// project or client.
    #[arg(long)]
    level: Option<String>,
}

#[derive(Subcommand)]
enum ProjectCmd {
    Add {
        #[arg(long)]
        client: u64,
        name: String,
        /// An optional abbreviation (R24).
        #[arg(long)]
        abbr: Option<String>,
    },
    /// Rename a project or set its abbreviation; omitted options keep their current value,
    /// and `--abbr ""` removes the abbreviation.
    Edit {
        id: u64,
        #[command(flatten)]
        naming: NamingArgs,
    },
    /// Archive a project: no new entries, not offered any more (R2a).
    Archive { id: u64 },
    /// Make an archived project active again (R2a).
    Unarchive { id: u64 },
}

#[derive(Subcommand)]
enum TaskCmd {
    Add {
        #[arg(long)]
        project: u64,
        name: String,
        /// An optional abbreviation (R24).
        #[arg(long)]
        abbr: Option<String>,
    },
    /// Rename a task or set its abbreviation; omitted options keep their current value,
    /// and `--abbr ""` removes the abbreviation.
    Edit {
        id: u64,
        #[command(flatten)]
        naming: NamingArgs,
    },
}

#[derive(Args)]
struct NamingArgs {
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    abbr: Option<String>,
}

impl NamingArgs {
    /// The name and abbreviation to send: the given ones, else the current ones.
    fn resolve(
        self,
        name: &tt_core::Name,
        abbreviation: Option<&tt_core::Abbreviation>,
    ) -> (String, Option<String>) {
        (
            self.name.unwrap_or_else(|| name.to_string()),
            self.abbr.or_else(|| abbreviation.map(ToString::to_string)),
        )
    }
}

#[derive(Args)]
#[group(multiple = false)]
struct TaskArgs {
    /// An existing task of the project.
    #[arg(long)]
    task: Option<u64>,
    /// Create the task and use it (R3).
    #[arg(long, value_name = "NAME")]
    new_task: Option<String>,
}

#[derive(Subcommand)]
enum EntryCmd {
    /// Add a finished entry after the fact.
    Add {
        #[arg(long)]
        project: u64,
        #[command(flatten)]
        task: TaskArgs,
        #[arg(long, value_name = "LOCAL")]
        start: civil::DateTime,
        #[arg(long, value_name = "LOCAL")]
        end: civil::DateTime,
        #[arg(long)]
        note: Option<String>,
    },
    /// Replace an entry's combo, times and note. Without --end the running entry keeps
    /// running.
    Edit {
        id: u64,
        #[arg(long)]
        project: u64,
        #[command(flatten)]
        task: TaskArgs,
        #[arg(long, value_name = "LOCAL")]
        start: civil::DateTime,
        #[arg(long, value_name = "LOCAL")]
        end: Option<civil::DateTime>,
        #[arg(long)]
        note: Option<String>,
    },
    Delete {
        id: u64,
    },
}

#[derive(Subcommand)]
enum SettingsCmd {
    /// Change settings; omitted options keep their current value.
    Set {
        #[arg(long)]
        hotkey: Option<String>,
        /// auto, dot or comma.
        #[arg(long)]
        decimal_separator: Option<String>,
        #[arg(long)]
        recents_n: Option<u32>,
        #[arg(long)]
        backup_dir: Option<String>,
        #[arg(long)]
        backup_keep: Option<u32>,
    },
}

/// A failure, with the exit code it maps to.
enum Failure {
    Domain(tt_core::DomainError),
    Other(String),
}

impl From<AppError> for Failure {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Domain(d) => Failure::Domain(d),
            other => Failure::Other(other.to_string()),
        }
    }
}

impl From<tt_core::DomainError> for Failure {
    fn from(e: tt_core::DomainError) -> Self {
        Failure::Domain(e)
    }
}

fn other(message: impl Into<String>) -> Failure {
    Failure::Other(message.into())
}

fn main() -> ExitCode {
    let cli = Cli::parse(); // usage errors exit with 2
    match run(cli) {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output).expect("JSON"));
            ExitCode::SUCCESS
        }
        Err(Failure::Domain(e)) => {
            let error =
                json!({ "error": "domain", "message": e.to_string(), "detail": format!("{e:?}") });
            eprintln!("{error}");
            ExitCode::from(3)
        }
        Err(Failure::Other(message)) => {
            eprintln!("{}", json!({ "error": "other", "message": message }));
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<Value, Failure> {
    // Before touching any file: never the live database (OQ-5).
    refuse_live_db(&cli.db)?;
    let tz = match &cli.tz {
        Some(name) => TimeZone::get(name).map_err(|e| other(format!("--tz {name}: {e}")))?,
        None => TimeZone::system(),
    };
    let now = match &cli.now {
        Some(text) => parse_now(text, &tz)?,
        None => Instant::from(Timestamp::now()),
    };
    let ctx = Ctx::new(now, tz);

    let backups = tt_app::default_backup_dir(&cli.db);
    let config = AppConfig {
        backup_dir_override: Some(backups),
        ..AppConfig::new(&cli.db)
    };
    let app = App::open(config, &ctx)?;
    if let Some(reason) = app.status().broken {
        return Err(other(reason));
    }
    let today = || read::local_date(ctx.now, &ctx.tz);

    let command = match cli.command {
        Cmd::Client(ClientCmd::Add { name, abbr, rule }) => Command::CreateClient {
            name,
            abbreviation: abbr,
            rule: rule.apply(RoundingRule::default())?,
        },
        Cmd::Client(ClientCmd::Edit {
            id,
            name,
            abbr,
            rule,
        }) => {
            let id = ClientId::new(id);
            let catalog = app.catalog()?;
            let current = catalog
                .client(id)
                .ok_or(tt_core::DomainError::UnknownClient(id))?;
            let (name, abbreviation) =
                NamingArgs { name, abbr }.resolve(&current.name, current.abbreviation.as_ref());
            Command::UpdateClient {
                id,
                name,
                abbreviation,
                rule: rule.apply(current.rule)?,
            }
        }
        Cmd::Project(ProjectCmd::Add { client, name, abbr }) => Command::CreateProject {
            client: ClientId::new(client),
            name,
            abbreviation: abbr,
        },
        Cmd::Project(ProjectCmd::Edit { id, naming }) => {
            let id = ProjectId::new(id);
            let catalog = app.catalog()?;
            let current = catalog
                .project(id)
                .ok_or(tt_core::DomainError::UnknownProject(id))?;
            let (name, abbreviation) = naming.resolve(&current.name, current.abbreviation.as_ref());
            Command::UpdateProject {
                id,
                name,
                abbreviation,
            }
        }
        Cmd::Project(ProjectCmd::Archive { id }) => Command::ArchiveProject {
            id: ProjectId::new(id),
        },
        Cmd::Project(ProjectCmd::Unarchive { id }) => Command::UnarchiveProject {
            id: ProjectId::new(id),
        },
        Cmd::Task(TaskCmd::Add {
            project,
            name,
            abbr,
        }) => Command::CreateTask {
            project: ProjectId::new(project),
            name,
            abbreviation: abbr,
        },
        Cmd::Task(TaskCmd::Edit { id, naming }) => {
            let id = TaskId::new(id);
            let catalog = app.catalog()?;
            let current = catalog
                .task(id)
                .ok_or(tt_core::DomainError::UnknownTask(id))?;
            let (name, abbreviation) = naming.resolve(&current.name, current.abbreviation.as_ref());
            Command::UpdateTask {
                id,
                name,
                abbreviation,
            }
        }
        Cmd::Start { project, task } => Command::Start {
            project: ProjectId::new(project),
            task: task.choice(),
        },
        Cmd::Stop { at } => Command::Stop { at },
        Cmd::Entry(EntryCmd::Add {
            project,
            task,
            start,
            end,
            note,
        }) => Command::AddEntry {
            project: ProjectId::new(project),
            task: task.choice(),
            start,
            end,
            note,
        },
        Cmd::Entry(EntryCmd::Edit {
            id,
            project,
            task,
            start,
            end,
            note,
        }) => Command::EditEntry {
            id: EntryId::new(id),
            project: ProjectId::new(project),
            task: task.choice(),
            start,
            end,
            note,
        },
        Cmd::Entry(EntryCmd::Delete { id }) => Command::DeleteEntry {
            id: EntryId::new(id),
        },
        Cmd::Settings(SettingsCmd::Set {
            hotkey,
            decimal_separator,
            recents_n,
            backup_dir,
            backup_keep,
        }) => {
            let mut settings = app.catalog()?.settings().clone();
            if hotkey.is_some() {
                settings.hotkey = hotkey;
            }
            if let Some(sep) = decimal_separator {
                settings.decimal_separator = DecimalSeparator::parse(&sep)
                    .ok_or_else(|| other(format!("--decimal-separator {sep}")))?;
            }
            if let Some(n) = recents_n {
                settings.recents_n = n;
            }
            if backup_dir.is_some() {
                settings.backup_dir = backup_dir;
            }
            if let Some(keep) = backup_keep {
                settings.backup_keep = keep;
            }
            Command::UpdateSettings { settings }
        }
        // Reads
        Cmd::Entries { date } => {
            return Ok(to_json(&app.day_entries(&ctx, date.unwrap_or_else(today))?));
        }
        Cmd::Overview { date } => {
            return Ok(to_json(
                &app.daily_overview(&ctx, date.unwrap_or_else(today))?,
            ));
        }
        Cmd::Recents => return Ok(to_json(&app.recents()?)),
        Cmd::QuickSwitch { query } => return Ok(to_json(&app.quick_switch(&query)?)),
        Cmd::State => return Ok(to_json(&app.snapshot()?)),
    };
    let events = app.execute(&ctx, command)?;
    Ok(json!({ "events": events }))
}

fn to_json(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("read models serialise")
}

impl RuleArgs {
    fn apply(self, base: RoundingRule) -> Result<RoundingRule, Failure> {
        let mut rule = base;
        if let Some(unit) = self.unit {
            rule.unit = RoundingUnit::from_minutes(unit)
                .ok_or_else(|| other(format!("--unit {unit}: use 6, 12, 15, 30 or 60")))?;
        }
        if let Some(d) = self.direction {
            rule.direction = RoundingDirection::parse(&d)
                .ok_or_else(|| other(format!("--direction {d}: use up, nearest or down")))?;
        }
        if let Some(l) = self.level {
            rule.level = RoundingLevel::parse(&l)
                .ok_or_else(|| other(format!("--level {l}: use project or client")))?;
        }
        Ok(rule)
    }
}

impl TaskArgs {
    fn choice(self) -> TaskChoice {
        match (self.task, self.new_task) {
            (Some(id), _) => TaskChoice::Existing(TaskId::new(id)),
            (None, Some(name)) => TaskChoice::New(name),
            (None, None) => TaskChoice::NoTask,
        }
    }
}

/// RFC 3339 with an offset, or a local wall-clock time in `tz` (rejected in a DST gap/fold).
fn parse_now(text: &str, tz: &TimeZone) -> Result<Instant, Failure> {
    if let Ok(ts) = text.parse::<Timestamp>() {
        return Ok(Instant::from(ts));
    }
    let local: civil::DateTime = text
        .parse()
        .map_err(|e| other(format!("--now {text}: {e}")))?;
    Ok(to_instant(tz, local)?)
}

fn refuse_live_db(db: &Path) -> Result<(), Failure> {
    let Some(live) = tt_app::live_db_path() else {
        return Ok(());
    };
    if same_path(db, &live) {
        return Err(other(format!(
            "refusing to open the live database {}; tt-cli only runs on scratch databases",
            live.display()
        )));
    }
    Ok(())
}

/// Whether `a` and `b` name the same file. First the absolute paths, ignoring case (Windows
/// file names are case-insensitive) and a verbatim `\\?\` prefix. Then, for other spellings
/// (8.3 short names, links), the canonical paths of the files, or of their folders if the
/// files don't exist yet. Canonicalising only reads metadata.
fn same_path(a: &Path, b: &Path) -> bool {
    let (a, b) = (absolute(a), absolute(b));
    if text(&a) == text(&b) {
        return true;
    }
    let canonical = |p: &Path| std::fs::canonicalize(p).ok().map(|c| text(&c));
    if let (Some(a), Some(b)) = (canonical(&a), canonical(&b)) {
        return a == b;
    }
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().to_lowercase());
    let folder = |p: &Path| p.parent().and_then(canonical);
    name(&a) == name(&b) && folder(&a).is_some() && folder(&a) == folder(&b)
}

/// `p` as an absolute path without a verbatim prefix (`\\?\C:\…`, `\\?\UNC\server\…`).
fn absolute(p: &Path) -> PathBuf {
    let s = p.to_string_lossy().replace('/', "\\");
    let s = match s.strip_prefix(r"\\?\") {
        Some(rest) => match rest.strip_prefix(r"UNC\") {
            Some(unc) => format!(r"\\{unc}"),
            None => rest.to_string(),
        },
        None => s,
    };
    let p = PathBuf::from(s);
    std::path::absolute(&p).unwrap_or(p)
}

fn text(p: &Path) -> String {
    absolute(p).to_string_lossy().to_lowercase()
}
