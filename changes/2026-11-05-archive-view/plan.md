# Plan: Archive view
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-05). Branch: `feat/archive-view`.
Scope: R2e, R2f. Deferred: archiving clients and tasks, totals, search, deleting unused items.

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
| Action | Unarchive with a confirmation | No confirmation; a read-only list |
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
     project rows with the period or "No entries"; Unarchive with a confirmation, a failure as a
     toast; the empty state.
   - `lib/format.ts`: `formatPeriod(first, last)`.
   - `router.ts` route `/archive`; `App.vue` nav entry; the `ClientsView.vue` comment.
   - Tests: `ArchiveView.test.ts`, `format.test.ts`.
7. **Release (stage 5).** Version 0.5.0 in all version files; release notes (no migration);
   fresh-context review against `REVIEW.md`, this spec and this plan; the owner's "approved to
   merge", then the merge.

## Coverage
| Requirement | Steps |
|---|---|
| R2e | 1, 2, 3, 4, 5, 6 |
| R2f | 6 |

## Proof
- `gatekeeper` runs the full `pwsh scripts/check.ps1` after each step and must report
  `ALL CHECKS PASSED`, including `bindings_up_to_date`.
- `gatekeeper` runs the load benchmark after step 1 (AC-10).
- tt-cli scenario on a scratch `--db`.
- In the dev app, with made-up names: archive three projects of one client (one without
  entries, two with different periods), check the order and the dates, unarchive one.
