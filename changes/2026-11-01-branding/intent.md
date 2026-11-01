# Intent: ThorSoft branding (logo and colour theme)
Author: Thorsten Reichert. Status: approved (owner, 2026-11-01).
Builds on: [../../intent.md](../../intent.md), [../../spec.md](../../spec.md) (0.2.0).

## Problem
"ThorSoft" is my made-up software company. It has an established logo
(the original artwork is `ui/src/assets/thorsoft-logo.svg`), but the app
doesn't show it anywhere, and its colours are PrimeVue's stock emerald green, which has nothing
to do with the brand.

## Proposed outcome
- The logo appears where suitable: at the left of the main window's navigation bar, in an
  "About" section in Settings (with the product name and version), and in the quick-switch
  popup.
- The UI uses a colour theme that matches the logo (navy and slate blue) for its accents.
- The theme is light. White stays the predominant colour, as it is now.

## Constraints
- Light only: the app no longer follows the Windows dark mode, in the main window and the popup.
- The accent colour stays readable on white (WCAG AA, at least 4.5:1).
- The architecture rules of the main spec still apply; this change touches only the UI.

## Out of scope
- The app, window, installer and tray icons. They stay as they are (owner, 2026-11-01).
- A dark theme.
- Any change to behaviour.

## Open questions
None.
