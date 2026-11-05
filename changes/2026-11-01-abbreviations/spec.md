# Spec: Abbreviations for clients, projects and tasks
Source: [intent.md](intent.md). Amends: [../../spec.md](../../spec.md). Author: Thorsten Reichert.
Status: approved (owner, 2026-11-01).
Revised 2026-11-05 (owner): R24, an abbreviation can have up to 50 characters (was 20, too
short in practice).

This spec adds R24 and amends R1, R3 and R6 and spec §2. Everything else in the main spec
stays as it is.

## 1. Requirements

- **R24 Abbreviations.** Clients, projects and tasks have an optional abbreviation. It follows
  the name rules (trimmed, no control characters) with at most 50 characters. Blank input means
  "no abbreviation". It can be set when an item is created and changed or removed when the item
  is edited (in the UI and in tt-cli). A task created on the fly from a task input (R3) has none.
  - The **short label** of a combo is, per level, the abbreviation or else the name, joined like
    the full label: `NWI / ASCP / Review`.
  - The quick-switch popup, the combo pickers, the recents row, the running bar and the
    stop-at dialog show the short label. Hovering shows the full label. The tray tooltip and
    menu show the short label.
  - The daily overview, entry lists, the Clients view and exports keep the full names. The
    Clients view also shows the abbreviations.

Amendments:
- **R1 (names).** Names and abbreviations share one namespace per parent: within a parent, no
  name or abbreviation may equal a sibling's name or abbreviation, ignoring case. An item's own
  name and abbreviation may be equal. Archived items count (OQ-7).
- **R3 and R6 (matching).** Typed text is matched against the names and the abbreviations of
  every level, so mixed queries like `nwi calibration` work. A query equal to an abbreviation ranks
  that item first. In the task input, a text equal to an existing task's name or abbreviation
  doesn't offer `Create "<text>"`.

Notes for later slices:
- R2: the unarchive offer for a clashing new item also applies when the clash is an
  abbreviation.
- R23: the CSV export uses the full names. The JSON export includes the abbreviations.

## 2. Design

- **tt-core.** An `Abbreviation` newtype (like `Name`, max 50 characters, lowercase key).
  `Client`, `Project` and `Task` get `abbreviation: Option<Abbreviation>`. The `*_named`
  lookups match a name or an abbreviation, so the uniqueness checks and the R3 Create offer
  cover both. `check_invariants` checks the shared namespace.
- **Commands and events.** `CreateClient`, `UpdateClient`, `CreateProject` and `CreateTask` take
  `abbreviation: Option<String>`. `RenameProject` and `RenameTask` become `UpdateProject` and
  `UpdateTask` with name and abbreviation. Events: `ClientUpdated` carries the abbreviation, and
  `ProjectRenamed`/`TaskRenamed` become `ProjectUpdated`/`TaskUpdated` (still self-contained).
- **Read models.** `ComboRow` carries the abbreviation of each level and `short_label` next to
  `label`. Quick-switch and task matching score a match text that holds the names and
  abbreviations. Ties are still broken by the full label.
- **Data model.** Migration `0002_abbreviations`: a nullable `abbreviation TEXT` column on
  `client`, `project` and `task`. Backstop: partial unique indexes on
  `(parent, abbreviation COLLATE NOCASE) WHERE abbreviation IS NOT NULL`. Whether a name equals a
  sibling's abbreviation is checked by the core only (no database backstop, as for R12 overlaps).
- **Downgrade.** 0.1.0 refuses a database with migration 0002 (`UnknownMigration`). To go back,
  restore the backup taken before the migration (R23). The release notes say this.

## 3. Constraint traceability
| Constraint (intent.md) | Satisfied by |
|---|---|
| A short label is never ambiguous | R1 amendment (shared namespace), core check + abbreviation index |
| Architecture rules still apply | §2: the rules live in tt-core, tt-cli gets the same commands |

## 4. Open questions
None.

## 5. Areas of concern
- **AC-7 Fuzzy ranking with short queries.** A 3–4 letter query fuzzily matches many long names.
  *Mitigation:* a test pins that an exact abbreviation ranks first. More tuning goes under OQ-2.
