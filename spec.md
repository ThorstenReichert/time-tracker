# Spec: Personal project time tracker
Source: [intent.md](intent.md). Author: Thorsten Reichert. Status: approved (Thorsten Reichert, owner, 2026-10-17).
Date: 2026-10-16. Skills applied: none (personal project, no organisation skills).
Revised 2026-10-17: §2 architecture changed to a deterministic functional core (Decider),
the tt-app write path and a headless tt-cli driver, for testability. Second pass the same
day: R4 and §2 corrected after an external review; the decisions it raised are recorded as
OQ-3 to OQ-8 in §4. Third pass the same day: the storage seam was reviewed (whole State in
memory vs. ports and adapters vs. prefetch); recorded as OQ-9. The write path now evolves a
candidate State before the commit and reloads from the database on a store error.
Revised 2026-10-18 (owner): R6, with nothing typed the list continues after the recents with
the other active combos, so a new catalog is offered before anything is tracked.
Amended 2026-11-01 by [changes/2026-11-01-abbreviations/spec.md](changes/2026-11-01-abbreviations/spec.md):
R24 (abbreviations) and amendments to R1, R3 and R6.
Amended 2026-11-01 by [changes/2026-11-01-branding/spec.md](changes/2026-11-01-branding/spec.md):
R25 (ThorSoft logo and a matching colour theme, light and dark) and an amendment to §2 "Frontend".
Amended 2026-11-05 by [changes/2026-11-05-archive-projects/spec.md](changes/2026-11-05-archive-projects/spec.md):
R2 for projects (R2a–R2d) and an amendment to R7 (recents). OQ-7 is unchanged.
Amended 2026-11-05 by [changes/2026-11-05-archive-view/spec.md](changes/2026-11-05-archive-view/spec.md):
R2e (Archive view of archived projects with their active period), R2f (unarchive from it) and
an amendment to R2a (archive without a confirmation).
Amended 2026-11-05 by [changes/2026-11-05-archive-clients/spec.md](changes/2026-11-05-archive-clients/spec.md):
R2 for clients (R2g–R2k) and amendments to R2e/R2f (archived clients in the Archive view).
OQ-7 confirmed for clients.
Revised 2026-11-05 (owner): R10, the day's entries are listed newest first.
Revised 2026-11-05 (owner): R24, an abbreviation can have up to 50 characters (was 20).

Requirements and design in one document (SDLC stage 2). Requirements are numbered `R<n>`,
open questions `OQ-<n>`, areas of concern `AC-<n>`, so the plan and the reviews can cite them.

## Summary of decisions
| Topic | Decision |
|---|---|
| App type | Windows desktop app: **Rust + Tauri 2**, UI in **Vue 3 + TypeScript** |
| Core architecture | Deterministic functional core (Decider), single write path, drivable by UI, CLI and tests |
| Priority | Learning Rust counts for more than delivery speed; a v1 in a few weeks is fine |
| Storage | Local **SQLite**, automatic rolling backups, manual JSON/CSV export |
| Booked hours | Always computed from the current entries and rules; no "booked" state |
| Past edits / rule changes | Recomputed retroactively |
| Rounding scope | Strictly per client; no per-project override |
| Midnight | An entry counts entirely on its start day (local time) |
| Breaks | A gap with no timer running is a break; there is no pause concept |
| Hour format | Decimal hours (e.g. `1.25`) |
| Shell | Lives in the tray, starts with Windows, global hotkey opens a quick-switch popup |
| "Nearest" ties | Round half up |

---

## 1. Requirements

### Structure
- **R1 Hierarchy.** Clients → projects → tasks. A project belongs to exactly one client and a
  task to exactly one project. A time entry always references a project. The task is optional.
  Names are unique within their parent (case-insensitive).
