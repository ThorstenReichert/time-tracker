//! Tauri commands: thin mappings from UI calls to tt-app. No business rules here. All
//! commands are `async`, so they run off the main thread. Write commands return nothing;
//! the UI refreshes on the `data-changed` event.

use jiff::civil;
use tauri::State;
use tt_core::{ClientId, Command, Ctx, EntryId, ProjectId, TaskId};

use crate::backend::{Backend, ctx};
use crate::dto::{
    CatalogDto, ComboDto, EntryDto, OverviewDto, RuleDto, SettingsDto, StatusDto, TaskChoiceDto,
    TaskMatchesDto,
};
use crate::errors::UiError;

type Res<T> = Result<T, UiError>;

fn local_time(text: &str) -> Res<civil::DateTime> {
    text.parse()
        .map_err(|e| UiError::input(format!("not a local time {text:?}: {e}")))
}

fn date(text: &str) -> Res<civil::Date> {
    text.parse()
        .map_err(|e| UiError::input(format!("not a date {text:?}: {e}")))
}

fn parse_rule(dto: RuleDto) -> Res<tt_core::RoundingRule> {
    dto.to_rule()
        .ok_or_else(|| UiError::input(format!("unsupported rounding unit {}", dto.unit)))
}

/// Executes a command through the single write path.
fn execute(backend: &Backend, ctx: &Ctx, command: Command) -> Res<()> {
    let app = backend.app()?;
    app.execute(ctx, command)
        .map(|_| ())
        .map_err(|e| UiError::from_app(e, app, ctx))
}

/// Runs a read, mapping its error.
fn read<T>(
    backend: &Backend,
    f: impl FnOnce(&tt_app::App, &Ctx) -> Result<T, tt_app::AppError>,
) -> Res<T> {
    let app = backend.app()?;
    let ctx = ctx();
    f(app, &ctx).map_err(|e| UiError::from_app(e, app, &ctx))
}

#[tauri::command]
#[specta::specta]
pub async fn get_status(backend: State<'_, Backend>) -> Res<StatusDto> {
    Ok(match backend.app() {
        Ok(app) => {
            let status = app.status();
            StatusDto {
                broken: status.broken,
                warnings: status.warnings,
                db_path: backend.db_path().to_owned(),
            }
        }
        Err(e) => StatusDto {
            broken: Some(e.message),
            warnings: Vec::new(),
            db_path: backend.db_path().to_owned(),
        },
    })
}

