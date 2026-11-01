# Spec: ThorSoft branding (logo and colour theme)
Source: [intent.md](intent.md). Amends: [../../spec.md](../../spec.md). Author: Thorsten Reichert.
Status: approved (owner, 2026-11-01).

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
  - **Light only.** Both windows stay light when Windows is in dark mode.
  - The app, window, installer and tray icons don't change.

Amendment to §2 "Frontend": the PrimeVue theme is a preset derived from Aura with the brand
scale as `primary`, defined once in `ui/src/lib/brand.ts`; dark mode is off. The quick-switch
popup doesn't use PrimeVue and mirrors the accent colour in its own CSS.

## 2. Design
- `ui/src/lib/brand.ts`: the brand scale (50–950) and `ThorSoftPreset = definePreset(Aura,
  { semantic: { primary: brand } })`. `main.ts` installs it with dark mode off.
- Components use PrimeVue tokens (`--p-primary-color`, `--p-primary-800`, …), not hex literals.
- `BrandLogo.vue` renders the SVG at a given size; `BrandMark.vue` is logo + wordmark for the
  nav bar. Both windows use `BrandLogo`.
- The version shown in About comes from `ui/package.json` at build time (Vite `define`). A Rust
  test keeps `Cargo.toml`, `tauri.conf.json` and `ui/package.json` on the same version.

## 3. Constraint traceability
| Intent constraint | How it is met |
|---|---|
| Light only | dark mode selector off in PrimeVue; `color-scheme: light` in both windows; the popup's dark block removed |
| Accent readable on white | unit test: contrast of brand 500 on white ≥ 4.5:1 |
| UI only | no change outside `ui/` except a version consistency test in `src-tauri` |

## 4. Open questions
None.

## 5. Areas of concern
- **AC-1 Contrast.** Pinned by `brand.test.ts` (500 on white and white on 500, both ≥ 4.5:1).
- **AC-2 Version drift.** The About version must match the installed app; pinned by the Rust
  version test.
- **AC-3 Unseen visual regressions.** No browser tests (owner, 2026-10-18); checked with dev app
  screenshots, also with Windows in dark mode.
