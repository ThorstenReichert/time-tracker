# Spec: Archive clients
Source: [intent.md](intent.md). Amends: [../../spec.md](../../spec.md). Author: Thorsten Reichert.
Status: approved (owner, 2026-11-05). Skills applied: none.

This spec implements R2 of the main spec for clients (R2g–R2k) and amends R2e and R2f. It builds
on R2a–R2d of [../2026-11-05-archive-projects/spec.md](../2026-11-05-archive-projects/spec.md)
and R2e–R2f of [../2026-11-05-archive-view/spec.md](../2026-11-05-archive-view/spec.md).
Archiving tasks and deleting unused items stay deferred. OQ-7 is unchanged and now confirmed for
clients.

## 1. Requirements

- **R2g Archive a client.** A client can be archived and unarchived, in the UI and in tt-cli.
  - Archiving sets the client's `archived_at` to now. Unarchiving clears it. Archiving an
    archived client, or unarchiving an active one, changes nothing.
  - Neither touches the projects' own `archived_at`. The projects of an archived client are
    inactive through their client (R2a "inactive"). Unarchiving the client restores the previous
    state: projects archived on their own stay archived.
  - Archiving a client is rejected while the running entry is on one of its projects. The timer
    has to be stopped first.
  - The Clients & projects view hides archived clients. Each active client gets an **Archive**
    action, which runs at once, without a confirmation (as R2a amended). A rejection is shown as
    a toast. If every client is archived, the view says "No active clients."
  - Entries under an archived client keep their combo. They still show in the entry lists, the
    daily overview, the statistics and the billing error.
- **R2h Booking under an archived client.** R2b applies to every project of an archived client,
  since they are inactive. In addition:
  - `CreateProject` for an archived client is rejected.
  - Editing an archived client (name, abbreviation, rule) stays allowed, as renaming an archived
    project does. The UI doesn't reach it.
- **R2i New client name check.** While the user types in the New client dialog, a pure read
  model checks the typed name and abbreviation against all clients, by the rule of R2c:
  - no client hit → **free**: the dialog creates the client;
  - one archived client *A* hit name to name or abbreviation to abbreviation → **unarchive**:
    the dialog shows inline that *A* is archived, with its name and abbreviation, and its button
    reads **Unarchive client**. Clicking it unarchives *A* as it was. The typed abbreviation and
    rounding rule are ignored, so the rounding controls are disabled while the hint shows (owner,
    2026-11-05). Name and abbreviation stay editable;
  - anything else → **taken**: the dialog names the clashing clients and disables its button.

  `CreateClient` still enforces uniqueness on save.
- **R2j Archive view with clients.** Amends R2e and R2f.
  - A client is listed if it is archived itself or has an archived project. The client order is
    unchanged: name (case-insensitive), then ID.
  - An archived client is marked "archived" and has one **Unarchive client** button, which runs
    `UnarchiveClient` at once (no confirmation). It lists **all** its projects, ordered as in
    R2e, each with its active period. Its projects have no Unarchive button. Projects archived on
    their own are marked "archived separately". An archived client without projects shows
    "No projects".
  - An active client lists only its archived projects, each with its Unarchive button (R2f,
    unchanged).
  - With nothing archived, the view says "Nothing archived."
- **R2k Messages.**
  - Booking under an archived client: `Client "Acme" is archived` (the client comes first, R2a).
  - Archiving a client while the timer runs on one of its projects: `The timer is running on a
    project of this client; stop it first`.
  - A duplicate client name or abbreviation names an archived client as
    `archived client "Acme" (AC)` (as R2d).

## 2. Design

- **Commands:** `ArchiveClient { id }`, `UnarchiveClient { id }`.
- **Events:** `ClientArchived { id, at }`, `ClientUnarchived { id }`. They are self-contained:
  tt-store updates `client.archived_at` without reading State.
- **Error:** `ClientRunning { client }`. The booking checks need no change: `Catalog::archived_in`
  and `is_*_active` already check the client first, and so does the invariant "the running
  entry's combo is active".
- **Read model** `NameCheck<Id>` becomes generic: `Free`, `Unarchive { item: Id }`,
  `Taken { by: Vec<Id> }`. `project_name_check` keeps its rule. `client_name_check(catalog,
  name, abbreviation)` is new. Both share one private helper over the candidate siblings.
- **Read model** `read::archived_projects`, renamed `read::archive_view` because it lists
  clients too (owner, 2026-11-05). `ArchivedClient` and `ArchivedProject` each get
  `archived: bool` (the item's own flag). It still reaches entries only through
  `Entries::start_span` (OQ-9).
- **src-tauri:** `archive_client`, `unarchive_client` and `check_client_name`. `NameCheckDto`
  names its items `items` instead of `projects`. The archive DTOs carry `archived`.
- **UI:** the live-check logic of `NameDialog` moves into a composable that `ClientDialog`
  shares.
- **Data model:** no migration. `client.archived_at` exists since 0001. A 0.6.0 build opens a
  0.7.0 database and already refuses booking under an archived client, but still lists the
  client in Clients & projects.

## 3. Constraint traceability
| Constraint (intent.md) | Satisfied by |
|---|---|
| Client names unique across archived and active (OQ-7) | R2i, R2k, unchanged `CreateClient` check |
| Projects' own archive state untouched; one-click undo | R2g, R2j (Unarchive client) |
| Architecture rules still apply | R2g/R2h in `decide`; R2i/R2j tt-core read models; tt-cli commands |
| Removed from Clients & projects | R2g |
| Reversible through Unarchive | R2g, R2i, R2j |

## 4. Open questions
None.

## 5. Areas of concern
- **AC-11 Shared name check.** Moving the live check of `NameDialog` into a composable for
  `ClientDialog` risks regressing R2c (stale answers, the button state). *Mitigation:* the
  existing `NameDialog.test.ts` cases pass unchanged in behaviour, and `ClientDialog.test.ts`
  gets the same cases. Owner: Thorsten Reichert.
