# Plan: Archive clients
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-05). Branch: `feat/archive-clients`.
Scope: R2g–R2k (amending R2e, R2f). Deferred: archiving tasks, deleting unused items.

## Context
0.4.0 and 0.5.0 laid the groundwork:
- `Client.archived_at` (0001), loaded by tt-store;
- `Catalog::is_project_active` / `is_task_active` / `archived_in`, which check the client first;
- because of those checks, R2b (`resolve_combo`, `new_task`), the running-entry invariant,
  quick-switch, recents, task matches and pickers already treat a project of an archived client
  as inactive;
- the generic `DomainError::Archived { item }`, and `errors::item_label`, which already labels
  `archived client "X" (X)`.

Missing pieces:
- the commands and events;
- the running check and the `CreateProject` check;
- the client name check;
- the client side of the Archive view;
- tt-cli, the DTOs and the UI.

The owner approved intent, spec and plan in one go.

## Decisions made while planning
| Topic | Decision | Rejected alternatives (why) |
|---|---|---|
| Projects of an archived client | Independent flags; projects are inactive through their client | Asking, then archiving all active projects too (a mistaken "yes" is undone project by project; a dialog on a reversible action; a self-undoing cascade needs a migration or timestamp matching) |
| Archive view | Client section marked "archived", one Unarchive client button, all its projects listed, the ones archived on their own tagged "archived separately" | Only a client row without projects (hides what comes back) |
| New client | Live name check with inline "Unarchive client", as R2c | An error only (inconsistent with R2 and OQ-7) |
| Running timer | Archiving the client is rejected | Stopping the timer as a side effect |
| Confirmation | None, as for projects | A confirmation (the owner finds it too much friction) |

## Risks
- **Generic `NameCheck` and the DTO rename (steps 2, 6, 7)** touch the shipped R2c path. The
  existing tt-core, src-tauri and `NameDialog` tests must pass with only the renames changed
  (AC-11).
- **Model-test drift.** Archiving a client makes many combos inactive at once, so `Archived`
  rejections can crowd out other errors (the 0.4.0 lesson). Keep the archive weight low and
  check that every expected outcome name is still seen.
- **Archive view grid.** The new tags and button must keep the subgrid columns aligned. Check
  this in the dev app.

## Steps
1. **tt-core domain (R2g, R2h, R2k).**
   - `command.rs`: `ArchiveClient`, `UnarchiveClient`.
   - `event.rs`: `ClientArchived { id, at }`, `ClientUnarchived { id }`.
   - `decide.rs`: dispatch.
   - `decide/catalog.rs`:
     - `archive_client` rejects with `ClientRunning` while the running entry's project belongs
       to the client; already archived → `[]`;
     - `unarchive_client`;
     - `create_project` rejects an archived client.
   - `error.rs`: `ClientRunning { client }`.
   - `evolve.rs`: apply the new events.
   - Tests in `tests/decide_catalog.rs` and `tests/decide_tracking.rs`.
   - `testing.rs` and `tests/model.rs`: the generator gets client archive and unarchive.
2. **tt-core read models (R2i, R2j).**
   - `read/name_check.rs`: `NameCheck<Id>`, `client_name_check`.
   - `read/archive.rs`: the `archived` flags, and archived clients with all their projects.
   - Tests in `tests/read_models.rs`.
3. **tt-store (R2g).** `apply.rs`: `UPDATE client SET archived_at`. A round-trip test.
4. **tt-app (R2g, R2i).** `client_name_check`. `tests/write_path.rs`: archive a client, start is
   rejected, it survives a reload, and the archive read lists it.
5. **tt-cli (R2g, R2j).** `client archive <id>`, `client unarchive <id>`. A scenario test.
6. **src-tauri (R2g, R2i, R2j, R2k).**
   - `archive_client`, `unarchive_client`, `check_client_name`;
   - `NameCheckDto.items`, and the archive DTO flags;
   - the `ClientRunning` message, and an errors test;
   - regenerate the bindings.
7. **UI (R2g, R2i, R2j, AC-11).**
   - `lib/nameCheck.ts`, a composable shared by `NameDialog` and `ClientDialog`.
   - `ClientDialog`: `check` / `unarchive`.
   - `ClientsView`: hides archived clients, gets an Archive button, and the New client check.
   - `ArchiveView`: the archived client section.
   - Tests for each.
8. **Release (stage 5).**
   - Version 0.7.0.
   - Release notes: no migration; 0.6.0 still lists archived clients.
   - A fresh-context review.
   - The owner's "approved to merge", then the merge.

## Deviations
- Step 1: the model generator weights client archive 1 and client unarchive 6. With lower
  unarchive weights, the deterministic `generator_reaches_the_interesting_cases` run no longer
  reached `NonexistentLocalTime`, which is rare by nature (a DST gap). This repeats the 0.4.0
  lesson.
- Step 1: `decide_tracking.rs`'s `an_archived_client_makes_its_projects_inactive` built its
  archived client from events, since there was no command. It is replaced by a test driven by
  commands: start, add and move are rejected with `Archived { Client }`, and keeping the combo
  or moving away is allowed.
- Step 6: `CheckedProjectDto` is now `CheckedItemDto`. `NameCheckDto::new` takes the `ItemRef`
  constructor (`ItemRef::Project` or `ItemRef::Client`) for the labels.
- Step 7: in the Archive view the client row is a subgrid too, so "Unarchive client" sits in the
  action column with the projects' Unarchive buttons.
- Stage 5 review fixes (2026-11-05). Rust: nothing Important. UI: one Important.
  - AC-11: `ClientDialog.test.ts` was missing two of the R2c cases ("goes back to creating",
    "keeps only the newest answer"). Both are ported now. A new case reopens a New dialog with
    the same check and pins the reset on open and close, which the edit-reuse tests couldn't
    prove.
  - `ItemRef::id()` replaces a re-match in `NameCheckDto::new`. A src-tauri test checks that
    the client check labels its item as a client.
  - The tt-app test asserts that an archived client's entries still show in the daily overview
    (R2g).
  - Small fixes: the Archive view test asserts the tag itself, and `ClientsView` uses the
    `NameChecker` type.
  - Owner decision after the review: New client disables the rounding controls (unit,
    direction, level) while the button reads "Unarchive client", because R2i ignores them. Name
    and abbreviation stay editable. Covered in `ClientDialog.test.ts`. Spec R2i is updated.
  - Owner decision after the review: every row button in Clients & projects and the Archive
    view gets an `aria-label` that names its item, after its visible text (WCAG 2.5.3 "Label
    in Name"). Examples: "Archive client Acme", "New task for Web", "Edit task Review",
    "Unarchive project Legacy". The visible text is unchanged. Tests in `ClientsView.test.ts`
    and `ArchiveView.test.ts` pin the labels.
  - Not changed: `read::archived_projects` keeps its name although it now lists clients too.

## Coverage
| Requirement | Steps |
|---|---|
| R2g | 1, 3, 4, 5, 6, 7 |
| R2h | 1 |
| R2i | 2, 4, 6, 7 |
| R2j | 2, 5, 6, 7 |
| R2k | 1, 6 |

## Proof
- The `gatekeeper` subagent runs the full `pwsh scripts/check.ps1` after the Rust steps and the UI
  step and must report `ALL CHECKS PASSED`, including `bindings_up_to_date`. The load benchmark
  runs after step 2.
- A tt-cli scenario on a scratch `--db`.
- In the dev app, with made-up names:
  - archive a client that has one project archived on its own;
  - check Clients & projects, the quick-switch and the Archive view;
  - try the archive while the timer runs;
  - type the client's name in New client, then unarchive it.
