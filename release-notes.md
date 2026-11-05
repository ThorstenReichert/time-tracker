# Release notes

## 0.5.0 (unreleased)

The Archive view, from [changes/2026-11-05-archive-view](changes/2026-11-05-archive-view/plan.md).

- **Source:** branch `feat/archive-view`, not merged yet.
- **Database:** no migration. 0.5.0 and 0.4.0 read the same database file.

### What's in it
- An **Archive** view in the main window lists the archived projects, grouped by client
  (R2e). Each project shows its active period, from the day of its first entry to the day of
  its last (by start date, like the other views). Projects with no entries come first and show
  "No entries". The others are ordered newest period first, by its first day, then its last
  day, then the name.
- **Unarchive** on each archived project, with a confirmation (R2f).
- tt-cli: `archive` prints the same list as JSON.

### Rollback
- **App:** uninstall it in Settings → Apps, then install the 0.4.0 installer (or build tag
  `v0.4.0`). No data step is needed, because 0.5.0 doesn't change the database schema.

## 0.4.0 (2026-11-05)

Archiving projects, from [changes/2026-11-05-archive-projects](changes/2026-11-05-archive-projects/plan.md).

- **Source:** tag `v0.4.0` on `master`, which is the merge commit `7f5ce1f` of branch
  `feat/archive-projects`. Approved to merge by Thorsten Reichert (owner), 2026-11-05, after
  the stage 5 review against [REVIEW.md](REVIEW.md) and a manual test.
- **Installer:** `ThorSoft TimeTracker_0.4.0_x64-setup.exe` (NSIS, per user, unsigned),
  SHA-256 `CDD71D77C0FAC343AD1283EBD7D2D8F19FBC59982A1DB4B5C369D4724945EB05`.
- **Database:** no migration. 0.4.0 and 0.3.0 read the same database file. 0.3.0 already hides
  archived projects from its pickers, but it doesn't reject booking onto them.

### What's in it
- **Archive** on each project in Clients & projects, with a confirmation (R2a). An archived
  project is hidden there and no longer offered by the quick-switch, the combo pickers, the
  recents, the tray menu or the task input. Its entries stay in the entry lists, the daily
  overview and the statistics.
- An archived project can't be booked onto: starting or switching to it, adding an entry on it,
  moving an entry onto it and creating a task under it are rejected. Entries already on it can
  still have their times and note fixed, be moved to another project, or be deleted (R2b).
- The project of the running timer can't be archived: stop the timer first.
- **New project** checks the name and abbreviation while you type (R2c). If they match an
  archived project, the dialog shows it, with its name and abbreviation, and the button becomes
  **Unarchive project**, which brings it back as it was. A clash with an active project, or
  with more than one project, is shown inline and blocks the button. Names stay unique across
  archived and active projects.
- Duplicate-name messages say when the clashing project is archived (R2d).
- The recents always show N combos when there are N active ones, however many newer combos are
  archived (R7).
- tt-cli: `project archive <id>` and `project unarchive <id>`.
- Not yet: a view that lists archived projects, archiving clients or tasks.

### Rollback
- **App:** uninstall it in Settings → Apps, then install the 0.3.0 installer (or build tag
  `v0.3.0`). No data step is needed, because 0.4.0 doesn't change the database schema.
  Archived projects stay archived.

## 0.3.0 (2026-11-01)

ThorSoft branding: the logo and a matching colour theme, light and dark, from
[changes/2026-11-01-branding](changes/2026-11-01-branding/plan.md).

- **Source:** tag `v0.3.0` on `master`, which is the merge commit `f9cbf9c` of branch
  `feat/branding`. Approved to merge by Thorsten Reichert (owner), 2026-11-01, after the
  stage 5 review against [REVIEW.md](REVIEW.md).
- **Installer:** `ThorSoft TimeTracker_0.3.0_x64-setup.exe` (NSIS, per user, unsigned),
  SHA-256 `266E18E5EE389E2D1A7B281541D976F3A0DFBBADC4F3235BC4FB22A72B231071`.
- **Database:** no migration. 0.3.0 and 0.2.0 read the same database file.

### What's in it
- The ThorSoft logo in the navigation bar (with the word "TimeTracker"), in a new "About"
  section at the end of Settings with the version, and in the quick-switch popup (R25).
- A colour theme from the logo. Light mode stays white, with the brand blue as accent instead
  of green. Dark mode follows Windows and uses navy backgrounds, a light blue accent and a
  lighter logo.
- Error text in light mode is a darker red, so it is easier to read.
- The app, installer and tray icons are unchanged.

### Rollback
- **App:** uninstall it in Settings → Apps, then install the 0.2.0 installer (or build tag
  `v0.2.0`). No data step is needed, because 0.3.0 doesn't change the database.

## 0.2.0 (2026-11-01)

Abbreviations for clients, projects and tasks, from
[changes/2026-11-01-abbreviations](changes/2026-11-01-abbreviations/plan.md).

