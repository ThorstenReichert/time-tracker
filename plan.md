# Plan: Personal project time tracker — slice 0 + slice 1
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude). Status: approved.
Approved by: Thorsten Reichert (owner). Date: 2026-10-17.
Scope: **slice 0** (skeleton + installer check on the work machine, AC-2) and **slice 1** (thin
vertical slice from AC-6, then the pieces needed for daily use). In scope: R1, R3, R4, R5, R6,
R7 (recents only), R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R22 (startup
backup + prune + folder setting). Deferred to a later plan.md (slice 2+): R2 (archiving
commands/UI; the schema columns exist from step 5), R7 favorites, R20, R21, R22 daily timer, R23.

## Context
The repo has intent.md (approved) and spec.md (approved at the start of this stage, owner
Thorsten Reichert) but no code. The spec fixes the stack (Rust + Tauri 2, Vue 3 + TS, SQLite)
and the architecture (pure Decider core, single write path, headless CLI). AC-6 asks for a thin
vertical slice first, and AC-2 asks to check the installer on the work machine before any
features. This plan does both, and ends with an app that can replace Clockify for daily
tracking. Statistics, billing error, archiving, favorites and export/restore come in slice 2.
The draft was reviewed adversarially before approval, and its findings are folded in.

## Decisions made while planning
| Topic | Decision | Rejected alternatives (why) |
|---|---|---|
| Time library | `jiff` 0.2 (`serde` feature). tt-core stores instants as its own `Instant(i64)` (Unix seconds, matches the DB). Typed local times are `jiff::civil::DateTime` (seconds; a non-zero subsecond part is rejected) and are converted with `Ctx.tz`. A `Gap` or `Fold` offset is rejected (OQ-6). | chrono + chrono-tz (clunkier API; the OS zone needs `iana-time-zone`) |
| R12 end time | **Decided (owner, 2026-10-17):** an added or edited end must also be ≤ `now`, not only the start. spec.md R12 gets this sentence in the approval commit. | Allowing a future end (it would block starting a timer until then) |
| Tauri | Tauri **2.12.x**, features `tray-icon`, `image-png`. Plugins: single-instance 2.5 (registered **first**), global-shortcut 2.4, autostart 2.7, clipboard-manager 2.5. Tauri CLI: `cargo install tauri-cli --version "^2.12" --locked`, run from the repo root. All Tauri commands are `async`, so they run off the main thread. | Tauri 3.0 alpha (pre-release; the spec says Tauri 2); `@tauri-apps/cli` via npm (`ui/` isn't the Tauri app dir) |
| Rust↔TS types | `tauri-specta` and `specta` **=2.0.0-rc.25**, `specta-typescript` 0.0.12, all pinned exactly. Export with `Typescript::default().bigint(BigIntExportBehavior::Number)` (IDs and Unix seconds stay below 2^53). One shared `specta_builder()`. The test `bindings_up_to_date` compares against `ui/src/bindings.ts`; regenerate with `UPDATE_BINDINGS=1 cargo test -p timetracker-app bindings`. | `ts-rs` (fallback if rc.25 breaks: types only, command wrappers written by hand) |
| Fuzzy matching (R3, R6) | `nucleo-matcher` 0.3 inside tt-core (pure, no IO). Ranking: score descending, then recency, then label. | `fuzzy-matcher` (unmaintained); hand-written (worse quality, weaker Unicode) |
| SQLite | `rusqlite` 0.40 with `bundled`, `backup`. Own migration runner with a `schema_migrations` table (as in the spec); `pub static MIGRATIONS: &[Migration]`, SQL via `include_str!`. Pragmas `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=FULL`. | `rusqlite_migration` (uses `user_version`, not the spec's table) |
| Single-writer lock | std `File::try_lock()` (stable since Rust 1.89) on `<db>.lock`, held for the process lifetime | `fs4` (its traits clash with the std methods); SQLite `locking_mode=EXCLUSIVE` (doesn't cover the backup step) |
| Start vs switch (R5) | One command `Start { project, task }`. No timer → start. A different combo running → switch. The same combo running → no-op (`[]`). Event order is fixed: `[EntryStopped \| EntryDeleted, TaskCreated?, EntryStarted]`. `Start` checks `overlapping(now, ∞)` against completed entries. `now < running.start` (clock went back) → `EndNotAfterStart`. | Separate `Start`/`Switch` (the UI and hotkey would have to know the state first) |
| Zero-length running entry | Stop with `at: None`, or a switch, when `running.start == now` → the entry is discarded (`EntryDeleted`) instead of saved as 0 s. `Stop { at: Some(t) }` with `t ≤ start` → `EndNotAfterStart` (a typed time is never adjusted). | Reject (breaks a quick double switch from the hotkey) |
| Quick task creation (R3) | `TaskChoice = None \| Existing(TaskId) \| New(Name)` in `Start`, `AddEntry`, `EditEntry`. `New` emits `TaskCreated` plus the entry event in one command, so it's atomic. | Two commands from the UI (not atomic, two round trips) |
| Editing entries (R10, R11) | One `EditEntry { id, project, task, start, end: Option<civil::DateTime>, note }`. On the running entry, `end: None` keeps it running (this covers "edit the running start", R11). On a completed entry, `end: None` → `EndRequired`. | A separate `EditRunningStart` (duplicates validation) |
| Events | **Every event carries every column it writes** (self-contained; tt-store never reads State), e.g. `EntryStopped { id, end, updated_at }`. Counters are derived from the IDs in the events (`next = id + 1`). | Store looking up rows (breaks OQ-9) |
| Delete (R10) | Confirmation dialog (PrimeVue ConfirmDialog). No undo in slice 1. | Undo as an inverse command (more work; R10 allows either) |
| Backups (R22) | The default folder is always `<db dir>/backups` (release: `%LOCALAPPDATA%\ThorSoft.TimeTracker\backups`, as in the spec). The folder is read before migrations: `tt_store::raw_setting(path, "backup_dir") -> Option<String>` (missing table/key → `None`). tt-app then falls back to `tt_core::Settings::default()`; no second set of defaults. If a backup fails and a migration is pending → Broken. If none is pending → open, with a warning banner. A file name that already exists is overwritten. tt-cli always backs up to `<db dir>/backups`. | Backing up after load (breaks "backup before migrations") |
| Dev vs installed app | `src-tauri/tauri.dev.conf.json` sets identifier `de.thorsoft.timetracker.dev` and productName `ThorSoft TimeTracker (dev)`; run with `cargo tauri dev --config src-tauri/tauri.dev.conf.json`. Debug builds use `%LOCALAPPDATA%\ThorSoft.TimeTracker\dev\timetracker.db`, the default hotkey `Ctrl+Alt+Shift+Space`, never auto-enable autostart, and honour a debug-only `TT_DB` env override. | One identity (single-instance, hotkey and autostart clash with the installed app) |
| Live DB path | `tt_app::live_db_path()` (from `LOCALAPPDATA`), used by both drivers. tt-cli refuses a `--db` whose normalised path equals it, before touching any file. | Path logic per driver |
| Decimal separator (R15) | Setting `auto \| dot \| comma`, default `auto`. `auto` resolves in the UI via `Intl.NumberFormat().formatToParts`. Hours are formatted in the UI only, with integer math (`hundredths = (secs·100 + 1800) div 3600`, half away from zero for negatives). | Detecting the locale in Rust (wrong if the OS locale changes) |
| Installer | NSIS only, `bundle.windows.nsis.installMode: "currentUser"` (no admin), unsigned | MSI (Tauri's MSI installs per machine and needs admin) |
| Frontend | npm; vue 3.5, vue-router 5.4, pinia 4.1, **primevue 4.5.x** + `@primeuix/themes` 2.x (Aura), vite 8.3, vitest 5, vue-tsc 3.3, **typescript 5.9**. ECharts comes in slice 2. | PrimeVue 5 (new license-manager dependency); TypeScript 7 (the Go port, which vue-tsc doesn't support) |
| Rust edition / toolchain | Edition 2024; `rust-toolchain.toml` pinned to `1.96.1` with rustfmt and clippy | Floating `stable` |
| Panics (bugs) | **Crash-only (owner, 2026-10-17):** `panic = "abort"` in `[profile.dev]` and `[profile.release]` (Cargo ignores it for tests, so proptest shrinking still works). No `catch_unwind` and no poison recovery: a bug ends the process; nothing was written, because evolve runs on a candidate, and the restart reloads from the DB with an invariant check. The running entry survives (it's stored with no end). | `catch_unwind` → `AppError::Bug` (more code, keeps a buggy process alive); default unwinding (UI call hangs, poisoned lock) |
| Purity guard | `crates/tt-core/clippy.toml`: `disallowed-methods` = `jiff::Timestamp::now`, `jiff::Zoned::now`, `jiff::tz::TimeZone::system`, `jiff::tz::TimeZone::get`, `jiff::tz::db`, `std::time::SystemTime::now`, `std::time::Instant::now`, `std::env::var`; `disallowed-types` = `std::collections::HashMap`, `std::collections::HashSet`, `std::fs::File`. `#[allow]` only in `testing.rs`. tt-core deps: `serde`, `jiff`, `thiserror`, `nucleo-matcher` (+ optional `proptest`). | Review only (easy to miss) |
| Test tools | `cargo test`, `proptest` 1.12, `assert_cmd` 2.2, `tempfile` 3, `memory-stats` (benchmark only). The tt-core feature `test-support` exports the random command generator, so tt-app reuses it. | cargo-nextest, insta (not needed yet) |
| Lint | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `vue-tsc --noEmit`. No ESLint in slice 1. | ESLint/Prettier now (more config, small gain for one person) |
| CSP | `default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost`, plus `app.security.dangerousDisableAssetCspModification: ["style-src"]` (otherwise Tauri's nonces disable `'unsafe-inline'` and PrimeVue's runtime styles break) | Strict style-src (breaks PrimeVue styled mode) |
| Git | Spec approval + `plan.md` committed on `master`; the build happens on branch `slice-1`, one commit per verified step. No push (no remote), no merge (stage 5). | — |

## Pinned names and shapes
**tt-core**
- `Ctx { now: Instant, tz: jiff::tz::TimeZone }`. ID newtypes over `u64`: `ClientId`,
  `ProjectId`, `TaskId`, `EntryId`. `Name`: trimmed, 1–100 chars, key = Unicode lowercase.
  This is stricter than SQLite `NOCASE`, which only folds ASCII, so the backstop never
  rejects what the core accepted. Renaming to a case variant of the item's own name is
  allowed. `Note`: ≤ 2000 chars.
- `RoundingRule { unit: RoundingUnit (M6|M12|M15|M30|M60), direction: Up|Nearest|Down,
  level: Project|Client }`, default 15 / nearest / project.
  `rounding::round(secs: i64, rule: &RoundingRule) -> i64`.
- `State { catalog: Catalog, entries: Entries }`. `Catalog` holds the client, project and task
  `BTreeMap`s, `Settings` (hotkey, decimal separator, recents_n = 8, backup_dir:
  Option<String>, backup_keep = 14) and `Counters`. `Entries` is private, with the index
  `BTreeSet<(Instant, EntryId)>` and `running: Option<EntryId>`. Named queries: `running()`,
  `get(id)`, `overlapping(start, end: Option<Instant>) -> Vec<&Entry>`,
  `started_between(from, to) -> Vec<&Entry>`, `recent_combos(n) -> Vec<Combo>`. Entities
  carry `created_at` (entries also `updated_at`), taken from `Ctx.now` in `decide`.
- `Command`: `CreateClient { name, rule }`, `UpdateClient { id, name, rule }`,
  `CreateProject { client, name }`, `RenameProject { id, name }`, `CreateTask { project,
  name }`, `RenameTask { id, name }`, `Start { project, task: TaskChoice }`, `Stop { at:
  Option<civil::DateTime> }`, `AddEntry { project, task, start, end, note }`, `EditEntry {…}`,
  `DeleteEntry { id }`, `UpdateSettings { settings }`.
- `Event`: `ClientCreated`, `ClientUpdated`, `ProjectCreated`, `ProjectRenamed`,
  `TaskCreated`, `TaskRenamed`, `EntryStarted`, `EntryStopped { id, end, updated_at }`,
  `EntryAdded`, `EntryUpdated`, `EntryDeleted { id }`, `SettingsChanged`. Each carries the
  full row it writes.
- `DomainError` (thiserror): `DuplicateName { existing }`, `Overlap { with: EntryId, start,
  end: Option<Instant> }`, `StartInFuture`, `EndInFuture`, `EndNotAfterStart`, `EndRequired`,
  `AmbiguousLocalTime`, `NonexistentLocalTime`, `SubsecondTime`, `NotRunning`,
  `TaskNotInProject`, `UnknownClient|Project|Task|Entry`, `InvalidName`, `NoteTooLong`.
- Read models (take the entries they need, never `State`):
  `read::day_bounds(civil::Date, &TimeZone) -> (Instant, Instant)`;
  `read::daily_overview(&Catalog, &[&Entry], now: Instant, &TimeZone, civil::Date) -> Overview`,
  with `Overview { date, clients: Vec<ClientBlock> }`, `ClientBlock { client, name, level,
  total: Figure, projects: Vec<ProjectRow>, running: bool }`, `ProjectRow { project, name,
  raw: i64, booked: Option<Figure>, running: bool }` (`booked` is `None` at client level), and
  `Figure { raw, rounded, diff }` in seconds. At project level, `total.rounded` is the sum of
  the rounded project values. Clients are sorted by name key, then projects.
  `read::day_entries(&Catalog, &[&Entry], now, &TimeZone, civil::Date) -> Vec<EntryRow>`;
  `read::recents(&Catalog, &[Combo], n) -> Vec<ComboRow>` (active only);
  `read::quick_switch(&Catalog, recents: &[Combo], query: &str) -> Vec<ComboRow>` (empty query
  → recents, otherwise fuzzy over "Client / Project / Task" and project-only combos of active
  items); `read::task_matches(&Catalog, ProjectId, text: &str) -> TaskMatches { tasks,
  offer_create: Option<Name> }`.

**tt-store**: `Store::open(path, migrations: &[Migration]) -> Result<(Store, OpenInfo {
existed, pending_migrations })>`, `migrate()`, `load() -> State`, `apply(&[Event])` (one
transaction), `backup_to(dir, now: Instant, keep: usize) -> PathBuf` (file
`timetracker-YYYYMMDDTHHMMSSZ.db`), `raw_setting(path, key)`, `DbLock::acquire(path)`.

**tt-app**: `AppConfig { db_path, backup_dir_override: Option<PathBuf>, migrations: &'static
[Migration] }` (default `tt_store::MIGRATIONS`). `App::open(AppConfig, &Ctx) -> Result<App,
AppError>`. `execute(&Ctx, Command) -> Result<Vec<Event>, AppError>`. Reads:
`daily_overview(&Ctx, Date)`, `day_entries(&Ctx, Date)`, `recents()`, `quick_switch(&str)`,
`task_matches(ProjectId, &str)`, `catalog()`, `running()`, `status() -> AppStatus`
(Ready/Broken + warnings). `subscribe(Box<dyn Fn(&[Event]) + Send + Sync>)`: called under the
lock after the swap. Subscribers only `send` on an `mpsc` channel and never call `App`.
Internally the state is `Ready { state, store } | Broken { reason }`. `check_invariants` runs
on the candidate under `debug_assertions` (a violation panics, so the process aborts). The
lock is a plain std `Mutex`; with `panic = "abort"` it can't be poisoned, so `lock()` uses
`.expect("app lock")`. `AppError`: `Domain(DomainError)`, `Store`, `Locked`, `Broken`.

**src-tauri** (`src-tauri/Cargo.toml`: package `timetracker-app`, `[lib] name =
"timetracker_app_lib"`). `tauri.conf.json`: productName `ThorSoft TimeTracker`, identifier
`de.thorsoft.timetracker`, mainBinaryName `timetracker`. `build.beforeDevCommand` =
`{"script":"npm run dev","cwd":"../ui"}`, `build.devUrl` `http://localhost:5173` (Vite
`strictPort: true`), `build.beforeBuildCommand` = `{"script":"npm run build","cwd":"../ui"}`,
`build.frontendDist` `../ui/dist`. Windows: `main` (`url: "index.html"`, `visible: false`,
shown in setup unless the process was started with `--autostart`) and `quick-switch` (`url:
"quick-switch.html"`, `visible: false`, `decorations: false`, `alwaysOnTop: true`,
`skipTaskbar: true`, `center: true`, `resizable: false`, 600×360).
`capabilities/default.json`: windows `["main","quick-switch"]`, permissions `core:default`,
`clipboard-manager:allow-write-text`. Autostart is
`tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--autostart"]))`, toggled
through Rust commands. `App::open` runs inside `.setup()`, after the single-instance plugin is
registered. The single-instance callback shows and focuses `main`. A bridge thread receives
the event channel, emits `data-changed` / `timer-changed` and refreshes the tray (reading
`App` outside the lock). The global-shortcut handler acts only on `ShortcutState::Pressed`:
show + focus `quick-switch`, then emit `quick-switch-shown` (the page clears its query and
focuses its input). `quick-switch` hides on `WindowEvent::Focused(false)`. `main`:
`CloseRequested { api }` → `api.prevent_close(); hide()`. The tray tooltip is cut to 127
chars. Icons: `cargo tauri icon` from `src-tauri/icons/app.svg`; `icons/tray-idle.png` and
`icons/tray-running.png` (32 px).

**ui**: Vite multi-page: `index.html` → `src/main.ts`, `quick-switch.html` →
`src/quick-switch.ts`, via `build.rolldownOptions.input` (Vite 8; if that key is rejected, use
`rollupOptions`). npm scripts: `dev`, `build` (`vue-tsc --noEmit && vite build`), `typecheck`,
`test` (`vitest run`).

## Files that change
| Path | New / changed / deleted | Purpose | Requirements |
|---|---|---|---|
| `spec.md` | changed | `Status: approved` (Thorsten Reichert, 2026-10-17); R12 end ≤ now | R12 |
| `plan.md` | new | this plan | all in scope |
| `Cargo.toml`, `rust-toolchain.toml`, `.gitignore` | new / changed | workspace (members `crates/*`, `src-tauri`; resolver 3), toolchain pin; ignore `target/`, `ui/node_modules/`, `ui/dist/` | — |
| `scripts/check.ps1` | new | one-command feedback loop: fmt, clippy, tests, typecheck, vitest; ends with `ALL CHECKS PASSED` | — |
| `CLAUDE.md`, `AGENTS.md` | changed | "Build & Test" section: commands + healthy output (kept identical) | — |
| `crates/tt-core/src/{lib,ids,name,instant,ctx,rounding,catalog,entries,state,command,event,error,decide,evolve,invariants,localtime}.rs` | new | domain types, Decider, invariants, OQ-6 conversion | R1, R3–R5, R10–R13, R17, R18 |
| `crates/tt-core/src/read/{mod,day,overview,day_entries,recents,quick_switch,task_match}.rs` | new | read models | R3, R6, R7, R10, R14, R16, R18, R19 |
| `crates/tt-core/src/testing.rs` (feature `test-support`) | new | proptest command generator, simulated clock and zone | — |
| `crates/tt-core/tests/{rounding,decide_catalog,decide_tracking,model,read_models}.rs` | new | worked examples, given/when/then, property tests | in-scope R1–R19 |
| `crates/tt-core/clippy.toml` | new | purity guard | — |
| `crates/tt-store/src/{lib,migrate,load,apply,backup,lock,raw_setting}.rs`, `migrations/0001_init.sql` | new | schema from spec §2 (incl. `archived_at`, `favorite`), load, apply in one tx, online backup + prune, lock | R1, R4 backstops, R22 |
| `crates/tt-store/tests/{schema,roundtrip,backup}.rs` | new | backstop constraints, round-trip, backup + prune | R1, R4, R22 |
| `crates/tt-app/src/{lib,startup,paths,error}.rs`, `tests/{write_path,startup,reload_equality,load_bench}.rs` | new | write path, startup order, Broken mode, events, live path, benchmark | R4, R5, R22 |
| `crates/tt-cli/src/main.rs`, `tests/scenarios.rs` | new | clap driver, JSON out, scenario tests | R1, R3–R5, R10–R14, R17–R19 |
| `src-tauri/{Cargo.toml,build.rs,tauri.conf.json,tauri.dev.conf.json,capabilities/default.json,icons/*}` | new | app config and identity, windows, CSP, NSIS currentUser | R6, R8, R9 |
| `src-tauri/src/{main,lib,commands,dto,bridge,ctx,tray,hotkey,bindings}.rs` | new | thin driver: DTOs, async commands, event bridge, tray, hotkey, single instance, autostart | R5–R9, R14, R15 |
| `ui/{package.json,vite.config.ts,tsconfig.json,index.html,quick-switch.html}` | new | Vite multi-page | — |
| `ui/src/{main.ts,quick-switch.ts,router.ts,bindings.ts}` | new | entries; `bindings.ts` generated | — |
| `ui/src/views/{TodayView,OverviewView,ClientsView,SettingsView}.vue` | new | main window views | R1, R3, R7, R10–R15, R17 |
| `ui/src/quick-switch/QuickSwitch.vue` | new | popup | R3, R6, R7 |
| `ui/src/components/{ComboPicker,TaskInput,EntryDialog,RunningBar,RecentsRow,CopyValue,StatusBanner}.vue` | new | shared widgets | R3, R5, R7, R10, R11, R15, AC-1 |
| `ui/src/stores/{timer,catalog}.ts`, `ui/src/lib/{format,format.test}.ts` | new | Pinia stores refreshed on `data-changed`; hour formatting + Vitest | R14, R15 |

## Order of work
Each step can be verified on its own. Run its proof before you move on. Steps 1–7 are
headless; the AC-6 skeleton runs end to end after step 11.

0. **Skeleton + work-machine check** (AC-2, AC-1, R6, R8, R9 spike)
   - Do: commit the spec approval (+ R12 sentence) and `plan.md` on `master`, then
     `git switch -c slice-1`. Workspace, toolchain pin, empty crates with the dependency
     direction wired, the `ui/` scaffold (both pages, "hello"), and `src-tauri` with the pinned
     config. It has: a tray icon with Quit; the `tauri-specta` command `ping` returning a DTO
     with a `u64` field, rendered on the page; a hidden `quick-switch` window shown and
     focused by the hotkey (Pressed only); single-instance focus; autostart enabled (release
     only); `tauri.dev.conf.json`. Install the Tauri CLI. `scripts/check.ps1`; Build & Test in
     CLAUDE.md/AGENTS.md.
   - Proof: `pwsh scripts/check.ps1` green, including `cargo test -p timetracker-app
     bindings_up_to_date`. If a Tauri test exits with `STATUS_ENTRYPOINT_NOT_FOUND`, apply the
     `build.rs` manifest workaround and record it as a deviation. `cargo tauri build` →
     `target/release/bundle/nsis/ThorSoft TimeTracker_0.1.0_x64-setup.exe` (this proves the
     NSIS download works on the corporate network). **Thorsten, on this machine:** install it
     and check:
     - SmartScreen/AppLocker: record the result.
     - The tray icon appears.
     - The hotkey brings the popup to the front with focus from another app (Notepad,
       Outlook).
     - Launching again focuses the first instance.
     - It's listed in Settings → Apps → Startup.
     - It starts hidden after sign-out/sign-in.
     - `cargo tauri dev --config src-tauri/tauri.dev.conf.json` runs alongside the installed
       app.

     Record the results under "AC-2 result" in plan.md. If anything is blocked, stop and talk
     to IT before step 1.
1. **Domain values + rounding** (R17, R18)
   - Do: `ids`, `name`, `instant`, `ctx`, `rounding`; the purity `clippy.toml`.
   - Proof: `cargo test -p tt-core --test rounding`:
     - Worked examples: 22 min @15 → up 30, nearest 15, down 15; 22 min 30 s @15 nearest → 30
       (tie rounds up); 0 → 0; one example per unit.
     - Proptests: idempotent; `down ≤ raw ≤ up`; `|nearest − raw| ≤ u/2`; the result is a
       multiple of `u`.
     - `Name` tests: trim, empty, length, case key.
     - `TimeZone::get("Europe/Berlin")` resolves (bundled tzdb).

     `cargo tree -p tt-core -e normal` lists no rusqlite or tauri. Clippy stays green with the
     purity lints.
2. **Decider: catalog** (R1, R3, R17)
   - Do: `State`/`Catalog`/`Counters`; `decide`/`evolve` for client/project/task create and
     rename, and client rule updates; `check_invariants` (names unique per parent,
     case-insensitive; references exist; counters > max IDs).
   - Proof: `cargo test -p tt-core --test decide_catalog`:
     - Each command produces the expected events.
     - A duplicate name in any case → `DuplicateName`; a case-only rename of the item itself
       is OK.
     - An unknown parent is rejected.
     - IDs increase.
3. **Decider: tracking + local times** (R4, R5, R10, R11, R12, R13)
   - Do:
     - `Entries` with its index and named queries.
     - The commands `Start`, `Stop { at }`, `AddEntry`, `EditEntry`, `DeleteEntry`, following
       the decisions above (switch order, no-op, zero-length discard, clock went back,
       `EndRequired`), plus `TaskChoice::New`.
     - `localtime::to_instant(&TimeZone, civil::DateTime)`.
     - Invariants: at most one running entry; no overlap (half-open; the running entry
       counts from its start to ∞ for the invariant, and to `now` for validation); end >
       start; the task belongs to the entry's project.
   - Proof: `cargo test -p tt-core --test decide_tracking`:
     - A switch emits exactly `[EntryStopped, EntryStarted]` at the same instant.
     - `Start` with `New` emits `[EntryStopped, TaskCreated, EntryStarted]`.
     - The same combo → `[]`.
     - Start/Stop at the same second → `EntryDeleted`.
     - `Stop { at ≤ start }` → `EndNotAfterStart`.
     - An overlap → `Overlap { with, start, end }`.
     - A future start or end is rejected.
     - Europe/Berlin 2026-03-29 02:30 → `NonexistentLocalTime`; 2026-10-25 02:30 →
       `AmbiguousLocalTime`.
     - Editing the running entry's combo, note or start keeps it running.
     - The entry ID isn't reused after deleting the newest entry.

     `cargo test -p tt-core --test model`: 256 random sequences of up to 200 commands, in
     Europe/Berlin around both DST changes. `check_invariants` holds after every step;
     `decide` is deterministic (run twice → equal results); a rejected command leaves State
     unchanged.
4. **Read models** (R3, R6, R7, R10, R14, R16, R18, R19)
   - Do: the read functions as pinned above.
   - Proof: `cargo test -p tt-core --test read_models`:
     - Worked day: Acme (project level, 15 nearest) Web 09:00–10:22 → raw 4920 s, rounded
       4500 s, diff −420 s. Beta (client level, 30 up) API 10:22–11:07 + Ops 11:07–11:20 →
       raw 3480 s, total rounded 3600 s, diff +120 s, project rows raw only with `booked:
       None`.
     - An entry crossing midnight counts on its start day.
     - A running entry counts to `now` and is marked.
     - Changing Beta's rule to 15 down changes the result (R19).
     - Fuzzy "acweb" finds "Acme / Web".
     - An exact task name (any case) → `offer_create: None`.
     - Recents are distinct, newest first, n = 8.
5. **tt-store** (R1/R4 backstops, R22)
   - Do: `0001_init.sql` exactly as spec §2 (`one_running` index, NOCASE unique indexes,
     composite FKs, CHECKs, `counter`, `schema_migrations`); the Store API as pinned; prune
     keeps the newest `keep` matching files.
   - Proof: `cargo test -p tt-store`:
     - Raw SQL: a second running entry fails; `ACME`/`acme` fails; a task from another
       project fails.
     - apply → load equals the evolved State.
     - The backup opens and holds the same rows.
     - Prune keeps 14; the same file name is overwritten.
     - A second `DbLock` fails.
     - `raw_setting` on an empty file → `None`.
6. **tt-app write path + startup** (R4, R5, R22, OQ-9 benchmark)
   - Do: `App::open` (lock → backup if the file exists → migrate → load → `check_invariants`
     → Ready, or Broken), `execute` (decide → evolve candidate → tx → commit → swap →
     publish; on a store error reload; if the reload fails, Broken), reads, `subscribe`,
     `live_db_path()`.
   - Proof: `cargo test -p tt-app`:
     - The step 3 generator runs against a temp DB, which is reopened after each sequence:
       the serialised State is equal.
     - A store error (a second raw connection holds `BEGIN EXCLUSIVE`, busy timeout 0) → the
       command fails, State is reloaded and equals the DB, and the next command succeeds.
     - Backup before migrations: opened with `migrations: [0001, test_0002]`, the backup has
       no `test_0002` table.
     - A failing backup with a pending migration → Broken.
     - Overlapping rows inserted with raw SQL → `open` gives Broken, and `execute` →
       `AppError::Broken`.
     - A second `App::open` → `Locked`.

     `cargo test -p tt-app --release -- --ignored load_bench`: 100k entries are written with
     one bulk `apply`. Load + check < 500 ms, process memory < 50 MB, and one `execute(Start)`
     is timed; all printed.
7. **tt-cli** (R1, R3–R5, R10–R14, R17–R19)
   - Do: clap with global flags:
     - `--db <path>`: required; refuses `live_db_path()`.
     - `--now <local datetime | RFC3339>`: default is the system clock.
     - `--tz <IANA>`: default is the system zone.

     Subcommands: `client add|edit`, `project add|rename`, `task add|rename`, `start --project
     <id> [--task <id> | --new-task <name>]`, `stop [--at]`, `entry add|edit|delete`,
     `entries --date`, `overview --date`, `recents`, `quick-switch <q>`, `settings set`,
     `state`. Output goes to stdout as JSON. Exit codes: 0 ok, 1 other error, 2 usage (clap),
     3 domain error (JSON on stderr). Backups go to `<db dir>/backups`.
   - Proof: `cargo test -p tt-cli`:
     - The step 4 worked day, built with chained calls (`start` Acme/Web 09:00 → `start`
       Beta/API 10:22 → `start` Beta/Ops 11:07 → `stop` 11:20), gives the same seconds in
       `overview`.
     - A forgotten stop is fixed with `stop --at`.
     - An overlapping `entry add` → exit 3 naming the entry.
     - A missing `--db` → exit 2.
     - The live DB path → refused.
8. **src-tauri driver** (R5, R9, R14)
   - Do: `ctx` (`Instant` from `jiff::Timestamp::now()`, `TimeZone::system()`); `App` in
     managed state (opened in `setup`); DTOs + async commands mirroring the tt-app API; errors
     as `UiError { kind, message }`; the bridge thread; Broken mode → UI error screen;
     autostart enabled on first run (new DB, release only).
   - Proof: `cargo test -p timetracker-app`. `cargo tauri dev --config
     src-tauri/tauri.dev.conf.json` → the hello page renders `catalog()` as JSON (empty), and
     the dev DB appears under `...\ThorSoft.TimeTracker\dev\`. With `TT_DB=<tt-cli scenario
     db>` it shows the scenario's catalog.
9. **UI: shell, clients & projects, start/stop, recents** (R1, R3, R5, R7, R17)
   - Do:
     - PrimeVue + router + Pinia.
     - Navigation: Today / Daily overview / Clients & projects / Settings.
     - ClientsView: create and rename clients with their rule, projects and tasks; errors are
       shown.
     - RunningBar: the combo plus elapsed time computed in the UI each second, and Stop.
     - RecentsRow: one click starts or switches.
     - ComboPicker + TaskInput (R3 "Create …").
     - StatusBanner for Broken mode and warnings.
   - Proof: `npm --prefix ui run typecheck`. Manual check in dev:
     - Create "Acme" @15 nearest and the project "Web".
     - Start with a new task "Review" typed in; the timer counts.
     - Switch via recents, then stop.
     - A duplicate name shows the error.
10. **Quick-switch popup + tray** (R6, R7, R8, AC-1)
    - Do:
      - QuickSwitch.vue: the running entry on top, then recents. Typing filters via
        `quick_switch`.
      - Keys: Enter = start/switch, then hide. Tab on a project → TaskInput. Esc = hide.
        Ctrl+S = stop.
      - Tray: idle/running icon. The tooltip ("Client / Project / Task · 1:23") is refreshed
        each minute and on `timer-changed`. Menu: Stop, recents, Open window, Quit.
      - A hotkey registration failure → StatusBanner + tray tooltip.
    - Proof: manual check:
      - From Notepad: hotkey → 3–4 letters → Enter switches within ~3 s / ~5 keystrokes (R6
        target, timed with a stopwatch).
      - The tray tooltip updates.
      - AC-1: set the dev build's hotkey to `Ctrl+Alt+Space` while the installed app holds
        it → the banner text is visible.
11. **Daily overview** (R14, R15, R16), which completes the **AC-6 skeleton**
    - Do: OverviewView: a date with prev/next/today; the client blocks from step 4; rounded,
      raw and diff for each booked value; the running marker; CopyValue copies the number
      only; `format.ts` (2 decimals, separator from the setting).
    - Proof: `npm --prefix ui test` (format: 0 → 0.00, 900 s → 0.25, 1 s → 0.00, 1800 s →
      0.50, −900 s → −0.25, 4920 s → 1.37, comma separator). Manual check: dev with
      `TT_DB=<tt-cli scenario db>`. The worked day shows Acme/Web 1.25 (raw 1.37, diff −0.12)
      and Beta 1.00 (raw 0.97, diff +0.03). Copy with the comma setting → Notepad shows
      `1,25`.
12. **Entries: list, add, edit, delete, forgotten stop** (R10, R11, R12, R13)
    - Do: the TodayView entry list for the selected day (prev/next); EntryDialog (combo,
      start, end, note; DatePicker + time); delete via ConfirmDialog; "Stop at…" and editing
      the running entry; domain errors rendered with the conflicting entry's combo and times.
    - Proof: manual check:
      - Add a past entry.
      - An overlapping save shows "Overlaps Acme / Web 09:00–10:22".
      - A time in the DST gap shows the OQ-6 message.
      - A future end is rejected.
      - Delete asks first.
      - Notes appear in the list and not in the overview.
13. **Settings, autostart, backup folder** (R6, R9, R15, R22)
    - Do: SettingsView: hotkey (re-registered on save, AC-1 error), decimal separator,
      recents N, backup folder + keep N, the autostart toggle (the plugin is the only source of
      truth).
    - Proof: manual check:
      - Change the hotkey → the old one is free and the new one works.
      - Toggle autostart (release build) → Startup apps reflects it.
      - Set the backup folder to a OneDrive path, restart → a new backup file is there, and
        the DB stays in `%LOCALAPPDATA%`.
      - With an unreachable folder → the warning banner, and the app still works.
14. **Release + dogfood** (all in scope)
    - Do: version 0.1.0, `cargo tauri build`, install over the step 0 build on this machine.
    - Proof: the whole-change proof below, plus one real workday tracked and the evening
      overview copied into the booking system. Findings go into beads.

## Risks
| Risk | Where | Mitigation / early check |
|---|---|---|
| Corporate policy blocks the unsigned app, autostart or the hotkey | 0 | Installer check on this machine before any feature work; NSIS per user; stop and ask IT if blocked |
| `cargo tauri build` can't download NSIS (proxy) | 0 | Seen in step 0. Fallback: download NSIS manually into the Tauri tools cache |
| The popup doesn't get foreground focus (Windows foreground lock) | 0, 10 | Spiked in step 0 from another app. A hotkey press gives the receiving process foreground rights. Fallback: `set_always_on_top` + `set_focus` retry |
| `tauri-specta` rc breaks with Tauri 2.12, or 64-bit numbers | 0, 8 | Exact pin; `ping` with a `u64` in step 0; fallback `ts-rs` + hand-written wrappers |
| Tauri test binaries fail on Windows (`STATUS_ENTRYPOINT_NOT_FOUND`) | 0 | A Tauri crate test runs in step 0; `build.rs` manifest workaround |
| Dev build clashes with the installed app | 0, 8 | Separate identifier, DB, hotkey; no autostart in debug |
| Overlap/index logic in `Entries` is wrong at the edges (half-open, running) | 3 | Given/when/then edge tests plus the model property test; invariants checked after every evolve |
| State and DB drift | 5, 6 | Reload-equality property test; the DB is the authority on store errors |
| DST and day-boundary bugs | 3, 4 | jiff `start_of_day`; tests on both 2026 Berlin DST dates |
| A bug crashes the tray app (crash-only), so tracking UI and hotkey are gone until relaunch | 3–13 | Property tests before the UI; the running entry survives in the DB; relaunch from the Start menu or the next sign-in (autostart). Revisit if it happens in dogfooding |
| Clone per command is too slow for R6 at volume | 6 | `execute(Start)` timed in `load_bench`; if slow, persistent maps later (OQ-9) |
| PrimeVue runtime styles vs CSP | 9 | `'unsafe-inline'` + `dangerousDisableAssetCspModification: ["style-src"]`, styles only; scripts stay `'self'` |
| The learning curve stalls progress (AC-6) | all | Headless steps 1–7 first (pure Rust, fast tests), UI later |

Riskiest step: **0**, because a block by corporate policy or a popup that can't take focus
would change the shell design (R6, R8, R9), and both are outside the code's control. Doing it
first, before any feature code exists, de-risks it. Of the code steps, the riskiest is **3**
(the overlap, switch and DST rules), de-risked by the model-based property test.

## Proof (whole change)
- Build: `cargo tauri build` ends by listing the NSIS bundle path
  (`target/release/bundle/nsis/ThorSoft TimeTracker_0.1.0_x64-setup.exe`).
- Tests: `cargo test --workspace` is all green, with nothing ignored except `load_bench`;
  `npm --prefix ui test` is green. New tests are listed per step.
- Lint: `cargo fmt --all --check` (no output); `cargo clippy --workspace --all-targets --
  -D warnings` (zero warnings); `npm --prefix ui run typecheck` (no errors).
- One command: `pwsh scripts/check.ps1` runs all of the above and ends with
  `ALL CHECKS PASSED`.
- Benchmark: `cargo test -p tt-app --release -- --ignored load_bench` stays within its limits.
- Manual: the checks from steps 0 and 9–13 on the installed build; the R6 timing with a
  stopwatch.

## Requirement coverage
| R | Step(s) |
|---|---|
| R1 | 1, 2, 5, 7, 9 |
| R2 | **deferred** (slice 2); `archived_at` columns in 5; quick-switch/recents filter active items in 4 |
| R3 | 2, 3, 4, 9, 10 |
| R4 | 3, 5, 6 |
| R5 | 3, 6, 7, 9, 10 |
| R6 | 0, 4, 10, 13 |
| R7 | recents: 4, 9, 10; favorites **deferred** (slice 2) |
| R8 | 0, 10 |
| R9 | 0, 8, 13 |
| R10 | 3, 4, 7, 12 |
| R11 | 3, 7, 12 |
| R12 | 3, 7, 12 |
| R13 | 3, 12 |
| R14 | 4, 7, 11 |
| R15 | 11, 13 |
| R16 | 4, 11 |
| R17 | 1, 2, 9 |
| R18 | 1, 4 |
| R19 | 4, 7 |
| R20 | **deferred** (slice 2) |
| R21 | **deferred** (slice 2) |
| R22 | startup backup + prune + folder: 5, 6, 13; daily timer **deferred** (slice 2) |
| R23 | **deferred** (slice 2) |

## Beads (after approval)
One epic "Slice 0+1" with one bead per step 0–14. Each bead names its step and R numbers.
Dependencies: 1←0, 2←1, 3←2, 4←3, 5←3, 6←4+5, 7←6, 8←6, 9←8, 10←9, 11←9, 12←9, 13←10,
14←11+12+13. Follow-ups go into beads, not into this file.

## AC-2 result
2026-10-17, work machine, installer `ThorSoft TimeTracker_0.1.0_x64-setup.exe` (step 0 build).
The owner reports that all step 0 manual checks passed as expected:
- installed per user, with no blocking by SmartScreen or AppLocker
- the tray icon appears and the window shows the ping result
- the hotkey brings the popup to the front with focus from another app
- a second launch focuses the running instance
- the app is listed in Startup apps and starts hidden after sign-in
- the dev build runs next to the installed app

No IT involvement needed; AC-2 is closed for slice 1.

## Deviations
- **Step 0, BigInt export (2026-10-17).** tauri-specta rc.25 has no
  `Typescript::bigint(BigIntExportBehavior::Number)`. It's
  `tauri_specta::Builder::dangerously_cast_bigints_to_number()`, used in
  `src-tauri/src/bindings.rs`. The effect is the same (`u64` → `number`); it's checked by
  `Pong.buildId = 2^53 − 1`.
- **Step 0, Windows manifest (2026-10-17).** The predicted `STATUS_ENTRYPOINT_NOT_FOUND`
  happened. tauri-build only gives the binary the Common Controls v6 manifest, which Tauri
  needs. `src-tauri/build.rs` now uses `WindowsAttributes::new_without_app_manifest()` and
  links `src-tauri/windows-app-manifest.xml` (Tauri's default) into every linked target
  (`cargo:rustc-link-arg=/MANIFEST:EMBED` + `/MANIFESTINPUT`). The mobile-only crate types
  `staticlib`/`cdylib` were dropped from `[lib]`.
- **Step 0, tray icons (2026-10-17).** The 32 px PNGs come from `src-tauri/icons/tray-idle.svg`
  and `tray-running.svg` via `cargo tauri icon <svg> -o <tmp> -p 32`. The SVGs are committed
  with the PNGs.
- **Step 0, UI test script.** `npm test` runs `vitest run --passWithNoTests` until step 11 adds
  the first test file. Drop the flag then.
