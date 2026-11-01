import { readFileSync } from "node:fs";
import Aura from "@primeuix/themes/aura";
import { describe, expect, it } from "vitest";
import { brand, darkLogoColours, night, ThorSoftPreset } from "./brand";

// WCAG 2 relative luminance and contrast ratio of two #rrggbb colours.
function luminance(hex: string): number {
  const channel = (i: number) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
}
function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}
function darkensStepByStep(scale: Record<number, string>) {
  const l = Object.values(scale).map(luminance);
  for (let i = 1; i < l.length; i++) expect(l[i]).toBeLessThan(l[i - 1]!);
}

// The theme as PrimeVue sees it: ThorSoftPreset merged over Aura, with its tokens resolved.
type Palette = Record<string, string>;
type Scheme = {
  surface: Palette;
  primary: { color: string; contrastColor: string; hoverColor: string; activeColor: string };
  text: { color: string; mutedColor: string };
  content: { background: string };
  formField: { background: string; borderColor: string };
};
const preset = ThorSoftPreset as unknown as {
  primitive: Record<string, Palette>;
  semantic: { primary: Palette; colorScheme: { light: Scheme; dark: Scheme } };
};
const modes = ["light", "dark"] as const;
type Mode = (typeof modes)[number];
const scheme = (mode: Mode) => preset.semantic.colorScheme[mode];

/** Resolves a token like `{primary.300}` or `{surface.950}` to its hex colour in `mode`. */
function resolve(value: string, mode: Mode): string {
  const token = /^\{(\w+)\.(\d+)\}$/.exec(value);
  if (token === null) return value;
  const [, palette, step] = token as unknown as [string, string, string];
  const source =
    palette === "primary" ? preset.semantic.primary : palette === "surface" ? scheme(mode).surface : preset.primitive[palette];
  const next = source?.[step];
  if (next === undefined) throw new Error(`unknown token ${value}`);
  return resolve(next, mode);
}
const accent = (mode: Mode) => resolve(scheme(mode).primary.color, mode);
const background = (mode: Mode) => resolve(scheme(mode).content.background, mode);
const fieldBackground = (mode: Mode) => resolve(scheme(mode).formField.background, mode);

/** The app's own text colours (style.css), per mode: `var(--p-red-600)` → PrimeVue's red 600. */
function styleVar(name: string, mode: Mode): string {
  const [light, dark] = readFileSync("src/style.css", "utf8").split("@media (prefers-color-scheme: dark)") as [string, string];
  const match = new RegExp(String.raw`--${name}: var\(--p-(\w+)-(\d+)\);`).exec(mode === "light" ? light : dark);
  if (match === null) throw new Error(`--${name} not set for ${mode} mode`);
  return resolve(`{${match[1]}.${match[2]}}`, mode);
}

describe("brand", () => {
  it("is a full 50-950 scale of colours", () => {
    expect(Object.keys(brand).map(Number)).toEqual([50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950]);
    for (const colour of Object.values(brand)) expect(colour).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("uses the logo's colours", () => {
    expect(brand[500]).toBe("#436088");
    expect(brand[800]).toBe("#1f3453");
    expect(brand[950]).toBe("#0d1d37");
  });

  it("gets darker step by step", () => {
    darkensStepByStep(brand);
    darkensStepByStep(night);
  });
});

describe("ThorSoftPreset", () => {
  it("is Aura with the brand as primary: light on white, dark on the navy `night` surfaces", () => {
    expect(preset.semantic.primary).toEqual(brand);
    expect(preset.primitive).toEqual(Aura.primitive);
    expect(background("light")).toBe("#ffffff");
    expect(scheme("dark").surface).toEqual(night);
    expect(accent("light")).toBe(brand[500]);
    expect(accent("dark")).toBe(brand[300]);
  });

  for (const mode of modes) {
    it(`${mode} mode: accent, text and the app's own text colours are readable (WCAG AA)`, () => {
      const s = scheme(mode);
      const texts = {
        accent: accent(mode),
        text: resolve(s.text.color, mode),
        muted: resolve(s.text.mutedColor, mode),
        "--brand-text": styleVar("brand-text", mode),
        "--error-text": styleVar("error-text", mode),
      };
      for (const bg of [background(mode), fieldBackground(mode)]) {
        for (const [what, colour] of Object.entries(texts)) {
          expect(contrast(colour, bg), `${what} ${colour} on ${bg}`).toBeGreaterThanOrEqual(4.5);
        }
      }
      // Buttons: the contrast colour on the accent and its hover and active states.
      for (const state of [s.primary.color, s.primary.hoverColor, s.primary.activeColor]) {
        expect(contrast(resolve(s.primary.contrastColor, mode), resolve(state, mode))).toBeGreaterThanOrEqual(4.5);
      }
    });
  }

  it("is mirrored by the quick-switch popup, which doesn't use PrimeVue", () => {
    const popup = readFileSync("src/quick-switch/QuickSwitch.vue", "utf8");
    const values = (name: string) => [...popup.matchAll(new RegExp(`--${name}: (#[0-9a-f]{6});`, "g"))].map((m) => m[1]);
    // The first value is light mode's, the second dark mode's.
    const expected = (pick: (mode: Mode) => string) => modes.map(pick);
    expect(values("primary")).toEqual(expected(accent));
    expect(values("surface")).toEqual(expected(background));
    expect(values("text")).toEqual(expected((m) => resolve(scheme(m).text.color, m)));
    expect(values("muted")).toEqual(expected((m) => resolve(scheme(m).text.mutedColor, m)));
    expect(values("border")).toEqual(expected((m) => resolve(scheme(m).formField.borderColor, m)));
    expect(values("danger")).toEqual(expected((m) => styleVar("error-text", m)));
  });
});

describe("dark logo", () => {
  const original = readFileSync("src/assets/thorsoft-logo.svg", "utf8");
  const dark = readFileSync("src/assets/thorsoft-logo-dark.svg", "utf8");

  it("is the original drawing with only the colours swapped, so the two can't drift apart", () => {
    let expected = original;
    for (const [from, to] of Object.entries(darkLogoColours)) expected = expected.replaceAll(from, to);
    expect(dark).toBe(expected);
    for (const from of Object.keys(darkLogoColours)) expect(dark).not.toContain(from);
  });

  it("keeps the logo's shading (top lightest, then sides, then front)", () => {
    const top = darkLogoColours[brand[500]];
    const side = darkLogoColours[brand[800]];
    const front = darkLogoColours[brand[950]];
    expect(luminance(top)).toBeGreaterThan(luminance(side));
    expect(luminance(side)).toBeGreaterThan(luminance(front));
  });

  it("stands out on the dark surfaces (WCAG graphics contrast, 3:1)", () => {
    for (const colour of Object.values(darkLogoColours)) {
      expect(contrast(colour, background("dark"))).toBeGreaterThanOrEqual(3);
    }
  });
});
