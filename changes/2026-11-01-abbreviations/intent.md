# Intent: Abbreviations for clients, projects and tasks
Author: Thorsten Reichert. Status: approved (owner, 2026-11-01).
Builds on: [../../intent.md](../../intent.md), [../../spec.md](../../spec.md) (0.1.0).

## Problem
The client, project and task names I deal with in practice can be long, for example the
project "Northwind Instruments: adaptive signal calibration pipeline". The full
"Client / Project / Task" label is unwieldy, especially in the quick-switch popup, the recents
row, the running bar and the tray.

## Proposed outcome
- Clients, projects and tasks can have an optional abbreviation.
- The compact places (quick-switch popup, combo pickers, recents, running bar, tray tooltip and
  menu) show a short label: per level the abbreviation, or the name if there is none (for example
  `NWI / ASCP / Review`). Hovering shows the full label.
- The daily overview, entry lists and the Clients view keep the full names.
- Typed text is matched against the abbreviations as well as the names (quick-switch and task
  input).

## Constraints
- A short label is never ambiguous: within a parent, no abbreviation may equal a sibling's
  abbreviation or name, ignoring case. Archived items count, as for names (OQ-7).
- The architecture rules of the main spec still apply (pure core, single write path, every
  feature reachable without the UI).

## Out of scope
- Automatically generated abbreviations.
- Abbreviations in the daily overview or in exports.

## Open questions
None.