- **R2 Archiving.** Clients, projects and tasks can be archived and unarchived. Archived items
  (and the children of an archived parent) don't appear in quick-switch, favorites, recents or
  pickers for new entries. They stay visible in existing entries, in the daily overview,
  statistics and billing error. Items with entries can't be deleted, only archived. Unused
  items can be deleted. Archived names still count: a new item whose name matches an archived
  sibling isn't created; the UI offers to unarchive the existing item instead.
  *Projects: detailed as R2a–R2d in [changes/2026-11-05-archive-projects/spec.md](changes/2026-11-05-archive-projects/spec.md).*
  *The Archive view: R2e–R2f in [changes/2026-11-05-archive-view/spec.md](changes/2026-11-05-archive-view/spec.md).*
  *Clients: R2g–R2k in [changes/2026-11-05-archive-clients/spec.md](changes/2026-11-05-archive-clients/spec.md).*
- **R3 Quick task creation.** Wherever a task is chosen, a free-text input matches the
  project's active tasks as you type (case-insensitive substring/fuzzy). If no existing task
  matches exactly, it offers **Create "<text>"**, which creates the task and selects it in one
  step.

### Tracking
- **R4 Single running timer.** At most one entry is running (no end time) at any time. The
  core rejects a second running entry; a partial unique index in the database is the
  backstop (§2).
- **R5 Start / stop / switch.** *Start* creates a running entry for a client/project/task
  combo at "now". *Stop* sets the running entry's end to "now". *Switch* stops the running
  entry and starts the new one at the same instant, so there is no gap and no overlap.
  Switching to the combo that is already running does nothing. Times are recorded to the
  second.
- **R6 Global quick-switch.** A configurable global hotkey (default `Ctrl+Alt+Space`) opens
  a small quick-switch popup from any application:
  - It shows the running entry (combo and elapsed time) at the top, then favorites, then
    recents, then the other active combos (A–Z). The same list serves the combo pickers in
    the main window.
  - Typing fuzzy-filters over "Client / Project / Task" combos (existing tasks plus
    project-only combos) of active items.
  - `Enter` switches to the highlighted combo and closes the popup. `Tab` on a project
    moves to a task input with R3 behaviour. `Esc` closes without changes.
  - A **Stop** action (shortcut inside the popup, e.g. `Ctrl+S`) stops the running timer.
  - Target: a known combo is running again within about 3 seconds / about 5 keystrokes
    after pressing the hotkey.
- **R7 Favorites and recents.** You can pin combos as favorites (ordered). Recents are the
  last N distinct combos used (default N = 8). Both are one click in the main window and
  listed first in the popup.
  *Recents count active combos only: amended by [changes/2026-11-05-archive-projects/spec.md](changes/2026-11-05-archive-projects/spec.md).*
- **R8 Tray.** The tray icon shows whether a timer is running or idle. Its tooltip shows the
  current combo and the elapsed time. The tray menu offers Stop, the recents/favorites,
  Open window and Quit. Closing the main window hides it to the tray. The app quits only
  via the tray.
- **R9 Startup.** An option (on by default) starts the app with Windows. Only one instance
  can run: launching it again brings the running instance to the front.

### Manual corrections
- **R10 Entry management.** Entries can be added after the fact (combo, start, end, note).
  Start, end, combo and note can be edited, and entries can be deleted (with a confirmation
  or undo). The main window lists the entries for a selected day, newest first, so the
  running entry is on top.
- **R11 Forgotten stop.** The running entry's start can be edited, and it can be stopped at
  a chosen past time instead of "now".
- **R12 Validation.** End must be after start. Entries can't start or end in the future. No
  two entries may overlap, and a running entry counts as lasting until now. An invalid save is
  rejected with a message naming the conflicting entry. No silent auto-adjustment.
- **R13 Notes.** Each entry has an optional free-text note. Notes appear in entry lists and
  exports, not in the billing overview.

