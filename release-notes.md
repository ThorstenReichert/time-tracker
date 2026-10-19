# Release notes

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
