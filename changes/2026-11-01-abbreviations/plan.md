# Plan: Abbreviations for clients, projects and tasks
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-01). Branch: `feat/abbreviations`. Target release: 0.2.0.
Scope: R24, plus the amendments to R1, R3 and R6. Deferred: the notes for R2 and R23 (slice 2).

## Context
Long real names make the compact places unwieldy. The owner decided where to show the short
label (only the compact places) and how unique an abbreviation must be (shared namespace with
names). Both decisions are recorded in spec.md.

## Risks
- **Riskiest step: the migration (step 3).** It is the first migration that runs on the live
  database. De-risked by the startup backup before migrations, a test that migrates a 0001
  database, and a dry run on a copy of the live database.
- **Uniqueness spans two fields.** A rename can clash with a sibling's abbreviation. The core
  checks both directions; the DB backstop covers abbreviation vs abbreviation only.
- **Labels are also built in the UI** (`projectOnly()` in `ui/src/lib/combo.ts`). It must build
  the short label too, or a picked project shows the long label.
- **Ranking (AC-7).** A test pins that an exact abbreviation ranks first.
- Rejected: matching the short label and the full label separately and taking the best score
  (mixed queries like `nwi calibration` would fail); a separate `SetAbbreviation` command (two
  writes for one dialog save).

## Steps
1. **tt-core domain (R24, R1).** `crates/tt-core/src/abbreviation.rs`: `Abbreviation` newtype
   (trimmed, no control characters, 1–20 characters, lowercase key). `catalog.rs`:
   `abbreviation: Option<Abbreviation>` (`#[serde(default)]`) on `Client`, `Project` and `Task`;
   the `*_named` lookups match name or abbreviation. `decide/catalog.rs`: the uniqueness checks
   cover the new name and the new abbreviation, excluding the item itself. Commands: an
   `abbreviation: Option<String>` on `CreateClient`, `UpdateClient`, `CreateProject` and
   `CreateTask`; `RenameProject`/`RenameTask` become `UpdateProject`/`UpdateTask`. Events:
   `ClientUpdated` gets the abbreviation; `ProjectRenamed`/`TaskRenamed` become
   `ProjectUpdated`/`TaskUpdated`. `evolve.rs` and `invariants.rs` follow. Tests: each clash
   direction, a clash with an archived item, blank abbreviation, length limit, no-op update.
2. **tt-core read models (R24, R3, R6).** `read/combo.rs`: `ComboRow` gains the abbreviation of
   each level and `short_label`. `read/quick_switch.rs`, `read/task_match.rs`: score a match text
   with names and abbreviations. `TaskRow` gains `abbreviation`. Tests: abbreviation query, mixed
   query, an exact abbreviation ranks first against a fuzzy decoy, no Create offer for an exact
   abbreviation.
3. **tt-store (R24).** `migrations/0002_abbreviations.sql` (columns + partial unique indexes),
   registered in `migrate.rs`; `load.rs` and `apply.rs` follow. Tests: a 0001 database migrates
   and loads, the backstop index rejects a duplicate abbreviation, reload equality with
   abbreviations.
4. **tt-cli (R24).** `--abbr` on create and update for clients, projects and tasks; `project
   rename`/`task rename` become `update`. A scenario test.
5. **src-tauri (R24).** DTOs with abbreviations and `shortLabel`; `update_project`/`update_task`
   replace the rename commands; the create and update commands take the abbreviation; `bridge.rs`
   gets the new event names; `tray.rs` uses the short label. Regenerate `ui/src/bindings.ts`.
6. **UI (R24, R3, R6).** `ClientDialog.vue` and `NameDialog.vue` get an optional Abbreviation
   field; `ClientsView.vue` shows abbreviations. `QuickSwitch.vue`, `ComboPicker.vue`,
   `RecentsRow.vue`, `RunningBar.vue` and `StopAtDialog.vue` show `shortLabel`, with the full
   label as the hover title. `lib/combo.ts` `projectOnly()` builds both labels. `TaskInput.vue`
   shows the abbreviation muted next to the name. Component tests next to each component.
7. **Release.** Release notes for 0.2.0 with the migration and downgrade note; stage 5 review;
   merge after the owner's "approved to merge"; install over the live database.

## Deviations
- Step 4: the tt-cli subcommands are `project edit` and `task edit` (with `--name`, `--abbr`),
  not `update`, to match the existing `client edit`. Omitted options keep their value, and
  `--abbr ""` removes the abbreviation.
- Step 5: a clash message names the item with its abbreviation, e.g. `The abbreviation is
  already used by project "Web" (W)`, so a clash with an abbreviation is understandable.
- Step 6: the recents' hover text and the quick-switch task step's hint (the task's
  abbreviation, muted) were added for the same reason. The Clients view's Rename buttons are now
  Edit, because the dialog edits name and abbreviation.
- The fake migration in `crates/tt-app/tests/startup.rs` moved from version 2 to 3, because 0002
  is now real.

## Coverage
R24: steps 1–6. R1 amendment: step 1 (+ step 3 backstop). R3/R6 amendments: step 2 (+ step 6
display).

## Proof
- `gatekeeper` runs the full `pwsh scripts/check.ps1` after each step (`ALL CHECKS PASSED`,
  including `bindings_up_to_date`).
- Migration dry run: copy the live database to a scratch folder, run `tt-cli --db <copy>` on it,
  set an abbreviation and match it.
- Dev app: the long Northwind example with abbreviations; screenshots of the quick-switch (`ascp`,
  `nwi cal`), the recents row, the running bar and the tray tooltip. The daily overview still
  shows full names.