### Daily billing overview
- **R14 Overview.** For any date (default today, with quick previous/next day navigation)
  it shows, per client with time on that day:
  - *Client-level* clients: one **rounded client total** (the value to book), plus raw
    client total. The per-project raw figures appear as informational rows without
    rounding.
  - *Project-level* clients: a **rounded total per project** (the values to book) and the
    raw value per project, plus a client subtotal of the rounded project values.
  - The raw value is always shown next to each rounded value, plus the difference.
  - A running entry counts up to "now" and is marked as running.
- **R15 Format and copy.** Values are shown in decimal hours with 2 decimals. Every value
  to book has a copy-to-clipboard button that copies just the number. The decimal separator
  is a setting (`.` or `,`, defaulting to the OS locale, see AC-3).
- **R16 Rounding scope.** Rounding is applied only in the daily billing overview (R14) and
  the billing error (R21) derived from it. Stored entries and statistics (R20) are never
  rounded.

### Rounding rules
- **R17 Per-client rule.** Each client has:
  - *unit*: minutes, one of 6, 12, 15, 30, 60. Default 15. The other divisors of 60 give
    values that aren't exact at 2 decimals (AC-3), so they are not offered.
  - *direction*: `up`, `nearest` or `down`. Default `nearest`.
  - *level*: `project` or `client`. Default `project`.
- **R18 Algorithm.** For a given day:
  1. Assign every entry to the local date of its **start** (an entry crossing midnight
     counts entirely on its start day).
  2. Sum the raw durations in **seconds** per group. The group is a project for
     project-level clients and the client for client-level clients.
  3. Round each group sum `t` to a multiple of `u = unit × 60` seconds:
     `down = ⌊t/u⌋·u`, `up = ⌈t/u⌉·u`, `nearest = ⌊(t + u/2)/u⌋·u` (exactly half rounds up).
  4. A zero sum stays zero. Groups with no entries don't appear.

  The rounding is a pure, deterministic function, and its unit tests use worked examples
  from this spec.
- **R19 Retroactive rules.** The overview and billing error are always computed from the
  current entries and the current rules. Changing a rule or editing a past entry changes
  the past figures (see AC-4).

### Statistics
- **R20 Raw statistics.** For a selectable range (this/last week (ISO, Monday start),
  this/last month, custom from/to) it shows raw tracked totals per client and per project,
  as a table and as charts:
  - a stacked bar chart per day, by client;
  - a share per client/project (bar or donut).

  Day assignment is the same as R18 step 1. Archived items are included. No rounding.

### Billing error
- **R21 Billing error.** Per day and per billable group, `error = rounded − raw` (signed:
  positive means more hours billed than worked). It is shown:
  - per client, for every client. For project-level clients this is the sum of the project
    errors.
  - per project, for project-level clients only (client-level clients have no per-project
    rounded value).
  - per day and summed over a selectable range (same ranges as R20), as a table and as a
    chart (per-day bars, cumulative line).

  The error is derived on the fly and never stored.

### Data
- **R22 Automatic backup.** On startup and then once a day while running, the app writes a
  consistent copy of the database to a configurable backup folder (default:
  `%LOCALAPPDATA%\ThorSoft.TimeTracker\backups`; may point to OneDrive). It keeps the last N
  copies (default 14).
- **R23 Export / restore.** Manual export of everything as JSON and of the entries as CSV
  (date, start, end, duration in decimal hours, client, project, task, note). You can
  restore from a selected backup. Before replacing the live database, a backup of the
  current one is taken.

### Exclusions (out of scope for v1)
From intent: invoicing, rates and monetary values; multi-user, accounts and permissions;
automatic activity tracking; work-hours/overtime/vacation accounting; push into booking
systems; import from Clockify or other tools.
Decided in this spec: billable flag, tags, pause/resume, per-project rounding overrides,
a "booked" state or frozen snapshots, a long-running-timer warning (OQ-1), and any network
or cloud sync feature.

---

## 2. Design