/// Today's date in the OS time zone (YYYY-MM-DD).
#[tauri::command]
#[specta::specta]
pub async fn get_today() -> Res<String> {
    let ctx = ctx();
    Ok(tt_core::read::local_date(ctx.now, &ctx.tz).to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_catalog(backend: State<'_, Backend>) -> Res<CatalogDto> {
    read(&backend, |app, _| app.catalog()).map(|c| (&c).into())
}

#[tauri::command]
#[specta::specta]
pub async fn create_client(backend: State<'_, Backend>, name: String, rule: RuleDto) -> Res<()> {
    let rule = parse_rule(rule)?;
    execute(&backend, &ctx(), Command::CreateClient { name, rule })
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    backend: State<'_, Backend>,
    id: u64,
    name: String,
    rule: RuleDto,
) -> Res<()> {
    let rule = parse_rule(rule)?;
    let id = ClientId::new(id);
    execute(&backend, &ctx(), Command::UpdateClient { id, name, rule })
}

#[tauri::command]
#[specta::specta]
pub async fn create_project(backend: State<'_, Backend>, client: u64, name: String) -> Res<()> {
    let client = ClientId::new(client);
    execute(&backend, &ctx(), Command::CreateProject { client, name })
}

#[tauri::command]
#[specta::specta]
pub async fn rename_project(backend: State<'_, Backend>, id: u64, name: String) -> Res<()> {
    let id = ProjectId::new(id);
    execute(&backend, &ctx(), Command::RenameProject { id, name })
}

#[tauri::command]
#[specta::specta]
pub async fn create_task(backend: State<'_, Backend>, project: u64, name: String) -> Res<()> {
    let project = ProjectId::new(project);
    execute(&backend, &ctx(), Command::CreateTask { project, name })
}

#[tauri::command]
#[specta::specta]
pub async fn rename_task(backend: State<'_, Backend>, id: u64, name: String) -> Res<()> {
    let id = TaskId::new(id);
    execute(&backend, &ctx(), Command::RenameTask { id, name })
}

/// Starts a timer on the combo, or switches to it (R5).
#[tauri::command]
#[specta::specta]
pub async fn start(backend: State<'_, Backend>, project: u64, task: TaskChoiceDto) -> Res<()> {
    let project = ProjectId::new(project);
    let task = task.into();
    execute(&backend, &ctx(), Command::Start { project, task })
}

/// Stops the running timer now, or at a typed local time (R11).
#[tauri::command]
#[specta::specta]
pub async fn stop(backend: State<'_, Backend>, at: Option<String>) -> Res<()> {
    let at = at.as_deref().map(local_time).transpose()?;
    execute(&backend, &ctx(), Command::Stop { at })
}

#[tauri::command]
#[specta::specta]
pub async fn add_entry(
    backend: State<'_, Backend>,
    project: u64,
    task: TaskChoiceDto,
    start: String,
    end: String,
    note: Option<String>,
) -> Res<()> {
    let command = Command::AddEntry {
        project: ProjectId::new(project),
        task: task.into(),
        start: local_time(&start)?,
        end: local_time(&end)?,
        note,
    };
    execute(&backend, &ctx(), command)
}

#[tauri::command]
#[specta::specta]
pub async fn edit_entry(
    backend: State<'_, Backend>,
    id: u64,
    project: u64,
    task: TaskChoiceDto,
    start: String,
    end: Option<String>,
    note: Option<String>,
) -> Res<()> {
    let command = Command::EditEntry {
        id: EntryId::new(id),
        project: ProjectId::new(project),
        task: task.into(),
        start: local_time(&start)?,
        end: end.as_deref().map(local_time).transpose()?,
        note,
    };
    execute(&backend, &ctx(), command)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_entry(backend: State<'_, Backend>, id: u64) -> Res<()> {
    let id = EntryId::new(id);
    execute(&backend, &ctx(), Command::DeleteEntry { id })
}

#[tauri::command]
#[specta::specta]
pub async fn update_settings(backend: State<'_, Backend>, settings: SettingsDto) -> Res<()> {
    let settings = settings.into();
    execute(&backend, &ctx(), Command::UpdateSettings { settings })
}

#[tauri::command]
#[specta::specta]
pub async fn get_running(backend: State<'_, Backend>) -> Res<Option<EntryDto>> {
    read(&backend, |app, ctx| {
        Ok(app.running(ctx)?.map(|r| EntryDto::new(&r, &ctx.tz)))
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_day_entries(backend: State<'_, Backend>, day: String) -> Res<Vec<EntryDto>> {
    let day = date(&day)?;
    read(&backend, |app, ctx| {
        Ok(app
            .day_entries(ctx, day)?
            .iter()
            .map(|r| EntryDto::new(r, &ctx.tz))
            .collect())
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_overview(backend: State<'_, Backend>, day: String) -> Res<OverviewDto> {
    let day = date(&day)?;
    read(&backend, |app, ctx| {
        Ok((&app.daily_overview(ctx, day)?).into())
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_recents(backend: State<'_, Backend>) -> Res<Vec<ComboDto>> {
    read(&backend, |app, _| {
        Ok(app.recents()?.iter().map(Into::into).collect())
    })
}

#[tauri::command]
#[specta::specta]
pub async fn quick_switch(backend: State<'_, Backend>, query: String) -> Res<Vec<ComboDto>> {
    read(&backend, |app, _| {
        Ok(app.quick_switch(&query)?.iter().map(Into::into).collect())
    })
}

#[tauri::command]
#[specta::specta]
pub async fn task_matches(
    backend: State<'_, Backend>,
    project: u64,
    text: String,
) -> Res<TaskMatchesDto> {
    read(&backend, |app, _| {
        let matches = app.task_matches(ProjectId::new(project), &text)?;
        Ok(TaskMatchesDto::new(&matches, project))
    })
}
