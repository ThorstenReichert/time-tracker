//! Tauri commands: thin mappings from UI calls to tt-app. No business rules here. All
//! commands are `async`, so they run off the main thread. Write commands return nothing;
//! the UI refreshes on the `data-changed` event.

use std::path::Path;

use jiff::civil;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt as _;
use tt_core::{ClientId, Command, Ctx, EntryId, ItemRef, ProjectId, TaskId};

use crate::backend::{Backend, ctx};
use crate::dto::{
    ArchiveClientDto, AutostartDto, CatalogDto, ComboDto, EntryDto, NameCheckDto, OverviewDto,
    RuleDto, SettingsDefaultsDto, SettingsDto, StatusDto, TaskChoiceDto, TaskMatchesDto,
};
use crate::errors::UiError;
use crate::hotkey::{self, HotkeyStatus};

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
pub fn execute(backend: &Backend, ctx: &Ctx, command: Command) -> Res<()> {
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
pub async fn get_status(
    backend: State<'_, Backend>,
    hotkey: State<'_, HotkeyStatus>,
) -> Res<StatusDto> {
    let (broken, mut warnings) = match backend.app() {
        Ok(app) => {
            let status = app.status();
            (status.broken, status.warnings)
        }
        Err(e) => (Some(e.message), Vec::new()),
    };
    warnings.extend(hotkey.warning());
    Ok(StatusDto {
        broken,
        warnings,
        db_path: backend.db_path().to_owned(),
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
pub async fn create_client(
    backend: State<'_, Backend>,
    name: String,
    abbreviation: Option<String>,
    rule: RuleDto,
) -> Res<()> {
    let rule = parse_rule(rule)?;
    execute(
        &backend,
        &ctx(),
        Command::CreateClient {
            name,
            abbreviation,
            rule,
        },
    )
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    backend: State<'_, Backend>,
    id: u64,
    name: String,
    abbreviation: Option<String>,
    rule: RuleDto,
) -> Res<()> {
    let rule = parse_rule(rule)?;
    let id = ClientId::new(id);
    execute(
        &backend,
        &ctx(),
        Command::UpdateClient {
            id,
            name,
            abbreviation,
            rule,
        },
    )
}

#[tauri::command]
#[specta::specta]
pub async fn create_project(
    backend: State<'_, Backend>,
    client: u64,
    name: String,
    abbreviation: Option<String>,
) -> Res<()> {
    let client = ClientId::new(client);
    execute(
        &backend,
        &ctx(),
        Command::CreateProject {
            client,
            name,
            abbreviation,
        },
    )
}

/// Renames a project and/or sets its abbreviation (R24).
#[tauri::command]
#[specta::specta]
pub async fn update_project(
    backend: State<'_, Backend>,
    id: u64,
    name: String,
    abbreviation: Option<String>,
) -> Res<()> {
    let id = ProjectId::new(id);
    execute(
        &backend,
        &ctx(),
        Command::UpdateProject {
            id,
            name,
            abbreviation,
        },
    )
}

/// Archives a project (R2a): no new entries, not offered any more.
#[tauri::command]
#[specta::specta]
pub async fn archive_project(backend: State<'_, Backend>, id: u64) -> Res<()> {
    let id = ProjectId::new(id);
    execute(&backend, &ctx(), Command::ArchiveProject { id })
}

/// Makes an archived project active again, as it was (R2a, R2c).
#[tauri::command]
#[specta::specta]
pub async fn unarchive_project(backend: State<'_, Backend>, id: u64) -> Res<()> {
    let id = ProjectId::new(id);
    execute(&backend, &ctx(), Command::UnarchiveProject { id })
}

/// The New project dialog's check while typing (R2c).
#[tauri::command]
#[specta::specta]
pub async fn check_project_name(
    backend: State<'_, Backend>,
    client: u64,
    name: String,
    abbreviation: Option<String>,
) -> Res<NameCheckDto> {
    read(&backend, |app, _| {
        let check =
            app.project_name_check(ClientId::new(client), &name, abbreviation.as_deref())?;
        Ok(NameCheckDto::new(&check, &app.catalog()?, ItemRef::Project))
    })
}

/// Archives a client (R2g): its projects are no longer offered or bookable.
#[tauri::command]
#[specta::specta]
pub async fn archive_client(backend: State<'_, Backend>, id: u64) -> Res<()> {
    let id = ClientId::new(id);
    execute(&backend, &ctx(), Command::ArchiveClient { id })
}

/// Makes an archived client active again, its projects as they were (R2g, R2i).
#[tauri::command]
#[specta::specta]
pub async fn unarchive_client(backend: State<'_, Backend>, id: u64) -> Res<()> {
    let id = ClientId::new(id);
    execute(&backend, &ctx(), Command::UnarchiveClient { id })
}

/// The New client dialog's check while typing (R2i).
#[tauri::command]
#[specta::specta]
pub async fn check_client_name(
    backend: State<'_, Backend>,
    name: String,
    abbreviation: Option<String>,
) -> Res<NameCheckDto> {
    read(&backend, |app, _| {
        let check = app.client_name_check(&name, abbreviation.as_deref())?;
        Ok(NameCheckDto::new(&check, &app.catalog()?, ItemRef::Client))
    })
}

#[tauri::command]
#[specta::specta]
pub async fn create_task(
    backend: State<'_, Backend>,
    project: u64,
    name: String,
    abbreviation: Option<String>,
) -> Res<()> {
    let project = ProjectId::new(project);
    execute(
        &backend,
        &ctx(),
        Command::CreateTask {
            project,
            name,
            abbreviation,
        },
    )
}

/// Renames a task and/or sets its abbreviation (R24).
#[tauri::command]
#[specta::specta]
pub async fn update_task(
    backend: State<'_, Backend>,
    id: u64,
    name: String,
    abbreviation: Option<String>,
) -> Res<()> {
    let id = TaskId::new(id);
    execute(
        &backend,
        &ctx(),
        Command::UpdateTask {
            id,
            name,
            abbreviation,
        },
    )
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

/// Saves the settings. The bridge then registers the new hotkey (R6); a hotkey the plugin
/// can't read is rejected here, so it's never saved.
#[tauri::command]
#[specta::specta]
pub async fn update_settings(backend: State<'_, Backend>, settings: SettingsDto) -> Res<()> {
    if let Some(text) = settings.hotkey.as_deref()
        && !hotkey::is_valid(text)
    {
        return Err(UiError::rejected(format!(
            "\"{text}\" isn't a valid hotkey; write it like {}",
            hotkey::DEFAULT_HOTKEY
        )));
    }
    let settings = settings.into();
    execute(&backend, &ctx(), Command::UpdateSettings { settings })
}

/// The values unset settings fall back to.
#[tauri::command]
#[specta::specta]
pub async fn get_settings_defaults(backend: State<'_, Backend>) -> Res<SettingsDefaultsDto> {
    let backup_dir = tt_app::default_backup_dir(Path::new(backend.db_path()));
    Ok(SettingsDefaultsDto {
        hotkey: hotkey::DEFAULT_HOTKEY.to_owned(),
        backup_dir: backup_dir.display().to_string(),
    })
}

/// The autostart entry as Windows has it (R9).
#[tauri::command]
#[specta::specta]
pub async fn get_autostart(app: AppHandle) -> Res<AutostartDto> {
    let enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|e| UiError::system(format!("Reading the autostart entry failed: {e}")))?;
    Ok(AutostartDto {
        available: crate::AUTOSTART_AVAILABLE,
        enabled,
    })
}

/// Turns starting with Windows on or off (R9). The entry is the only record of it.
#[tauri::command]
#[specta::specta]
pub async fn set_autostart(app: AppHandle, enabled: bool) -> Res<()> {
    if !crate::AUTOSTART_AVAILABLE {
        return Err(UiError::rejected(
            "Autostart can only be changed in the installed app",
        ));
    }
    let autolaunch = app.autolaunch();
    let result = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|e| UiError::system(format!("Windows refused the change: {e}")))
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

/// The Archive view (R2e, R2j). Reads the archive, then the catalog for the names.
#[tauri::command]
#[specta::specta]
pub async fn get_archive(backend: State<'_, Backend>) -> Res<Vec<ArchiveClientDto>> {
    read(&backend, |app, ctx| {
        let archive = app.archive_view(ctx)?;
        Ok(ArchiveClientDto::list(&archive, &app.catalog()?))
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
