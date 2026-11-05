# Spec: Archive view
Source: [intent.md](intent.md). Amends: [../../spec.md](../../spec.md). Author: Thorsten Reichert.
Status: approved (owner, 2026-11-05). Skills applied: none.

This spec adds the list of archived projects to R2 of the main spec (R2e, R2f) and amends R2a.
It builds on
R2a–R2d of [../2026-11-05-archive-projects/spec.md](../2026-11-05-archive-projects/spec.md).
Archiving clients and tasks and deleting unused items stay deferred.

## 1. Requirements

- **R2e Archive view.** The main window gets an **Archive** view (nav entry after Clients &
  projects). tt-cli lists the same content.
  - It lists every project whose `archived_at` is set, grouped by client.
  - A client shows up only if it has an archived project. Clients are ordered by name
    (case-insensitive), then by ID.
  - Each project shows its name, its abbreviation and its *active period*.
  - The active period runs from the local date of the start of the project's first entry to
    the local date of the start of its last entry. Entries count on the date they start
    (R18 step 1), so an entry that runs past midnight counts on its first day.
  - A project with no entries has no active period and shows "No entries".
  - Order within a client:
    1. projects with no entries, by name;
    2. then by the first day of the period, newest first;
    3. then by the last day, newest first;
    4. then by name (case-insensitive), then by ID.
  - With no archived projects, the view says "No archived projects."
- **R2f Unarchive from the view.** Each archived project has an **Unarchive** button. It runs
  `UnarchiveProject` (R2a) at once, without a confirmation: archiving it again is as easy.
  The project leaves the view and shows in Clients & projects again. A failure is shown as a
  toast.

Amendment:
- **R2a (archive action).** The Archive action in Clients & projects runs `ArchiveProject` at
  once, without a confirmation. The Archive view undoes it with one click (R2f). A rejection,
  such as archiving the running project, is still shown as a toast.

## 2. Design

- **Named query** `Entries::start_span(project) -> Option<(Instant, Instant)>`: the first and
  last start of the project's entries. `Entries` keeps an in-memory index
  `(project, start, id)`, rebuilt on load and checked by `indexes_consistent`. SQL answers the
  same query as `SELECT MIN(start_utc), MAX(start_utc) FROM time_entry WHERE project_id = ?`
  using an index on `(project_id, start_utc)`. The prefetch migration (OQ-9) would add that
  index; no migration now.
- **Read model** `read::archived_projects(catalog, entries, tz) -> Vec<ArchivedClient>`. It is
  pure, lives in tt-core and uses only `start_span`:
  - `ArchivedClient { client, projects: Vec<ArchivedProject> }`;
  - `ArchivedProject { project, period: Option<ActivePeriod> }`;
  - `ActivePeriod { first: civil::Date, last: civil::Date }`.

  The order of R2e is part of the read model, so no rule sits in the UI.
- **tt-app** `archived_projects(ctx)` takes the read lock and passes `ctx.tz`. **tt-cli**
  `archive` prints it as JSON. **src-tauri** `get_archive` maps it to DTOs with names and
  `YYYY-MM-DD` days.
- **Later client archiving:** the view lists projects that are archived themselves. When
  clients can be archived, the projects of an archived client join it, grouped the same way.
- **Data model:** no migration. 0.4.0 reads a 0.5.0 database.

## 3. Constraint traceability
| Constraint (intent.md) | Satisfied by |
|---|---|
| Rules in the pure core, reachable without the UI | R2e read model in tt-core; tt-app method; tt-cli `archive` |
| Entries only through indexable named queries (OQ-9) | `Entries::start_span` |
| Grouped by client, active period, order | R2e |
| No entries at the top | R2e order, step 1 |
| Unarchive from the view, one click | R2f |
| Archive with one click (owner, 2026-11-05) | R2a amendment |

## 4. Open questions
None.

## 5. Areas of concern
- **AC-10 Load cost of the new index.** A second index over all entries adds load time and
  memory. *Mitigation:* the load benchmark (100k entries, <500 ms, <50 MB) still passes.
  Owner: Thorsten Reichert.
