import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { brand, darkLogoColours, night } from "./brand";

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

const RED_400 = "#f87171"; // PrimeVue's red 400, dark mode's --error-text

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

  it("light mode: the accent is readable on white, and white text on the accent (WCAG AA)", () => {
    // PrimeVue's light scheme uses primary 500 as the accent, with white as its contrast colour.
    expect(contrast(brand[500], "#ffffff")).toBeGreaterThanOrEqual(4.5);
    // Hover and active states (600, 700) only get darker, so they pass too.
    expect(contrast(brand[600], "#ffffff")).toBeGreaterThan(contrast(brand[500], "#ffffff"));
  });

  it("dark mode: accent, text, muted text and errors are readable on the navy surfaces (WCAG AA)", () => {
    // Content and dialogs are on night 900, form fields on night 950.
    for (const background of [night[900], night[950]]) {
      expect(contrast(brand[300], background)).toBeGreaterThanOrEqual(4.5); // accent
      expect(contrast(night[0], background)).toBeGreaterThanOrEqual(4.5); // text
      expect(contrast(night[400], background)).toBeGreaterThanOrEqual(4.5); // muted text
      expect(contrast(brand[100], background)).toBeGreaterThanOrEqual(4.5); // --brand-text
      expect(contrast(RED_400, background)).toBeGreaterThanOrEqual(4.5); // --error-text
    }
    // Buttons: night 950 text on the accent and its hover and active states.
    for (const accent of [brand[300], brand[200], brand[100]]) {
      expect(contrast(night[950], accent)).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("is mirrored by the quick-switch popup, which doesn't use PrimeVue", () => {
    const popup = readFileSync("src/quick-switch/QuickSwitch.vue", "utf8");
    const values = (name: string) => [...popup.matchAll(new RegExp(`--${name}: (#[0-9a-f]{6});`, "g"))].map((m) => m[1]);
    // The first value is light mode's, the second dark mode's.
    expect(values("primary")).toEqual([brand[500], brand[300]]);
    expect(values("surface")).toEqual(["#ffffff", night[900]]);
    expect(values("text")[1]).toBe(night[0]);
    expect(values("muted")[1]).toBe(night[400]);
    expect(values("border")[1]).toBe(night[700]);
    expect(values("danger")[1]).toBe(RED_400);
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
      expect(contrast(colour, night[900])).toBeGreaterThanOrEqual(3);
    }
  });
});
