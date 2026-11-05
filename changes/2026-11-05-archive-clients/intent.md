# Intent: Archive clients
Author: Thorsten Reichert. Status: approved (owner, 2026-11-05).
Builds on: [../2026-11-05-archive-projects/intent.md](../2026-11-05-archive-projects/intent.md),
[../2026-11-05-archive-view/intent.md](../2026-11-05-archive-view/intent.md),
[../../spec.md](../../spec.md) (0.6.0).

## Problem
Projects can be archived (0.4.0) and brought back from the Archive view (0.5.0). Clients can't.
A client I no longer work for stays in Clients & projects, and its projects stay bookable unless
I archive each one.

## Proposed outcome
- A client can be archived with one click in Clients & projects. It then disappears from that
  view.
- Its projects can no longer be booked onto and aren't offered anywhere, as if each one were
  archived. Their entries stay where they are.
- Archiving a client is reversible through an Unarchive action, as for projects. Unarchiving
  restores exactly what was there before: projects I had archived on their own stay archived,
  the others are active again.
- The Archive view shows an archived client with all its projects and one "Unarchive client"
  button. Projects that were archived on their own are marked as such.
- Creating a client whose name or abbreviation matches an archived client offers to unarchive
  it inline, as the New project dialog does.

## Constraints
- Client names and abbreviations stay unique across archived and active clients (OQ-7).
- Archiving a client doesn't change its projects' own archive state. A mistaken archive is
  undone with one click.
- The architecture rules of the main spec still apply: the rules live in the pure core, there is
  one write path, and every feature is reachable without the UI.

## Out of scope
- Archiving tasks.
- Deleting unused clients, projects or tasks.
- Unarchiving a single project of an archived client (the client has to be unarchived first).

## Open questions
None.

Recorded: I considered cascading, so that archiving a client also archives its active projects
after a question. I dropped it. A mistaken "yes" could only be undone project by project, and
it adds a dialog to a reversible action.