### Architecture
A single Tauri 2 desktop app. The Rust side is a Cargo workspace built as a **functional core
with an imperative shell**: all domain decisions live in a pure, deterministic core. The app
is driven through one write path by the UI, the global hotkey and automated tests (headless
CLI for scenario tests on scratch databases). The core follows the **Decider** pattern, and its determinism follows TigerBeetle's
approach: everything non-deterministic (clock, time zone, IDs, IO) is passed in, never read by
the core. The goal is testability: given/when/then and model-based property tests without
mocks (see Testing approach).

```
ThorSoft.TimeTracker/
├─ crates/
│  ├─ tt-core/     domain types, Decider (decide/evolve), invariants (R1, R2, R4, R12),
│  │               rounding (R17–R19), read models: daily overview, statistics,
│  │               billing error, quick-switch matching (R6). Pure and deterministic, no IO.
│  ├─ tt-store/    SQLite persistence (rusqlite, bundled), embedded SQL migrations,
│  │               load State, apply Events in one transaction, backup/restore
│  │               (SQLite online backup API), export.
│  ├─ tt-app/      the single write path: owns State + Store behind one lock, executes
│  │               Commands, publishes Events. Shared by every driver.
│  └─ tt-cli/      headless driver for scenario tests, on scratch databases only.
├─ src-tauri/      GUI driver: Tauri commands (thin: map UI calls → tt-app), tray, global
│                  shortcut, autostart, single-instance, clipboard, UI DTO types (specta).
└─ ui/             Vue 3 + TypeScript + Vite frontend.
```

- **Dependency direction:** `src-tauri, tt-cli → tt-app → tt-store → tt-core`. `tt-core`
  depends on no IO crate (no Tauri, no rusqlite, no `std::fs`/`std::net`). The UI never
  touches the database; it only calls Tauri commands.
- **No port traits.** tt-app uses the concrete tt-store, and tests use a temporary SQLite
  database rather than a fake store. Determinism comes from passing `Ctx` in, not from
  injected clocks.
- **Core shape (Decider):** `decide(&State, &Ctx, &Command) -> Result<Vec<Event>, DomainError>`
  and `evolve(State, &Event) -> State`. `Command`, `Event` and `State` are plain `serde`
  values. Every change to State is an Event produced by `decide` and applied by `evolve`.
  Events are in-memory descriptions of change; the database stores the resulting State
  (OQ-3). Undo (R10) is an inverse command through the same write path.
- **Everything is an input:** `Ctx { now, tz }` carries the current instant (UTC seconds) and
  the time zone; the driver builds it. `decide` uses `tz` only to convert typed local times
  (R10, R11) to UTC; an ambiguous or nonexistent local time is rejected (OQ-6). IDs are assigned by
  the core from counters in `State`. tt-core never reads the system clock or OS time zone, and
  uses no randomness, env vars, IO, threads or global mutable state. Same inputs give identical
  outputs. `State` uses ordered maps (`BTreeMap`), so serialisation is stable.
- **Whole world in memory:** `State` holds all clients, projects, tasks, entries, favorites,
  settings and counters, loaded from SQLite once at startup (single user; tens of thousands of
  entries are a few MB). The core enforces the invariants itself; the SQLite constraints below
  are a backstop where one exists. `State::check_invariants()` runs after every `evolve` in
  debug builds and tests, and on every load in all builds. If the loaded State is invalid, the
  app shows an error and accepts no writes.
- **Keep a prefetch migration open (OQ-9):** `State` is split into `Catalog` (clients,
  projects, tasks, favorites, settings, counters; small and always fully loaded) and `Entries`
  (the only part that grows). `decide` reaches entries only through named queries that SQL
  could answer with an index (e.g. running entry, entries overlapping a range, recent combos),
  never by iterating the map directly. Read models take the entries they need as an input
  rather than the whole `State`. Events are self-contained: tt-store applies them without
  looking anything up in `State`. No business rule may need all entries at once.
