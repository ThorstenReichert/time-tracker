# Plan: Personal project time tracker — slice 0 + slice 1
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude). Status: approved; released as 0.1.0 on 2026-10-19 (tag `v0.1.0`, see [release-notes.md](release-notes.md)).
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
  → the newest `recents_n` recents, then the other active combos A–Z (step 9 follow-up); otherwise fuzzy over
  "Client / Project / Task" and project-only combos of active items); `read::task_matches(&Catalog, ProjectId, text: &str) -> TaskMatches { tasks,
  offer_create: Option<Name> }`.

**tt-store**: `Store::open(path, migrations: &[Migration]) -> Result<(Store, OpenInfo {
existed, pending_migrations })>`, `migrate()`, `load() -> State`, `apply(&[Event])` (one
transaction), `backup_to(dir, now: Instant) -> PathBuf` (file
`timetracker-YYYYMMDDTHHMMSSZ.db`) and `prune_backups(dir, keep: usize)` (stage 5 review),
`raw_setting(path, key)`, `DbLock::acquire(path)`.

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
Capabilities (narrowed in the stage 5 review): `capabilities/default.json` gives both windows
`core:event:allow-listen` and `core:event:allow-unlisten`; `capabilities/main.json` gives
`main` `clipboard-manager:allow-write-text`; `capabilities/quick-switch.json` gives the popup
`core:window:allow-hide`. Autostart is
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
| `src-tauri/{Cargo.toml,build.rs,tauri.conf.json,tauri.dev.conf.json,capabilities/{default,main,quick-switch}.json,icons/*}` | new | app config and identity, windows, CSP, NSIS currentUser | R6, R8, R9 |
| `src-tauri/src/{main,lib,commands,dto,bridge,backend,errors,tray,hotkey,bindings}.rs` | new | thin driver: DTOs, async commands, event bridge, tray, hotkey, single instance, autostart | R5–R9, R14, R15 |
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
  `ALL CHECKS PASSED`. In Claude Code, the `gatekeeper` subagent runs it (see Deviations).
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
- **Step 4, read model shapes (2026-10-17).** These are below plan-step level, recorded
  for the stranger test:
  - `read.rs` plus `read/{combo,day,day_entries,overview,quick_switch,task_match}.rs`
    replace `read/mod.rs` and `read/recents.rs`; `recents` lives in `combo.rs`, next to
    `ComboRow`.
  - `day_bounds` returns `Result<_, DomainError::TimeOutOfRange>`, so a far-out date can't
    crash the app.
  - `ClientBlock` carries the whole `rule`, not just `level`.
  - `quick_switch` returns at most `QUICK_SWITCH_LIMIT` = 50 rows.
- **Step 5, schema and backstop tests (2026-10-17).** The schema adds three things the spec
  didn't spell out:
  - `STRICT` tables;
  - a `CHECK (end_utc > start_utc)` backstop for R12;
  - an index `time_entry_start` for the named queries.

  The backstop tests feed crafted events through `Store::apply` instead of raw SQL. That's
  the path the app uses, with `foreign_keys=ON` set by `Store::open`. Updates and deletes
  that hit no row fail with `StoreError::RowMissing`, so drift between memory and the DB
  shows up as a store error. `Store::open` takes `&'static [Migration]`.
- **Step 6, fault injection (2026-10-17).** The store error is injected with a SQLite
  trigger, not with a second connection holding `BEGIN EXCLUSIVE`. A second connection
  creates `BEFORE INSERT ON time_entry … RAISE(ABORT)` and the next save fails at once. That
  needs no busy timeout and has no timing. A failed reload is forced the same way, plus a
  renamed `setting` table. An out-of-band `DELETE` shows that the DB is the authority
  (`RowMissing` → reload). `AppError` has no `Bug` variant (crash-only).
  `App::running(&Ctx)` returns an `EntryRow` via the new `read::entry_row`, and
  `App::snapshot()` serves the tests and `tt-cli state`.
