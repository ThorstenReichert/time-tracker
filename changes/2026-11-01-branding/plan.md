# Plan: ThorSoft branding (logo and colour theme)
Source: [spec.md](spec.md), [intent.md](intent.md). Author: Thorsten Reichert (with Claude).
Status: approved (owner, 2026-11-01). Branch: `feat/branding`.
Scope: R25 and the amendment to spec §2 "Frontend".

## Context
The app shows no ThorSoft logo and uses PrimeVue's stock emerald colours. The owner decided:
light only, no icon changes, the logo in the nav bar, a Settings "About" section and the
quick-switch popup, and a compact change chain.

## Risks
- **`darkModeSelector: false`** must really suppress the dark scheme in PrimeVue 4.5; fallback
  is a selector that never matches.
- **Contrast** of the accent on white; pinned by a unit test.
- **Visual regressions** no test sees; covered by dev app screenshots, also in Windows dark mode.
- Rejected: replacing the app and tray icons (owner), a branded dark theme (owner), reading the
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

## Deviations
- Step 2: only the SVG is in the repo; the owner removed the PNG and ICO exports, which nothing used.
- Step 3: the version test also checks `ui/package-lock.json`, which a version bump changes too.
- Step 4: `brand.test.ts` also pins that the popup's `--primary` equals brand 500 and that the
  popup has no dark variant, so the mirrored colour can't drift.
- Proof: the owner checked the dev app by hand instead of screenshots; no migration dry run
  (no database change).

## Coverage
R25: steps 1–4. §2 amendment: step 1 (+ step 5 docs).

## Proof
- `gatekeeper` runs the full `pwsh scripts/check.ps1` after the steps (`ALL CHECKS PASSED`).
- Dev app screenshots: Today (idle and running), Daily overview, Clients, Settings/About, a
  dialog, the quick-switch popup; once with Windows in dark mode.
