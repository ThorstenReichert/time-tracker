# Spec: Archive projects
Source: [intent.md](intent.md). Amends: [../../spec.md](../../spec.md). Author: Thorsten Reichert.
Status: approved (owner, 2026-11-05). Skills applied: none.

This spec implements R2 of the main spec for projects (R2a–R2d) and amends R7. Archiving
clients and tasks, deleting unused items and the unarchive list stay deferred. Everything else
in the main spec stays as it is. OQ-7 is unchanged: archived names still count.

## 1. Requirements

- **R2a Archive a project.** A project can be archived and unarchived (in the UI and in tt-cli).
  - Archiving sets the project's `archived_at` to now. Unarchiving clears it. Archiving an
    archived project, or unarchiving an active one, changes nothing.
  - Archiving the project of the running entry is rejected. The timer has to be stopped first.
  - A project is *inactive* if it is archived or, once clients can be archived, if its client is.
    A task of an inactive project is inactive too.
  - Inactive projects and their tasks aren't offered by the quick-switch, the recents, the combo
    pickers, the tray menu or the task input.
  - The Clients & projects view hides archived projects. Each active project gets an Archive
    action with a confirmation.
  - Entries on an archived project keep their combo. They still show in the entry lists, the
    daily overview, the statistics and the billing error.
- **R2b No booking onto an inactive project.** These commands are rejected for an inactive
  project or task:
  - `Start` (including a switch), and `AddEntry`.
  - `CreateTask`, and a task created on the fly from a task input (R3).
  - An `EditEntry` that changes the entry's combo to an inactive one.

  These stay allowed:
  - An `EditEntry` that keeps its combo (start, end, note).
  - Moving an entry from an archived project to an active combo.
  - `DeleteEntry`.
  - Renaming an archived project (not reachable from the UI yet).
- **R2c New project name check.** While the user types in the New project dialog, a pure
  read model checks the typed name and abbreviation against the client's projects. It compares
  the trimmed text case-insensitively, as the uniqueness rules do (R1, R24). Let *H* be the
  sibling projects whose name or abbreviation equals the typed name or the typed abbreviation.
  - *H* is empty → **free**: the dialog creates the project.
  - *H* is one archived project *A*, and *A* was hit name to name or abbreviation to
    abbreviation → **unarchive**. The dialog shows inline that *A* is archived, with *A*'s name and
    abbreviation, and its button reads **Unarchive project**. Clicking it unarchives *A* as it
    was. The typed abbreviation is ignored.
  - Anything else is **taken**, and creating is blocked. The dialog names the clashing projects
    and disables its button. That covers an active project in *H*, two or more projects in *H*,
    or a cross match with an archived project (the typed name is *A*'s abbreviation, or the other
    way round).

  `CreateProject` still enforces uniqueness on save, so the check is only guidance.
- **R2d Messages.**
  - A duplicate name or abbreviation names an archived item as `archived project "Web" (W)`. This
    applies to the Edit project dialog too, which offers no unarchive.
  - Booking onto an inactive project: `Project "Web" is archived`.
  - Archiving the running project: `The timer is running on this project; stop it first`.

Amendment:
- **R7 (recents).** Recents are the last N distinct *active* combos. If N active combos exist,
  there are N recents, however many newer combos are archived.

## 2. Design

- **Commands:** `ArchiveProject { id }`, `UnarchiveProject { id }`.
- **Events:** `ProjectArchived { id, at }`, `ProjectUnarchived { id }`. They are self-contained:
  tt-store updates `project.archived_at` without reading State.
- **Errors (generic for later client archiving):**
  - `Archived { item: ItemRef }`: an inactive project or task was the target of a booking.
    `item` is the first archived item found: the client, then the project, then the task.
  - `ProjectRunning { project }`: archiving the running project.
- **Booking checks** use `Catalog::is_project_active` / `is_task_active`, which already take the
  parent into account. Client archiving later only needs new commands. The shared combo check of
  `Start`, `AddEntry` and `EditEntry` gets the entry's current combo (for `EditEntry`). An inactive
  combo passes only if it is that combo.
- **Invariant:** the running entry's combo is active. `decide` can't break it, because archiving
  the running project is rejected.
- **Read model** `project_name_check(catalog, client, name, abbreviation) -> NameCheck` with
  `Free`, `Unarchive { project }` and `Taken { by: Vec<ProjectId> }`. It is pure and lives in
  tt-core. src-tauri maps it to a DTO for the dialog.
- **Recents (R7):** a named query `Entries::recent_combos_where(n, keep)` returns the newest `n`
  distinct combos that `keep` accepts. SQL could answer it with a join on `archived_at` (OQ-9).
  `read::recents` passes `is_active`. tt-app no longer guesses a fetch size.
- **Data model:** no migration. `archived_at` exists since 0001. The unique name and abbreviation
  indexes already include archived rows, which is OQ-7. A 0.3.0 build can open a 0.4.0 database
  and already hides archived combos from its pickers.

## 3. Constraint traceability
| Constraint (intent.md) | Satisfied by |
|---|---|
| Names and abbreviations unique across archived and active projects | OQ-7 kept (main spec §2 indexes), R2c, R2d |
| Extends to client archiving later | `is_*_active` checks the parent; generic `Archived { item }`; R2a "inactive" |
| Architecture rules still apply | R2b in `decide`; R2c a tt-core read model; archive/unarchive in tt-cli |
| No longer bookable | R2b |
| Not listed in autocomplete | R2a, R7 amendment |
| No automatic unarchive; mix-up caught | R2c (explicit Unarchive button, no second dialog) |

## 4. Open questions
None.

## 5. Areas of concern
- **AC-8 Picker showing an archived combo.** Editing an old entry on an archived project must
  still show its combo, although the picker no longer offers it. *Mitigation:* a component test
  for the entry dialog. Owner: Thorsten Reichert.
- **AC-9 New invariant on load.** The invariant check runs on every load. A hand-edited database
  with a running entry on an archived project would load read-only. *Accepted*, as for the other
  invariants: `decide` can't produce it.
