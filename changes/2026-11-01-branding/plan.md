# Plan: ThorSoft branding (logo and colour theme)
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-01); step 6 added the same day (owner). Branch: `feat/branding`.
Scope: R25 and the amendment to spec §2 "Frontend".

## Context
The app shows no ThorSoft logo and uses PrimeVue's stock emerald colours. The owner decided:
light only (reversed the same day, see step 6), no icon changes, the logo in the nav bar, a Settings "About" section and the
quick-switch popup, and a compact change chain.

## Risks
- **Following Windows** in both windows the same way: PrimeVue `darkModeSelector: "system"`,
  the media queries in `style.css` and the popup, and the logo `<picture>` all use
  `prefers-color-scheme`. (Superseded: `darkModeSelector: false` for light only.)
- **Contrast** of accent and text in both modes; pinned by unit tests on the resolved theme.
- **Visual regressions** no test sees; covered by dev app screenshots, also in Windows dark mode.
- Rejected: replacing the app and tray icons (owner), reading the
  version over IPC (needs a capability; the build-time value is simpler and pinned by a test).

## Steps
1. **Brand palette and preset (R25).** `ui/src/lib/brand.ts`: the brand scale around the logo
   colours and `ThorSoftPreset`. `main.ts`: the preset, dark mode off. `style.css`:
   `color-scheme: light`. Test `lib/brand.test.ts`: 11 steps, logo colours at 500/800/950,
   contrast ≥ 4.5:1.
2. **Logo components (R25).** `components/BrandLogo.vue` (the SVG at a size) and
   `components/BrandMark.vue` (logo + "TimeTracker"), with tests. Commit the SVG.
3. **Main window (R25).** `App.vue` places `BrandMark` left of the nav links. `SettingsView.vue`
   gets an About section (logo, "ThorSoft TimeTracker", version). `vite.config.ts` defines
   `__APP_VERSION__` from `ui/package.json`, declared in `env.d.ts`. A Rust test in
   `src-tauri` checks that `Cargo.toml`, `tauri.conf.json` and `ui/package.json` agree.
   `SettingsView.test.ts` checks the About section.
4. **Quick-switch popup (R25).** `QuickSwitch.vue`: no dark block, `color-scheme: light`,
   `--primary` mirrors brand 500, `BrandLogo` at the left of the header row.
   `QuickSwitch.test.ts` checks the logo.
5. **Docs and release.** `CLAUDE.md`/`AGENTS.md` (identical): where the theme lives. Root
   `spec.md` header: the amendment. Release notes at stage 5.
6. **Dark mode (R25, revised).** `lib/brand.ts`: `night` surfaces, the dark accent and
   `darkLogoColours`; `main.ts` follows the system. `assets/thorsoft-logo-dark.svg` (the
   original with the colours swapped). `BrandLogo.vue`: a `<picture>` with the dark source.
   `style.css`: `--brand-text`, `--error-text` with dark values, used by `BrandMark`,
   `SettingsView`, `DayBar`, `TodayView`, `OverviewView`. `QuickSwitch.vue`: a dark block.
   Tests: dark contrast, the popup's dark mirror, the dark logo derived from the original.

## Deviations
- Step 6 (owner, 2026-11-01): the owner reversed "light only" after seeing the light theme;
  the dark mode replaces steps 1 and 4's light-only parts.
- Step 2: only the SVG is in the repo; the owner removed the PNG and ICO exports, which nothing used.
- Step 3: the version test also checks `ui/package-lock.json`, which a version bump changes too.
- Step 4: `brand.test.ts` also pins the popup's mirrored colours (since step 6 for both modes),
  so they can't drift from the main window's theme.
- Stage 5 review fixes (2026-11-01):
  - Light `--error-text` is red 600 instead of 500 (3.8:1 on white failed AA). It now matches
    the popup's `--danger`.
  - `brand.test.ts` checks the theme PrimeVue actually uses: it resolves `ThorSoftPreset`'s
    tokens and the `style.css` variables, so a changed accent, surface or text colour fails a
    test. The popup's mirror is compared with the resolved theme in both modes.
  - That comparison showed the popup's dark border used the content border (night 700) while
    its light border uses the form-field border; dark now uses the form-field border (night 600).
  - Stale light-only wording fixed in the root and change spec, `plan.md` and a popup comment.
- Proof: the owner checked the dev app by hand instead of screenshots; no migration dry run
  (no database change).

## Coverage
R25: steps 1–4 and 6 (dark mode). §2 amendment: steps 1 and 6 (+ step 5 docs).

## Proof
- `gatekeeper` runs the full `pwsh scripts/check.ps1` after the steps (`ALL CHECKS PASSED`).
- Dev app screenshots: Today (idle and running), Daily overview, Clients, Settings/About, a
  dialog, the quick-switch popup; once with Windows in dark mode.
