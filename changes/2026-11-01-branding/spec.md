# Spec: ThorSoft branding (logo and colour theme)
Source: [intent.md](intent.md). Amends: [../../spec.md](../../spec.md). Author: Thorsten Reichert.
Status: approved (owner, 2026-11-01). Revised 2026-11-01 (owner): branded dark mode instead of light only.

This spec adds R25 and amends spec §2 "Frontend". Everything else in the main spec stays as it is.

## 1. Requirements

- **R25 Branding.** The UI carries the ThorSoft brand.
  - **Logo.** The ThorSoft logo (`ui/src/assets/thorsoft-logo.svg`) appears at the left of the
    main window's navigation bar, next to the word "TimeTracker"; in an "About" section at the
    end of Settings, with "ThorSoft TimeTracker" and the app version; and at the left of the
    quick-switch popup's header row, whether a timer runs or not.
  - **Colours.** The accent ("primary") colour of both windows comes from the logo: a scale
    whose steps 500, 800 and 950 are the logo's `#436088`, `#1f3453` and `#0d1d37`. Error
    colours stay red. Backgrounds stay white, borders and muted text a cool grey.
  - **Light and dark.** Both windows follow Windows' light or dark mode. Dark mode uses navy
    surfaces from the logo's hue, the lighter brand step 300 as accent, and a lighter logo
    (`thorsoft-logo-dark.svg`: the same drawing, colours swapped, shading kept).
  - The app, window, installer and tray icons don't change.

Amendment to §2 "Frontend": the PrimeVue theme is a preset derived from Aura with the brand
scale as `primary`, defined once in `ui/src/lib/brand.ts`, with its own dark surfaces and accent; it follows the
system setting. The quick-switch
popup doesn't use PrimeVue and mirrors the accent colour in its own CSS.

## 2. Design
- `ui/src/lib/brand.ts`: the brand scale (50–950) and `ThorSoftPreset = definePreset(Aura,
  { semantic: { primary: brand, colorScheme: { dark: … } } })` with `night` as the dark surfaces.
  `main.ts` installs it with `darkModeSelector: "system"`. `BrandLogo` picks the logo with a
  `<picture>` on the same media query. Text colours without a PrimeVue token are CSS variables
  in `style.css` (`--brand-text`, `--error-text`) with a dark value.
- Components use PrimeVue tokens (`--p-primary-color`, `--p-primary-800`, …), not hex literals.
- `BrandLogo.vue` renders the SVG at a given size; `BrandMark.vue` is logo + wordmark for the
  nav bar. Both windows use `BrandLogo`.
- The version shown in About comes from `ui/package.json` at build time (Vite `define`). A Rust
  test keeps `Cargo.toml`, `tauri.conf.json` and `ui/package.json` on the same version.

## 3. Constraint traceability
| Intent constraint | How it is met |
|---|---|
| Follow Windows light/dark | `darkModeSelector: "system"`, `color-scheme: light dark` and a dark block in both windows; the popup mirrors the dark values |
| Readable in both modes | unit tests: light accent on white; dark accent, text, muted, brand and error text on the dark surfaces ≥ 4.5:1; dark logo colours ≥ 3:1 |
| UI only | no change outside `ui/` except a version consistency test in `src-tauri` |

## 4. Open questions
None.

## 5. Areas of concern
- **AC-1 Contrast.** Pinned by `brand.test.ts` (500 on white and white on 500, both ≥ 4.5:1).
- **AC-2 Version drift.** The About version must match the installed app; pinned by the Rust
  version test.
- **AC-3 Unseen visual regressions.** No browser tests (owner, 2026-10-18); checked with dev app
  screenshots, also with Windows in dark mode.
