# Plan: Archive projects
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-05). Branch: `feat/archive-projects`. Version: 0.4.0.
Scope: R2a–R2d and the R7 amendment. Deferred: archiving clients and tasks, deleting unused
items, the archive view, favorites.

## Context
Most groundwork exists from slice 1:
- the `archived_at` columns (0001) and fields on `Client`, `Project` and `Task`;
- `Catalog::is_project_active`, which already checks the client;
- read models that already skip inactive combos (`read/combo.rs` `is_active`, the quick-switch
  candidates, the task matches).

What is missing: commands and events that set `archived_at`, and write-side checks. Today
nothing rejects booking onto an archived project (`decide/tracking.rs` `resolve_combo`). Also
missing are the UI and the name check. Bead 8q9 (the recents read set into tt-core) is folded
into step 2. The owner decided to approve intent, spec and plan in one go.

## Decisions made while planning
| Topic | Decision | Rejected alternatives (why) |
|---|---|---|
| Names | Not freed; OQ-7 stays | Freeing archived names (an ambiguous "which archived Web") |
| Unarchive offer | Inline in the New project dialog; the button becomes "Unarchive project" | A second confirmation dialog over the dialog (awkward); unarchiving silently (hides mix-ups) |
| When the check runs | Live while typing, through a tt-core read model | Only on submit (a wasted click); the rule in the UI (a business rule outside tt-core) |
| Running timer | Archiving the running project is rejected | Stopping the timer as a side effect |
| Edit entries on an archived project | Allowed while the combo stays | Freezing them (fixing an old entry would mean an unarchive) |
| Clients view | Archived projects are hidden | Greyed out (the archive view comes later) |

## Risks
- **Riskiest step: the combo check (step 1).** `Start`, `AddEntry` and `EditEntry` share
  `resolve_combo`. A wrong "current combo" would block fixing old entries. De-risked by
  explicit tests: editing times on an archived project is allowed, moving onto it is rejected,
  and moving away from it is allowed.
- **The new invariant** (the running combo is active) runs on every load (AC-9). It is accepted
  because `decide` can't break it.
- **The live check can race** (responses arriving out of order). The newest request wins, and
  `CreateProject` still validates on save.
- **Picker with an archived combo (AC-8):** covered by a test for the entry dialog.

## Steps
1. **tt-core domain (R2a, R2b, R2d).**
   - `command.rs`: `ArchiveProject { id }`, `UnarchiveProject { id }`.
   - `event.rs`: `ProjectArchived { id, at }`, `ProjectUnarchived { id }`.
   - `decide.rs`: dispatch the new commands.
   - `decide/catalog.rs`:
     - `archive_project`: rejects with `ProjectRunning` if the running entry is on the project.
       Already archived → `[]`.
     - `unarchive_project`: active → `[]`.
     - `new_task`: rejects an inactive project with `Archived`.
   - `decide/tracking.rs`: `resolve_combo` takes the entry's current combo
     (`Option<Combo>`). An inactive combo is rejected unless it equals that combo.
   - `error.rs`: `Archived { item: ItemRef }`, `ProjectRunning { project }`. The doc comments
     follow.
   - `evolve.rs`: apply the new events.
   - `invariants.rs`: the running entry's combo is active.
   - Tests in `tests/decide_catalog.rs` and `tests/decide_tracking.rs` for every rejection and
     every allowance in R2b, the no-ops, and archiving while running.
   - `tests/model.rs`: the command generator gets archive and unarchive.
2. **tt-core read models (R2c, R7).**
   - `read/name_check.rs`: `project_name_check` and `NameCheck`. Tests:
     - a free name;
     - an archived exact name, and an archived exact abbreviation → Unarchive;
     - a cross match with an archived project → Taken;
     - an active hit → Taken;
     - two siblings hit → Taken;
     - case and whitespace;
     - another client's project doesn't count.
   - `entries.rs`: `recent_combos_where(n, keep)`. `read/combo.rs`:
     `recents(catalog, entries, n)` passes `is_active`. Test: N active combos older than N
     archived ones still give N recents.
