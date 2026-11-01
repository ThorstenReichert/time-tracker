# Intent: ThorSoft branding (logo and colour theme)
Author: Thorsten Reichert. Status: approved (owner, 2026-11-01).
Revised 2026-11-01 (owner): a branded dark mode after all, following Windows, with a lighter logo.
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
- The light theme stays white-dominant, as it is now. In Windows dark mode the app uses a
  matching dark theme (navy from the logo) and a lighter variant of the logo.

## Constraints
- Both windows follow the Windows light or dark setting, as before.
- Text and accent colours stay readable on their background in both modes (WCAG AA, at least
  4.5:1); the logo stands out in both.
- The architecture rules of the main spec still apply; this change touches only the UI.

## Out of scope
- The app, window, installer and tray icons. They stay as they are (owner, 2026-11-01).
- Any change to behaviour.

## Open questions
None.