- **Source:** tag `v0.2.0` on `master`, which is the merge commit `a46e274` of branch
  `feat/abbreviations`. Approved to merge by Thorsten Reichert (owner), 2026-11-01, after the
  stage 5 review against [REVIEW.md](REVIEW.md).
- **Installer:** `ThorSoft TimeTracker_0.2.0_x64-setup.exe` (NSIS, per user, unsigned),
  SHA-256 `9B9D3C1EA7511A1D90A1DA19E7F3F40B064C8B1C106B8739E6EEAF4CF677B3C9`.
- **Database:** adds schema migration `0002` (abbreviation columns and their unique indexes).
  The first start of 0.2.0 backs up the database, then migrates it. **0.1.0 can't open the
  migrated database**: it starts with an "unknown migration 2" error and accepts no writes.

### What's in it
- Optional abbreviations (up to 20 characters) for clients, projects and tasks, set in the
  Clients view (R24).
- The quick-switch popup, combo pickers, recents, running bar, stop-at dialog and tray show
  the short label, e.g. `NWI / ASCP / Review`. Hovering shows the full label. The daily
  overview, entry lists and the Clients view keep the full names.
- Typing matches abbreviations as well as names, and an exact abbreviation ranks first (R3,
  R6). In the task input, typing an existing abbreviation doesn't offer to create a task.
- Names and abbreviations share one namespace per parent, ignoring case: a new name or
  abbreviation can't equal a sibling's name or abbreviation (R1).
- The value to book on the daily overview is the copy button itself (R15, since 0.1.0).

### Rollback
- **App:** uninstall it in Settings → Apps, then install the 0.1.0 installer (or build tag
  `v0.1.0`). 0.1.0 needs the database from before the migration, see Data.
- **Data:** the first start of 0.2.0 wrote a backup (`timetracker-YYYYMMDDTHHMMSSZ.db`) to
  the backup folder before migrating. To go back to 0.1.0: quit the app from the tray, delete
  the `-wal` and `-shm` files next to `%LOCALAPPDATA%\ThorSoft.TimeTracker\timetracker.db`,
  then copy that backup over `timetracker.db`. Entries made with 0.2.0 after the backup are
  lost that way.

## 0.1.0 (2026-10-19)

Slice 0+1 from [plan.md](plan.md): the thin vertical slice, ready for daily tracking.

- **Source:** tag `v0.1.0` on `master`, which is the merge commit `0e1354d` of branch
  `slice-1`. Approved to merge by Thorsten Reichert (owner), 2026-10-19, after the stage 5
  review against [REVIEW.md](REVIEW.md).
- **Installer:** `ThorSoft TimeTracker_0.1.0_x64-setup.exe` (NSIS, per user, unsigned),
  SHA-256 `E3173AF24B3EE43A181EEEE0BC734092718FC368066BFB79A0A3A4AEA7437FEA`.
- **Database:** schema migration `0001` only. This release adds no migration, so earlier
  0.1.0 builds and this one read the same file.

### What's in it
- Clients, projects and tasks with rounding rules per client (R1, R17, R18). Tasks can be
  created while typing (R3).
- Start, stop and switch; at most one running timer and no overlaps (R4, R5).
- The quick-switch popup on a global hotkey (R6), recents (R7), the tray (R8), single instance
  and autostart (R9).
- Entries: list per day, add, edit, delete, a forgotten stop, notes (R10–R13).
- The daily overview with values to book, raw and diff, copied as numbers (R14–R16, R19).
- Startup backups to a configurable folder, pruned to the newest N (R22, startup part).

Not in this release (slice 2): archiving (R2), favorites (R7), R20, R21, the daily backup
timer (R22) and export/restore (R23).

### Changes since the first 0.1.0 install (stage 5 review fixes)
- With nothing typed, quick-switch shows only the newest N recents (the Settings value), then
  every other combo A–Z.
- The backup folder must be a full path.
- Alt+F4 on the popup hides it; before, the hotkey stopped working.
- Today and the daily overview move on to the new day after midnight.
- Enter right after typing a task name uses that task.
- Narrower window permissions, plus cleaner backup failures and pruning.
- Details: plan.md, Deviations, "Stage 5 review fixes".

### Status after the release
Installed on the owner's machine on 2026-10-19, over the first 0.1.0 install. It started on the
existing database with no warning banner. After this install, the HKCU Run key had no
TimeTracker entry; check the Startup switch in Settings.

### Rollback
- **App:** uninstall it in Settings → Apps, then install a build of an earlier commit (for the
  pre-review 0.1.0: `git switch --detach 3d23a9b`, then `cargo tauri build`). The database
  stays where it is and is compatible (same schema).
- **Data:** each start writes a backup (`timetracker-YYYYMMDDTHHMMSSZ.db`) to the backup
  folder. Restore in the app (R23) doesn't exist yet. To go back by hand: quit the app from the
  tray, then copy a backup over `%LOCALAPPDATA%\ThorSoft.TimeTracker\timetracker.db`. First
  delete that file's `-wal` and `-shm` files, so SQLite doesn't replay them onto the restored
  copy. Then start the app again.