3. **tt-store (R2a).**
   - `apply.rs`: `UPDATE project SET archived_at = ?` for both events.
   - Tests: archive and unarchive round-trip in `tests/roundtrip.rs`.
4. **tt-app (R2c, R7).**
   - `recents()` calls `read::recents(catalog, entries, n)`, without the `n * 4` guess.
   - New method: `project_name_check(client, name, abbreviation)`.
   - Tests:
     - `tests/write_path.rs`: archive, then start is rejected and nothing is written.
     - `tests/reload_equality.rs`: a sequence with archive and unarchive.
5. **tt-cli (R2a).**
   - `project archive --id`, `project unarchive --id`.
   - `tests/scenarios.rs`: archive → start rejected → recents and quick-switch skip the project
     → unarchive → start works.
6. **src-tauri (R2a, R2c, R2d).**
   - `commands.rs`: `archive_project`, `unarchive_project`, `check_project_name` (returning a
     `NameCheckDto`), registered in `bindings.rs`.
   - `bridge.rs`: the new events count as data changes.
   - `errors.rs`: `item_name` says "archived" for an archived item; messages for `Archived` and
     `ProjectRunning`.
   - Regenerate `ui/src/bindings.ts`.
7. **UI (R2a, R2c, AC-8).**
   - `NameDialog.vue`: optional `check` and `unarchive` props. The `Unarchive` result shows an
     info message with the archived project's name and abbreviation, and the button "Unarchive
     project". The `Taken` result shows an error message naming the clashes and disables the
     button.
   - `ClientsView.vue`:
     - New project passes `check` and `unarchive`.
     - Each project gets an Archive button with a confirmation; a failure goes to a toast.
     - Archived projects are hidden.
   - Tests: `NameDialog.test.ts`, `ClientsView.test.ts`, and `EntryDialog.test.ts` (AC-8).
8. **Release (stage 5).**
   - Version 0.4.0 in all version files.
   - Release notes: no migration, and 0.3.0 can read the database.
   - Fresh-context review against `REVIEW.md`, this spec and this plan.
   - The owner's "approved to merge", then the merge.

## Deviations
- Step 2: `read::recents` takes the `Entries` and uses the new named query
  `read::recent_combos(catalog, entries, n)` (`Entries::recent_combos_where`).
  `read::quick_switch` keeps its signature. tt-app passes it
  `read::recent_combos(.., read::QUICK_SWITCH_RECENCY)`. The constant moved from tt-app into
  tt-core, so the rule and the fetch size sit together (bead 8q9).
- Step 5: the subcommands take the ID as a positional argument (`project archive 1`), like
  the existing `project edit 1`, instead of `--id`.
- Step 6: `bridge.rs` needs no change. Every batch already emits `data-changed` and nudges the
  tray, and archiving never touches the running entry, so there is no `timer-changed`.
- Step 6: the core's `ProjectRunning` text names the ID (`the timer is running on project 3; stop
  it first`), so tt-cli's message reads well too.
- Step 7: the New project dialog's button reads "Save" while the name is free, as in the other
  name dialogs, and "Unarchive project" for an archived match.
- Step 1: the model generator unarchives twice as often as it archives. With equal weights, the
  `Archived` rejections pre-empted the typed-time checks, and the generator stopped reaching
  `NonexistentLocalTime`.

## Coverage
| Requirement | Steps |
|---|---|
| R2a | 1, 3, 5, 6, 7 |
| R2b | 1 |
| R2c | 2, 4, 6, 7 |
| R2d | 1, 6 |
| R7 amendment | 2, 4 |

## Proof
- `gatekeeper` runs the full `pwsh scripts/check.ps1` after each step and must report
  `ALL CHECKS PASSED`, including `bindings_up_to_date`.
- tt-cli scenario on a scratch `--db`.
- In the dev app, with made-up names:
  - archive a project, then check that the quick-switch, the recents and the Clients view
    don't show it, and that today's entries and the overview still do;
  - type its name in New project and see the hint and "Unarchive project"; unarchive it;
  - try to archive the running project and see the error.