- **Reads:** the daily overview, statistics, billing error (R14–R21) and quick-switch matching
  (R6) are pure functions over the `Catalog`, the entries they need, `now`, `tz` and the date,
  range or query. They don't go through `decide`. A running entry counts up to `now` wherever
  it falls in a read (R14).
  Ranges are inclusive local dates; days are assigned by start (R18 step 1).
- **Write path (tt-app):** `execute(Ctx, Command)`. The driver builds `Ctx` from the system clock
  and time zone; tt-app never reads them. Under one lock: `decide` → `evolve` a copy of the
  State into a candidate (`check_invariants` on it in debug builds and tests) → one database
  transaction that applies the events in order → commit → replace the State with the
  candidate → publish the events. A bug that panics in `evolve` therefore fails before
  anything is written, and the swap itself can't fail. The database is the authority: on any
  store error during the transaction or commit, tt-app discards the candidate and reloads
  State from the database instead of assuming it is unchanged (a failed commit can leave its
  outcome unclear). If that reload fails, the app shows an error and accepts no writes, as for
  an invalid State. Reads take the same lock, so they never see a half-applied command. No
  business rules live in tt-app, the drivers or the UI.
- **Restore (R23)** is the one exception to "every change is an Event". Under the same lock it
  backs up the current database, replaces the file from the chosen backup, reloads State and
  checks its invariants.
- **Startup order:** take the exclusive lock → if a database exists, take the R22 backup →
  run migrations → load State and check invariants → start the drivers. The backup must come
  before migrations, or a bad migration has no protection.
- **Single writer:** one process per database file, enforced by an exclusive lock. `tt-cli`
  requires `--db <path>` and never opens the live database.
- **Code style (Faux-O):** newtypes with validating constructors, illegal states made
  unrepresentable (e.g. a rounding unit can only be a divisor of 60), methods that return new
  values instead of mutating shared state.
- **Tauri plugins:** `global-shortcut` (R6), `autostart` (R9), `single-instance` (R9),
  `clipboard-manager` (R15). The tray uses the built-in Tauri 2 tray API (R8). Autostart
  state is read from the plugin (the registry entry), which is the only source of truth; the
  setting table has no autostart key, so Windows' Startup apps screen can't drift from the app.
- **Windows:**
  - `main`: normal window with Today/Entries, Daily overview, Statistics, Billing error,
    Clients & projects, Archive (R2e) and Settings views. Closing it hides it to the tray.
  - `quick-switch`: small, frameless, always-on-top, centred. The hotkey shows and focuses
    it; it hides when it loses focus. It is created hidden at startup so it opens instantly.
- **State sync:** `src-tauri` turns the Events published by tt-app into `data-changed`
  (and `timer-changed`) events. Both windows and the tray refresh from them. The backend
  holds the single source of truth.
- **Tray elapsed time (R8):** the tray has no UI to compute elapsed time, so `src-tauri`
  refreshes the tooltip once a minute while a timer runs, and on every `timer-changed` event.
- **Frontend:** Vue 3 (`<script setup>`, Composition API), vue-router, Pinia for UI state,
  PrimeVue components (AutoComplete, DatePicker, DataTable, Dialog), ECharts via
  `vue-echarts` for charts. Command and DTO types are generated with `specta`/`tauri-specta`
  from DTO structs in src-tauri that map from tt-core types, so the Rust↔TS boundary is
  type-checked and tt-core carries no UI derives.
- **Elapsed time display:** the running entry's elapsed time is computed in the UI from its
  start time. The UI doesn't poll the backend for it.

