# Intent: Archive projects
Author: Thorsten Reichert. Status: approved (owner, 2026-11-05).
Builds on: [../../intent.md](../../intent.md), [../../spec.md](../../spec.md) (0.3.0).

## Problem
Finished projects stay in every autocomplete (quick-switch, combo pickers, recents, task input),
and I can still book time onto them by mistake. I want to archive them, starting with projects.
Later I may archive entire clients too.

## Proposed outcome
- A project can be archived.
- An archived project can no longer be booked onto.
- An archived project isn't listed in autocomplete searches.
- Existing entries on an archived project stay where they are and keep showing in the entry
  lists, the daily overview and the statistics.
- Creating a project whose name or abbreviation matches an archived project doesn't create a
  duplicate. The New project dialog shows inline that the project is archived, with its name and
  abbreviation, and offers to unarchive it instead. There is no second dialog on top. An honest
  mix-up is caught, and nothing is unarchived automatically. If I don't mean that project, I
  choose a different name.

## Constraints
- Project names and abbreviations stay unique across archived and active projects of a client.
  This keeps OQ-7 of the main spec.
- The design must extend to archiving clients later without rework.
- The architecture rules of the main spec still apply: the rules live in the pure core, there is
  one write path, and every feature is reachable without the UI.

## Out of scope
- A view that lists archived projects (a later change).
- Archiving clients or tasks.
- Deleting unused clients, projects or tasks.
- Favorites (not built yet). When they come, they follow the same "active only" rule.

## Open questions
None.

Recorded: my first idea was that archived projects "free" their names, so that a new project
could reuse them. I dropped it. Several archived projects with the same name would make "which
one to unarchive" unanswerable.