- **Step 6, benchmark result (2026-10-17, this machine, release).** 100k entries, 4 MB file:
  - load + check: 46 ms (limit 500 ms)
  - process memory with State loaded: 35 MB (limit 50 MB)
  - full startup including the backup: 74 ms
  - one `execute(Start)`: 17 ms

  The OQ-9 trigger isn't reached.
- **Step 8, driver details (2026-10-17).**
  - Write commands return nothing; the UI reloads on `data-changed`.
  - `ping` is gone; the hello page renders `getStatus` + `getCatalog`.
  - `UiError { kind: domain|store|broken|input, message }` renders an overlap as "Overlaps
    Acme / Web 09:00–10:22", via the new `App::entry`.
  - The tray icon and tooltip already follow the running entry (`tray::refresh` from the
    bridge). The once-a-minute refresh and the menu entries stay in step 10.
  - The hotkey comes from the settings, else the default.
  - Autostart is enabled only on first run (new DB) in release builds.

  Proof: the dev build started next to the installed app. Its default DB appeared under
  `%LOCALAPPDATA%\ThorSoft.TimeTracker\dev\`, and with `TT_DB` it showed the tt-cli
  scenario's catalog (screenshots in the session).
- **Step 0, UI test script.** `npm test` ran `vitest run --passWithNoTests` until the first test
  file existed. Step 9 added `lib/format.test.ts` and `lib/combo.test.ts` and dropped the flag.
- **Step 9, UI shell details (2026-10-18).**
  - `CatalogDto` also carries `defaultRule` and `roundingUnits`, so the client dialog keeps no
    copy of the core's R17 defaults.
  - A duplicate name is rendered with the existing item's name (`The name is already used by
    client "Acme"`), like the overlap message.
  - Extra files: `components/{ClientDialog,NameDialog}.vue`, `lib/combo.ts` (picked combos →
    `TaskChoiceDto`), `lib/attempt.ts` (toast for one-click actions; forms show errors
    inline), `style.css`. `format.ts` starts with `formatElapsed`; step 11 adds the hours.
  - OverviewView and SettingsView are placeholders until steps 11 and 13.
  - ComboPicker picks the project through `quick_switch` (a task combo fills both fields), and
    TaskInput through `task_matches` with `force-selection`. Typed text that wasn't picked
    doesn't count, except that leaving the task input with text that exactly matches an
    offered option (including `Create "<text>"`) picks it.

  Proof: `pwsh scripts/check.ps1` passed. Dev build with `TT_DB=<tt-cli demo db>`, driven
  through simulated input, with screenshots in the session: the shell and the running bar
  counting; Clients & projects tree; `acme` as a new client → inline duplicate error;
  "rev" + Enter → Acme / Web + Review; a new task "Docs" + Start → switched with "Docs"
  created, recents updated, picker cleared; Stop; a recents click starts.
- **Step 9 follow-up, blank quick-switch query (owner, 2026-10-18).** Found in manual testing:
  on a new database the project picker stayed empty until you typed, because a blank query
  returned only the recents, and recents come from entries. A blank query now returns the
  recents, then the other active combos sorted by label, capped at `QUICK_SWITCH_LIMIT`.
  The popup (step 10) gets the same list. spec.md R6 says so. Rejected: a separate read
  model for the main window only (the popup would stay empty on a new install). Proof:
  `cargo test -p tt-core --test read_models` (`quick_switch_lists_untracked_combos_for_a_blank_query`,
  `quick_switch_caps_a_blank_query`, and the blank cases in the existing tests); the owner
  checked in the dev build that new projects show at once, below the recents.
- **Process, gatekeeper subagent (owner, 2026-10-18).** From step 10 on, the automated proofs
  (`pwsh scripts/check.ps1`, a crate's or file's tests, `load_bench`) run through the
  `gatekeeper` subagent (`.claude/agents/gatekeeper.md`, Haiku). It only runs the checks and
  returns PASSED or the literal failure excerpt; the main session fixes and commits. The
  proof commands in the steps don't change, only who runs them. Reason: long build and test
  logs stay out of the main context. Manual checks and screenshot loops stay in the main
  session. CLAUDE.md and AGENTS.md "Build & Test" say so.
- **Step 10, popup and tray details (2026-10-18).**
  - The popup is plain Vue with its own CSS (light and dark), not PrimeVue, so it stays small
    and opens at once. One input serves both steps: the combo list, then after Tab the task
    step for the highlighted project ("No task" first while nothing is typed, then the
    matches and `Create "<text>"`). Shift+Tab, or Backspace in an empty task input, goes back.
    Enter and Tab wait for the latest search, so typing fast and pressing Enter never picks a
    stale row. With nothing typed, the running combo is left out of the list (it's shown
    above), so hotkey + Enter switches back to the previous combo. The state resets when the
    popup hides or loses focus. The row logic is in `ui/src/lib/quickSwitch.ts`, with
    `quickSwitch.test.ts`. `lib/combo.ts` gained `taskPicks`, shared with TaskInput.
  - The popup hides itself through `core:window:allow-hide`, granted to that window only in
    the new `src-tauri/capabilities/quick-switch.json`.
  - Tray: one `tray` thread owns the refresh. Every event batch and every hotkey status
    change nudges it to rebuild the menu and tooltip. Otherwise it wakes at the running
    entry's next full minute and updates only the tooltip and icon, so an open menu stays
    put. The menu: "Stop <combo>" (disabled when idle), the recents (the running one greyed
    out), Open window, Quit. `&` is doubled for Windows menus. A failed tray action shows the
    main window with a toast (event `action-failed`). The tooltip limit counts UTF-16 units
    (127), not chars.
  - AC-1: the managed `HotkeyStatus` feeds `StatusDto.warnings` (the main window's banner)
    and the first tooltip line. The hotkey is parsed before registering, so the banner can
    say "isn't a valid hotkey" or "Windows refused it, so another app probably uses it".

  Proof: `pwsh scripts/check.ps1` passed (gatekeeper). Dev build with `TT_DB=<demo db>`,
  driven through simulated input, with screenshots in the session:
  - From another app (Windows Terminal), hotkey + `rev` + Enter switched to Acme / Web /
    Review: 5 keystrokes, popup gone after 559 ms (automated input), focus back on the
    terminal.
  - `resea` → Tab → `lit` → `Create "lit"` → Enter: the task was created and the timer
    switched.
  - Ctrl+S stopped the timer; the tooltip went to idle.
  - The tray tooltip went 0:00 → 0:01 → 0:02 at 60 s intervals with no other events.
  - The tray menu switched to a recent and quit the app.
  - AC-1: with `Ctrl+Alt+Space` held by a scratch process and set as the demo DB's hotkey,
    the banner and the tray tooltip showed the failure.

  R6 stopwatch timing by hand (owner, 2026-10-18, dev build on the worked-day scratch DB,
  from Notepad): hotkey + `web` + Enter, hotkey + Enter (back to the previous combo) and
  hotkey + `api` + Enter each took 1–2 s after a little practice with the shortcuts, with at
  most 4 keystrokes after the hotkey. The R6 target (~3 s / ~5 keystrokes) is met.
- **Component tests (owner, 2026-10-18, before step 11).** The UI gets a component-test level,
  run by `npm --prefix ui test` and so by `scripts/check.ps1`:
  - New dev dependencies `@vue/test-utils` 2.5 and `happy-dom` 21. Vitest now runs in happy-dom
    with `ui/src/test/setup.ts`.
  - The helpers live in `ui/src/test/`:
    - `backend.ts` fakes the Tauri backend with `mockIPC`: handlers per command, recorded
      calls, and faked events, so `emit("data-changed")` reaches the page.
    - `fixtures.ts` holds the demo combos and catalog.
    - `mount.ts` mounts like main.ts (PrimeVue, toasts, Pinia).
    - `page.ts` finds dialogs, buttons and AutoComplete suggestions.
  - Tests sit next to the components (`<Name>.test.ts`). The backfill covers QuickSwitch, App
    (the `action-failed` toast), StatusBanner, RunningBar, RecentsRow, NameDialog,
    ClientDialog, TaskInput, ComboPicker, ClientsView and TodayView.
  - Rejected for now: Playwright against the WebView2 window, and scripted native checks
    (hotkey, tray, focus). The step 10 checks showed that most of the manual work was inside
    the page. Revisit if the in-page behaviour needs the real backend, or if native checks
    keep recurring.

  The steps from 11 on add component tests for their views. Bead `qvt.17`; steps 11–13
  depend on it. Proof: `npm --prefix ui test` passes 64 tests in 14 files, and
  `npm --prefix ui run typecheck` is clean. As a mutation check, removing a behaviour made the
  matching test fail. The behaviours removed were the Enter-waits-for-search line in
  QuickSwitch, TaskInput's "typing drops the pick" and ComboPicker's task fill.
- **Step 11, overview details (2026-10-18).**
  - Copy uses the clipboard plugin's JS API, so `@tauri-apps/plugin-clipboard-manager` **~2.5**
    is a new UI dependency. It has to match the Rust plugin's minor version (2.5); otherwise
    the Tauri CLI reports "version mismatched Tauri packages".
  - The day is picked with prev/next/today plus a PrimeVue DatePicker (R14 "any date"). The
    date math is in `ui/src/lib/day.ts` (UTC arithmetic on `YYYY-MM-DD`, so DST can't shift a
    day), with `day.test.ts`. `format.ts` gained `decimalMark`, `formatHours`, `formatDiff`
    and `formatRule` (shared with ClientsView).
  - While anything is running, the view reloads once a minute, so the running entry counts
    up (R14). It also reloads on `data-changed`, and drops an answer for a day that's no
    longer shown.
  - At project level, the client subtotal is shown without a copy button: only the project
    values are booked. At client level, the client total has the button.
  - The diff is rounded from seconds on its own, so it can differ by 0.01 from the
    difference of the two displayed values (e.g. 1.50 − 1.38 shown as +0.13 for 450 s), as
    spec §2 and AC-3 allow.

  Proof: `pwsh scripts/check.ps1` passed (gatekeeper), with `OverviewView.test.ts` and
  `CopyValue.test.ts` (as a mutation check, removing the stale-answer guard or the
  "only while running" check made a test fail). Dev build with `TT_DB=<tt-cli worked-day
  db>`, driven through simulated input, with screenshots in the session: 2026-10-17 shows
  Acme / Web 1,25 (raw 1,37, diff −0,12) and Beta 1,00 (raw 0,97, diff +0,03), with the
  separator on `auto` and a German locale. Copy put `1,25`, then `1,00`, on the clipboard
  (read back with `Get-Clipboard`). Today shows the running Beta / Ops marked; prev/next,
  Today and the DatePicker change the day; an empty day says so.
- **Step 12, entries details (2026-10-18).**
  - Typed times stay text, `YYYY-MM-DDTHH:MM:SS`. The new `LocalTimeInput` has a DatePicker for
    the date and a text field for the time. A JS `Date` would silently move a time in the DST gap,
    which has to reach the core and be rejected there (OQ-6). `ui/src/lib/localTime.ts` reads
    `9`, `9:05`, `905`, `09.05` and `09:05:30` (with `localTime.test.ts`), and an edit keeps an
    entry's seconds. `format.ts` gained `formatDuration` (h:mm).
  - `EntryDialog` adds and edits. On the running entry, an empty end keeps it running and an end
    stops it (R11). `StopAtDialog` is the forgotten stop. Both start on the entry's start day.
    Combo, start and end are read in the UI only to send them; the rules stay in the core.
  - RunningBar gained Edit and "Stop at…", so a timer that started on an earlier day can be
    fixed without finding its day in the list. `timer.stop(at)` takes the typed time.
  - The day navigation moved from OverviewView into `components/DayBar.vue`, which TodayView
    uses too.
  - Delete uses PrimeVue's ConfirmDialog. `ConfirmationService` is registered in `main.ts` and
    `test/mount.ts`, and the `ConfirmDialog` is in `App.vue`. The test helper `withToast` is now
    `withOverlays` and renders both overlays. The confirmation service is a global event bus, so
    `test/setup.ts` now unmounts every test's components (`enableAutoUnmount`). Without that, a
    ConfirmDialog left mounted by an earlier test answered the next test's confirmation.
  - Extra files: `components/{DayBar,LocalTimeInput,StopAtDialog}.vue`, `lib/localTime.ts`, plus
    tests for EntryDialog, StopAtDialog, RunningBar and TodayView.

  Proof: `pwsh scripts/check.ps1` passed (gatekeeper; 109 UI tests in 20 files). As a mutation
  check, deleting without the confirmation, or always sending an end for the running entry,
  made tests fail. Dev build with `TT_DB=<tt-cli worked-day db>` (the 17th, plus a timer
  running since 09:00 on the 18th), driven through simulated input, with screenshots in the
  session:
  - The 17th lists the three entries, with the note "Kickoff with the team" below Acme / Web.
  - Add entry 10:00–10:30 → "Overlaps Acme / Web 09:00–10:22", and the dialog stays open.
  - 12:00–13:00 with a note was added and listed.
  - "Stop at…" 23:59 today → "An entry can't end in the future"; 15:30 stopped the timer
    (09:00–15:30).
  - Editing that entry to 2026-03-29 02:30 → "This local time doesn't exist (daylight saving
    time starts); pick another". The end date followed the start date.
  - Delete asked "Delete the entry? Acme / Web / Review, 09:00–15:30" first, then deleted.
  - The overview for the 17th shows Web at 2,37 raw (with the added hour) and no notes.

  Found in the check and filed as a bead: the date pickers start the week on Sunday.
- **Step 13, settings details (2026-10-18).**
  - SettingsView saves the hotkey, recents N, separator, backup folder and keep N together, in
    one `update_settings`. A blank hotkey or folder is saved as `None` (the default), and the
    defaults are shown as placeholders from the new command `get_settings_defaults`.
    `tt_app::default_backup_dir` is now public, shared by startup, tt-cli and that command. The
    form follows the saved settings only when they change, so a reload for anything else keeps
    what is being typed.
  - The hotkey is a text field (`Ctrl+Alt+Space`, `Ctrl+Shift+K`), not a key recorder:
    pressing the current hotkey in a recorder would open the popup. `update_settings` rejects a
    hotkey the plugin can't parse, so it's never saved. After a save, the bridge sees
    `SettingsChanged`, unregisters every hotkey and registers the configured one, so the old
    one is free at once. If Windows refuses the new one, the AC-1 banner and tooltip say so.
  - Autostart: `get_autostart` / `set_autostart` read and change the plugin's entry; the
    switch shows what Windows has after every change, also a refused one. `AUTOSTART_AVAILABLE`
    is false in debug builds: the switch is disabled with a note, and `set_autostart` refuses.
  - `ErrorKind` gained `system` (Windows refused; nothing changed), and `UiError::rejected`
    for a value the driver can't use.
  - Extra file: `SettingsView.test.ts`.

  Proof: `pwsh scripts/check.ps1` passed (gatekeeper; 125 Rust tests, 117 UI tests in 21
  files). As a mutation check, without the re-render of the switch after a refused change the
  autostart test failed (the switch stayed flipped). Dev build with `TT_DB=<scratch db>`,
  driven through simulated input, with screenshots in the session:
  - The hotkey changed to `Ctrl+Alt+Shift+K`: another process could then register
    `Ctrl+Alt+Shift+Space` but not `Ctrl+Alt+Shift+K`, and `Ctrl+Alt+Shift+K` from Notepad
    brought up the popup with focus.
  - `Ctrl+Foo` → "isn't a valid hotkey", not saved.
  - `Ctrl+Alt+Shift+J`, held by a scratch process → saved, and the AC-1 banner appeared;
    clearing the field (default) removed it.
  - Backup folder set to a scratch folder, keep 3, restart → `timetracker-…Z.db` in that
    folder; the database stayed where it was.
  - `Q:\TimeTracker\backups` (no such drive), restart → the warning banner, and Stop still
    worked.
  - The Startup switch is disabled in the dev build, with its note.

  Not checked here (owner, with the installed build): the autostart toggle in Startup apps,
  because a release build would have to replace the installed app or open the live database;
  and the backup folder on a real OneDrive path (a scratch folder was used instead, so no files
  went into OneDrive). Both move to the step 14 install.
- **Step 14, release and install (2026-10-19).**
  - The version was already 0.1.0 (workspace `Cargo.toml`, `tauri.conf.json`,
    `ui/package.json`), so no file changed for it.
  - There was no step 0 build to install over: this machine had no uninstall entry and no Run
    entry. But a live database already existed in `%LOCALAPPDATA%\ThorSoft.TimeTracker`
    (created 2026-10-17, written 2026-10-18, with two startup backups) and held real
    clients. The owner decided to keep it. The installer ran silently (`/S`) with exit code 0
    and registered "ThorSoft TimeTracker" 0.1.0 under HKCU, installed to
    `%LOCALAPPDATA%\ThorSoft TimeTracker`.
  - Autostart wasn't switched on, because the database already existed, so this wasn't a
    first run. The owner switches it on in Settings, which is also the step 13 check.

  Proof so far:
  - `pwsh scripts/check.ps1` passed (gatekeeper; 125 Rust tests, only `load_bench` ignored;
    117 UI tests in 21 files).
  - `load_bench` (release): load + check 46 ms (limit 500 ms); 35 MB (limit 50 MB); startup
    including the backup 85 ms; one `execute(Start)` 16 ms.
  - `cargo tauri build` → `Finished 1 bundle at: ...\target\release\bundle\nsis\ThorSoft
    TimeTracker_0.1.0_x64-setup.exe (2.72 MiB)`.
  - The installed app started on the live database. It took the startup backup
    `timetracker-20261019T185702Z.db`, showed no banner, and listed the existing recents
    (screenshot in the session).

  Manual checks by the owner on the installed build (2026-10-19): all passed.
  - the step 0 checks on this install (tray, hotkey from another app, second launch, starting
    hidden after sign-in once autostart is on)
  - the autostart switch, checked against Startup apps
  - the backup folder on a OneDrive path, then a restart
  - one real workday tracked, with the evening overview copied into the booking system

  The date pickers start the week on Sunday (bead `qvt.18`, found in step 12). The owner
  judged this not an issue and closed the bead. It gets raised again if it becomes a
  problem.
- **Stage 5 review fixes (2026-10-19).** Four fresh-context reviewers (tt-core; tt-store and
  tt-app; tt-cli and src-tauri; ui) reviewed `master...slice-1` against `REVIEW.md` and found
  7 Important findings and 21 nits. The owner chose to fix the Important ones plus seven nits,
  and to file the other nits as beads (`zw5`, `6lb`, `zpe`, `8q9`, `lej`). Each bug got a
  failing test first, or a reproduction in the dev build.
  - **Recents (R7).** A blank quick-switch query listed up to 50 used combos by recency before
    the A–Z part. It now lists only the newest `recents_n` there; older ones join the A–Z part.
  - **Backup folder (R22).** A relative folder depended on the working directory (the install
    folder from the Start menu, `System32` on autostart). `validate_settings` now rejects it
    with `SettingProblem::RelativeBackupDir`. The invariant check on load uses the same rule,
    so a database that already holds a relative folder opens as Broken.
  - **Popup (R6).** Alt+F4 destroyed the `quick-switch` window, and the hotkey showed nothing
    until a restart. A close request now hides it, like `main`.
  - **tt-cli (OQ-5).** The live-database refusal compared path text only. It now strips a
    `\?\` prefix, resolves `.`/`..`, and compares canonical paths (of the files, or of their
    folders while the file doesn't exist), so a junction or an 8.3 name doesn't get through.
  - **Capabilities.** `core:default` gave both pages the tray and menu commands, and the popup
    the clipboard. They are narrowed as pinned above; `capabilities/main.json` is new.
  - **Today after midnight (R10, R14).** DayBar asked for today only when mounted, and the
    views outlive the night (closing only hides the window). It now asks again when the
    window gets focus or becomes visible, and a view that showed the old today moves on.
  - **Task typed + Enter (R3).** Enter within AutoComplete's 300 ms search delay started the
    timer (or saved the entry) without the typed task. TaskInput now runs the search for that
    text, picks its first offer and submits the form, as the popup does. Enter on a list that
    has answered stays with AutoComplete, so an arrow-key pick wins.
  - **Nits.** `backup_to` removes its `.partial` file on every failure, and pruning is a
    separate `prune_backups`: a failed prune is its own warning, never a failed backup (so it
    can't block a migration). `load_bench` fails when memory can't be measured, instead of
    reading 0. The popup resets its highlighted row when it hides. A failed day load in Today
    or the overview no longer shows the previous day's data. The file list above names
    `backend.rs`/`errors.rs` instead of `ctx.rs`, and the tray comment states the 127-unit
    limit correctly.
  - New test helpers in `ui/src/test/page.ts`: `typeKeys` (types without the `change` event
    that `setValue` fires; a browser fires it only on Enter or blur) and `pressEnter` (the
    keydown, then the implicit form submission happy-dom lacks).

  Proof: `pwsh scripts/check.ps1` passed (gatekeeper; 129 Rust tests, only `load_bench`
  ignored; 124 UI tests in 21 files). `load_bench`: 46 ms, 33 MB. Each new test failed before
  its fix, by running it against the old code or a mutation:
  - `quick_switch_lists_only_recents_n_recents_for_a_blank_query`
  - both live-database refusal tests in tt-cli
  - `a_failed_backup_leaves_no_partial_file`
  - the midnight and failed-day tests in `OverviewView.test.ts` and `TodayView.test.ts`
  - the highlight test in `QuickSwitch.test.ts`
  - the Enter-before-search tests in `TaskInput.test.ts` and `TodayView.test.ts`

  Dev build on a tt-cli worked-day scratch database, driven through simulated input:
  - Alt+F4 on the popup destroyed it before the fix; after it, the popup was only hidden, and
    the next hotkey showed it again.
  - With the narrowed capabilities, a switch from the popup updated the main window live
    (events), Copy put `1,00` on the clipboard, and Esc still hid the popup.
  - Picking "Beta / Ops", typing `Docs` and pressing Enter at once started "Beta / Ops /
    Docs" with the task created.

  Re-review of `441f084` (fresh context, only the fix commit): `important=0 nit=3`.
  - TaskInput's Enter path shared the `latest` counter with `complete()`. If AutoComplete's own
    delayed search fired while Enter waited, Enter was lost; a failed search lost it too. Now
    only typing on makes Enter give up, and a failed search leaves the list to AutoComplete.
    Test: `still picks when AutoComplete's own search fires while Enter waits`.
  - DayBar asked for today only on focus and visibility, so a window that stayed open and
    focused over midnight kept the old day. It now also asks once a minute. Tests: a TodayView
    midnight test with a minute passing, and a `visibilitychange` test in the overview.
  - Accepted (owner, 2026-10-19): a database that already holds a relative backup folder
    opens as Broken and can't be repaired in the app. Only 0.1.0 could have saved one, and it
    is installed only on the owner's machine, with an absolute OneDrive path and test data.
    No repair migration.
  Proof: `pwsh scripts/check.ps1` passed (gatekeeper; 129 Rust tests, only `load_bench`
  ignored; 127 UI tests in 21 files). Both new tests failed before their fix.
