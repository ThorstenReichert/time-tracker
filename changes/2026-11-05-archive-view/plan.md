# Plan: Archive view
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-05). Branch: `feat/archive-view`.
Scope: R2e, R2f and the R2a amendment. Deferred: archiving clients and tasks, totals, search, deleting unused items.

## Context
0.4.0 hid archived projects from Clients & projects and left a view of them for later. The
UI can unarchive a project only through a name match in the New project dialog (R2c). The
groundwork exists: `archived_at`, `UnarchiveProject`, `read::local_date`, and the
`read::recents(catalog, entries, n)` precedent of a read model that takes `&Entries` and uses
only named queries. The owner decided to approve intent, spec and plan in one go.

## Decisions made while planning
| Topic | Decision | Rejected alternatives (why) |
|---|---|---|
| Placement | Own Archive view | A section in Clients & projects (keeps that view about active work) |
| No entries | At the top, "No entries" | At the bottom (an unused project is the likeliest to be acted on) |
| Action | Unarchive with one click (owner, after trying the first version) | A confirmation (too much friction); a read-only list |
| Archive action | One click too (R2a amendment), since unarchiving is one click | Keeping the 0.4.0 confirmation (no longer warranted) |
| Day of an entry | Its start date (R18 step 1) | The end date (would differ from every other view) |
| Period source | Named query `start_span` over a new in-memory index | Scanning all entries (breaks OQ-9) |

## Risks
- **Riskiest step: the new index (step 1).** A missed update in `insert`, `remove` or
  `replace` gives a wrong period. De-risked by `indexes_consistent` (checked after every
  `evolve` in tests and on load) and the model test.
- **Load cost (AC-10):** checked by the load benchmark.

## Steps
1. **tt-core named query (R2e).** `entries.rs`:
   - an index `by_project: BTreeSet<(ProjectId, Instant, EntryId)>`, kept by `insert` and
     `remove` (so `replace` too) and checked by `indexes_consistent`;
   - `start_span(project) -> Option<(Instant, Instant)>`.
   - Tests: the span after insert, edit, a move between projects and delete; other projects
     don't count.
2. **tt-core read model (R2e).** `read/archive.rs`: `archived_projects`, `ArchivedClient`,
   `ArchivedProject`, `ActivePeriod`. Tests in `tests/read_models.rs`: the full order, active
   projects and clients left out, local dates around midnight in a non-UTC zone.
3. **tt-app (R2e).** `archived_projects(ctx)`. Test: archive, then the project is listed with
   its period.
4. **tt-cli (R2e).** `archive`. `tests/scenarios.rs`: two archived projects in order; after an
   unarchive, one.
5. **src-tauri (R2e).** `ArchiveClientDto`, `ArchivedProjectDto`, command `get_archive`,
   registered in `bindings.rs`; regenerate `ui/src/bindings.ts`.
6. **UI (R2e, R2f).**
   - `views/ArchiveView.vue`: loads `getArchive`, reloads on `data-changed`; client sections,
     project rows with the period or "No entries"; Unarchive with one click, a failure as a
     toast; the empty state.
   - `lib/format.ts`: `formatPeriod(first, last)`.
   - `router.ts` route `/archive`; `App.vue` nav entry; the `ClientsView.vue` comment.
   - Tests: `ArchiveView.test.ts`, `format.test.ts`.
7. **Release (stage 5).** Version 0.5.0 in all version files; release notes (no migration);
   fresh-context review against `REVIEW.md`, this spec and this plan; the owner's "approved to
   merge", then the merge.

## Deviations
- Step 1: the index is a `BTreeMap<ProjectId, BTreeSet<(Instant, EntryId)>>` instead of one
  flat set, so `start_span` is the first and last element of the project's set. A project's set
  goes with its last entry. The load benchmark: load and check 62 ms, 38 MB (AC-10).
- Steps 2-5 were gated together (the Rust part of `check.ps1`), not one by one.
- Step 3: the tt-app test extends `an_archived_project_survives_a_reload_and_is_not_bookable`
  in `tests/write_path.rs`, so the view is also checked after a reload.
- Step 5: `get_archive` reads the archive, then the catalog for the names, under two locks,
  like `check_project_name`. An item missing from the catalog is left out.
- Step 6: Unarchive runs at once. The first version asked for a confirmation, which the owner
  found too much friction; intent, spec (R2f) and the decisions above were changed with it.
- Step 6: Archive in Clients & projects runs at once too (owner). The confirmation of 0.4.0
  guarded an action that had no easy undo; the Archive view is that undo. Spec: the R2a
  amendment.
- Step 6: no `formatPeriod`. The view lays out all clients as one CSS grid (subgrids per
  client and project), so the first days start at one edge and the dashes share a centre
  (owner, after a look at the first version). Each day is formatted by `formatShortDay`
  ("2 Mar 2026") in `lib/day.ts`, tested in `day.test.ts`. A single day is a full range too
  ("5 Nov 2026 – 5 Nov 2026", owner), so every period reads the same.
- Stage 5 review fixes (2026-11-05):
  - The order test couldn't tell "first day newest first" from "last day newest first". A
    project with a newer first day and an older last day (Late short) now pins that the first
    day decides.
  - `ArchivedProjectDto` carries one `period: ActivePeriodDto | null` instead of two optional
    days, so a period with one end can't be represented.
  - The view styles its load error like the other views. New tests cover a failed load and
    the re-render after `data-changed`.
  - Small fixes: a doc comment that had slipped onto the wrong DTO, `sort_by_cached_key`, and a
    long doc line.

## Coverage
| Requirement | Steps |
|---|---|
| R2e | 1, 2, 3, 4, 5, 6 |
| R2f | 6 |
| R2a amendment | 6 |

## Proof
- `gatekeeper` runs the full `pwsh scripts/check.ps1` after each step and must report
  `ALL CHECKS PASSED`, including `bindings_up_to_date`.
- `gatekeeper` runs the load benchmark after step 1 (AC-10).
- tt-cli scenario on a scratch `--db`.
- In the dev app, with made-up names: archive three projects of one client (one without
  entries, two with different periods), check the order and the dates, unarchive one.
