# Intent: Personal project time tracker
Author: Thorsten Reichert (software engineer, consulting). Status: draft.

## Problem
I work as a software engineer at a software consulting firm and must bill my working
hours to several clients, and within each client to different projects (and tasks).
During a day I switch between clients/projects often, and every day I have to book
my hours per project into the company/client booking systems.

Today this is awkward:
- Starting, stopping and especially *switching* the tracked project interrupts my flow
  and is easy to forget or delay, which makes the recorded times inaccurate.
- Clients have different billing granularities (e.g. one requires hours rounded to
  30 min, another to 15 min) and different rounding rules. I do this by hand at the end
  of the day, which is tedious and error-prone.
- I have no easy overview of how much time went into a project or client over a
  week/month, or how much rounding adds or removes compared to the raw time.

General-purpose tools like Clockify cover the basic tracking, but not this workflow
(per-client rounding rules, minimal-friction switching) on my own machine.

## Proposed outcome
A personal time-tracking tool, with core functionality close to clockify.me, that I
use instead of Clockify.

- **Structure:** clients → projects → tasks. Tasks are optional but encouraged, and must
  be very quick to create: a free-form input matches existing tasks as I type and offers
  "create new" if nothing matches.
- **Ergonomic tracking:** start, stop and switch the running timer in seconds without
  breaking flow. This includes a global hotkey / quick-switch from anywhere and one-click
  favorites/recent client-project-task combos. Exactly one timer runs at a time;
  switching stops the current entry and starts the next.
- **Manual corrections:** add entries after the fact, edit start/end, fix a forgotten
  stop, delete entries.
- **Daily billing overview:** for any day (today or a past day I pick), a clear view of
  the hours to book per project (or per client, see below). The numbers are *rounded*
  by the client's rules and ready to copy into the booking systems by hand. The raw
  time is shown next to them for reference. This is the only place rounding is applied.
- **Per-client rounding rules:** each client has a rounding unit (e.g. 15 or 30 min),
  a direction (up / nearest / down) and a level:
  - *Project level:* add up all of a project's entries for the day, then round that
    project total.
  - *Client level:* add up all of the client's projects for the day, then round that
    client total once. These clients get only the rounded client total; per-project
    figures are informational and raw.
- **Statistics (accurate, not rounded):** totals per project and per client over a
  selectable range (week/month/custom), based on the time actually tracked, with
  charts.
- **Billing error statistic:** for each day, the gap between the hours booked (the
  rounded values from the daily billing overview) and the hours actually tracked. It is
  shown per client/project and per day, and adds up over a selectable range (e.g. "this
  month I billed 1h15 more than I worked for client A"). Charts are welcome here too.

Success looks like:
- Starting or switching to another project/task takes a couple of seconds or keystrokes.
- I stop using Clockify for my work time tracking.

## Affected users and systems
- **Users:** me only (single user, personal use at work).
- **Systems:** my work machine (Windows). The tool runs locally on it, either as a
  Windows desktop app or a local web app; the choice is deferred to design.
- **Downstream (not integrated):** company/client time-booking systems. I keep entering
  the daily summary there by hand; the tool only has to make the numbers easy to read
  and copy.
- **Replaced:** Clockify, for this purpose. No data migration; clients, projects and
  tasks are set up fresh.

## Constraints
- Single user, runs locally on one machine. No team/sharing features.
- Technology (desktop vs. local web, language, storage) is deliberately left open for
  requirements/design.
- Work-related data (client and project names) must stay under my control. No
  third-party cloud service is required to use the tool.
- Rounding must be deterministic and transparent. Raw times are always kept, and
  rounding is applied only in the daily billing overview (and in the billing error it
  produces), never to stored entries or general statistics.

**Out of scope**
- Invoicing, hourly rates or any monetary values (hours only).
- Multi-user, accounts, permissions, team features.
- Automatic activity tracking (watching active windows/apps).
- Work-hours/overtime/vacation accounting against a contract.
- Integration/API push into booking systems (manual copy is enough).
- Import from Clockify or other tools.

## Open questions
- **Data storage & backup:** local file/DB only, or local-first with backup/sync
  (e.g. via OneDrive or git)? To be decided in design.
- **"Booked" hours = computed or confirmed?** Booking happens by hand outside the tool.
  The assumption is that the billing error uses the computed rounded values. Should I
  also be able to mark a day as "booked" or record a differing actual booking?
- **Changing past data:** if I edit past entries or change a client's rounding rule,
  should past daily overviews and billing errors be recalculated, or stay frozen as
  they were booked? (Linked to the previous question.)
- **Rounding overrides:** do rounding rules ever differ per project within one client,
  or are they strictly per client?
- **Day boundary:** how should entries that span midnight count (split at midnight or
  belong to the start day)?
- **Breaks:** are breaks simply "no timer running", or is an explicit pause/break
  concept needed?
- **Archiving:** should finished clients/projects/tasks be archivable so they drop out
  of quick-switch suggestions but stay in historic stats?
- **Billable flag / tags / descriptions:** not requested for now. Confirm they aren't
  needed (e.g. a free-text note per entry).
- **Forgot-timer safeguards** (e.g. warn about a timer running overnight): not
  prioritised now. Revisit after first use.