### Data model (SQLite)
```
client     (id, name, round_unit_min, round_direction, round_level, archived_at?, created_at)
project    (id, client_id → client, name, archived_at?, created_at)
task       (id, project_id → project, name, archived_at?, created_at, UNIQUE (id, project_id))
time_entry (id, project_id → project, task_id?, start_utc, end_utc?, note?,
            created_at, updated_at,
            FOREIGN KEY (task_id, project_id) → task (id, project_id))
favorite   (id, project_id → project, task_id?, sort,
            FOREIGN KEY (task_id, project_id) → task (id, project_id))
setting    (key, value)            -- hotkey, decimal separator, backup folder/N, recents N
counter    (name, next_id)         -- ID counters, written in the same transaction as the rows
schema_migrations
```
- **One running timer (R4):** `CREATE UNIQUE INDEX one_running ON time_entry((end_utc IS NULL)) WHERE end_utc IS NULL`
  (a partial index on a non-NULL expression, so the uniqueness check actually applies). The
  core rejects a second running entry before this index is reached.
- **Task belongs to its entry's project:** enforced by `decide` and by the composite foreign
  key above. A NULL `task_id` skips that check (SQLite's default MATCH SIMPLE), which is what
  project-only entries need.
- **Name uniqueness (R1):** checked by `decide`. Backstop: case-insensitive unique indexes on
  `client(name)`, `project(client_id, name)` and `task(project_id, name)`, each
  `COLLATE NOCASE`. Archived rows count too (OQ-7).
- **Overlap check (R12):** checked by `decide` against the in-memory State only. There is no
  database backstop; the Decider tests cover it.
- **Switch order:** a switch applies its events in this order: stop the running entry, then
  insert the new one. Uniqueness checks run per statement, so the order is fixed in the
  transaction, not left to chance.
- **Intervals are half-open `[start, end)`:** a switch shares a boundary with the entry it
  stops, which is not an overlap.
- **IDs:** assigned by tt-core from counters in State. The `counter` table is written in the
  same transaction as the rows, so an ID is never reused after a restart, even if the newest
  entry was deleted.
- **Recents (R7):** derived from the entries ordered by start (distinct combos, active
  only), not stored.
- `round_direction` and `round_level` are stored as text enums with a `CHECK` constraint.

### Time handling
- Instants are stored as UTC, as Unix seconds (`INTEGER`). Durations are integer seconds.
- The driver sets `Ctx.tz` from the OS time zone. It is used for local dates (R18 step 1,
  R20, R21) and for converting typed local times (R10, R11) to UTC. tt-core never looks the
  zone up itself. A typed time in the repeated hour (DST end) or the skipped hour (DST start)
  is rejected with a message, as R12 requires for any invalid save; the user retypes it.
  The time library (`jiff` or `chrono`) is chosen in the plan stage.
- Decimal hours are formatted only at the edge (UI/export). All calculations stay in
  seconds, including the R14 difference and the R21 error. The displayed 2-decimal values
  can still differ from exact hours (see AC-3).

### Storage location and backup
- The live database is `%LOCALAPPDATA%\ThorSoft.TimeTracker\timetracker.db`, never inside a
  synced or roamed folder (AC-5, OQ-8). It uses WAL mode.
- Backups use the SQLite online backup API to write a timestamped file to the backup
  folder, then prune older copies (R22).

### Security and privacy
- No network access is required or used. No telemetry.
- Tauri capabilities are limited to the commands and plugins listed above, with a strict
  CSP and no remote content.
- All data stays in local files the user controls.

### Testing approach (detailed in stage 4)
- `tt-core`: unit tests for every rounding direction, level and the tie case, worked
  examples from R18, and property tests (`proptest`): rounding is idempotent, `down ≤ raw ≤ up`,
  `|nearest − raw| ≤ u/2`.
- `tt-core` Decider: given State / when Command / then Events-or-error tests for every
  command, including each rejection (R4, R12, R2 archiving rules), the switch event order,
  and ID assignment across a delete and a reload.
- `tt-core` model-based property tests: random Command sequences with a simulated clock and
  time zone; `check_invariants` after every step. This is deterministic simulation in
  spirit, without fault injection.
- `tt-core` read models: quick-switch matching and ranking (R3, R6) are tested in Rust.
- `tt-store` + `tt-app`: integration tests against a temporary database (migrations,
  backstop constraints, backup/restore round-trip, backup before migration). The same
  Command sequences as above: after a reload from disk, the serialised State must equal the
  in-memory State.
- `tt-cli`: scenario scripts against a temporary database.
- Load benchmark (OQ-9 trigger): a generated database with about 100k entries (roughly 10×
  the expected volume). Startup load plus invariant check stays under 500 ms, and process
  memory with the State loaded stays under 50 MB.
- `ui/`: Vitest for formatting only. End-to-end UI tests are deferred.

---

## 3. Constraint traceability
| Constraint (intent.md) | Satisfied by |
|---|---|
| Single user, runs locally on one machine; no team/sharing | Desktop app, no accounts or server (Design → Architecture); exclusions |
| Technology left open for design | Decided: Rust + Tauri 2, Vue 3 + TS, SQLite (Summary, Design) |
| Work data stays under my control; no third-party cloud required | Local SQLite in `%LOCALAPPDATA%`, no network (Design → Security), backups to a user-chosen folder (R22) |
| Rounding deterministic and transparent | Pure function with documented formula (R17, R18); raw shown next to rounded (R14); unit + property tests |
| Raw times always kept | Entries stored to the second, never modified by rounding (R5, R16) |
| Rounding only in the daily overview and the billing error | R16; statistics are raw (R20); billing error derives from R14 (R21) |
| Runs on my Windows work machine | Tauri 2 on WebView2 (ships with Windows 11); tray/autostart (R8, R9); AC-2 |
| Out of scope (invoicing, multi-user, auto-tracking, HR accounting, booking API, import) | Exclusions |
| Success: switching takes a couple of seconds/keystrokes | R6 target, R7, R8 |
| Success: stop using Clockify | Full coverage of the intent outcome: R1–R23 |

## 4. Open questions from intent
| # | Intent question | Resolution |
|---|---|---|
| 1 | Data storage & backup | **Answered:** local SQLite + automatic rolling backups + JSON/CSV export (R22, R23). No sync. |
| 2 | "Booked" hours computed or confirmed? | **Answered:** computed. No booked state or actual-booking override (R19, R21). |
| 3 | Recalculate past data on edits/rule changes? | **Answered:** always recompute (R19). Trade-off in AC-4. |
| 4 | Rounding overrides per project? | **Answered:** no, strictly per client (R17). |
| 5 | Entries spanning midnight | **Answered:** count entirely on the start day (R18 step 1). |
| 6 | Breaks | **Answered:** a gap with no timer running is a break; no pause concept. |
| 7 | Archiving | **Answered:** yes, at all three levels (R2). |
| 8 | Billable flag / tags / descriptions | **Answered:** free-text note per entry (R13). Billable flag and tags excluded. |
| 9 | Forgot-timer safeguards | **Carried forward as OQ-1.** |

Carried forward:
- **OQ-1 Forgot-timer safeguard.** Should the app warn when a timer runs past a threshold or
  overnight? Revisit after 2–4 weeks of real use.
- **OQ-2 Quick-switch tuning.** Recents count, fuzzy-match ranking (frequency vs. recency)
  and whether a second hotkey for "stop" is needed. Tune during use; defaults are in R6
  and R7.

Architecture decisions from the stage 2 review (2026-10-17):
- **OQ-3 Event persistence.** Decided: the database stores the State; Events are in-memory
  descriptions of change. Undo (R10) is an inverse command. No event table in v1.
- **OQ-4 Rounding units (R17).** Decided: only the units that are exact at 2 decimals (6, 12,
  15, 30, 60 minutes). See AC-3.
- **OQ-5 tt-cli.** Decided: kept as a test harness. It requires `--db` and runs only against
  scratch databases. No IPC to the running app in v1.
- **OQ-6 Local times at DST changes.** Decided: an ambiguous or nonexistent typed time is
  rejected with a message (R12). No silent choice of reading.
- **OQ-7 Archived names.** Decided: archived names still count for uniqueness (R1), and
  creating a clashing name offers to unarchive the existing item (R2). Confirmed for projects
  on 2026-11-05 (changes/2026-11-05-archive-projects, R2c), and for clients the same day
  (changes/2026-11-05-archive-clients, R2i).
- **OQ-8 Live database location.** Decided: `%LOCALAPPDATA%`, which isn't roamed or synced.
  The backup default follows (R22).
- **OQ-9 Storage seam.** Decided: whole `State` in memory with a pure Decider (as above).
  Options considered:
  - *Prefetch Decider* (TigerBeetle-style: the shell loads a command's read set, `decide`
    runs over that slice). Kept as the migration path, not built in v1. Only entries grow, so
    a migration would keep the `Catalog` in memory and load `Entries` by range. The rules in
    "Keep a prefetch migration open" (§2) keep that change inside tt-core's `Entries` type
    and tt-app. Remaining cost if it happens: a read-set step in tt-app, invariant checks
    only over the loaded window plus the SQL constraints, and R12 relying on the Decider
    tests alone (it has no database backstop).
  - *Ports and adapters* (the core calls a store interface). Rejected: IO errors inside
    domain decisions, Events would no longer be the output (tests, UI notifications, undo),
    and core tests would need a database or a fake. The Event→SQL mapping would move into
    the core, not go away.
  - *Event log as the source of truth.* Rejected for the reasons in OQ-3.

  Accepted risk: State in memory and the database are two copies that could drift; the
  reload-equality test guards it. Trigger for migrating: the load benchmark in Testing
  approach fails (load over 500 ms or memory over 50 MB at 100k entries).

## 5. Areas of concern
Owner of all items: Thorsten Reichert (single-user project).

- **AC-1 Hotkey conflicts.** `Ctrl+Alt+Space` may already be taken by another app or a
  corporate tool. *Mitigation:* the hotkey is configurable. If registration fails, the app
  shows an error in the main window and tray tooltip instead of failing silently.
- **AC-2 Corporate machine policy.** An unsigned executable, SmartScreen, an autostart
  registry entry or AppLocker may block the app on the work machine. *Mitigation:* verify
  early in the build stage with a locally built installer (MSI/NSIS) on the actual machine,
  before building features. Check with IT if it is blocked.
- **AC-3 Decimal format for booking systems.** German booking systems may expect `1,25`
  rather than `1.25`. Units that aren't exact at 2 decimals (1, 5, 10, 20 minutes) would
  show 10 minutes as 0.17 h, which books 10.2 minutes. The billing error stays exact, because
  calculations use seconds, but what gets booked is the displayed value. *Mitigation:* the
  units are limited to the exact set (R17, OQ-4), and the separator is a setting (R15).
  Confirm the needed format in the first week of use.
- **AC-4 Retroactive recompute.** Editing an old entry or changing a client's rule changes
  past daily overviews and billing errors, which may then no longer match what was
  actually booked. *Accepted* for v1 (simplicity). If it causes problems, reopen intent
  questions 2 and 3 (a "booked" snapshot per day).
- **AC-5 Synced-folder corruption.** A live SQLite file inside OneDrive can be corrupted
  by the sync client. *Mitigation:* the live database stays in `%LOCALAPPDATA%`, which is not
  roamed or synced (OQ-8); only the backup copies go to the (possibly synced) backup folder.
- **AC-6 Learning curve.** Rust, Tauri and Vue are all new at the same time and could
  stall progress. *Mitigation:* keep `tt-core` small and pure as the main Rust learning
  ground. Build a thin vertical slice first (clients/projects, start/stop/switch with the
  hotkey popup, daily overview), then add statistics, billing error and backups.
