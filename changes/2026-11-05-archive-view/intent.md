# Intent: Archive view
Author: Thorsten Reichert. Status: approved (owner, 2026-11-05).
Builds on: [../2026-11-05-archive-projects/intent.md](../2026-11-05-archive-projects/intent.md),
[../../spec.md](../../spec.md) (0.4.0).

## Problem
Archived projects disappear from the Clients & projects view, and nothing lists them. I can't
see which projects I archived, when I worked on them, or bring one back without typing its name
into the New project dialog.

## Proposed outcome
- An "Archive" view of its own lists all archived projects, grouped by client.
- Each archived project shows its *active period*: the day of its first time booking to the day
  of its last time booking.
- Within each client, archived projects are ordered first by the start of the active period
  (more recent at the top), then by its end (more recent at the top), then by name.
- An archived project with no time bookings has no active period. It shows "No entries" and
  stands at the top of its client.
- Each archived project can be unarchived from the view, after a confirmation.

## Constraints
- The architecture rules of the main spec still apply: the rules (which projects, the period,
  the order) live in the pure core, and the view is reachable without the UI.
- Reads reach entries only through named queries SQL could answer with an index (OQ-9).

## Out of scope
- Archiving clients or tasks.
- Totals of hours per archived project.
- Search or filter in the view.
- Deleting unused projects.

## Open questions
None.
