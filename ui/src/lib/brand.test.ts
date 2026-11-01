import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { brand } from "./brand";

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
    const l = Object.values(brand).map(luminance);
    for (let i = 1; i < l.length; i++) expect(l[i]).toBeLessThan(l[i - 1]!);
  });

  it("keeps the accent readable on white and white text readable on the accent (WCAG AA)", () => {
    // PrimeVue's light scheme uses primary 500 as the accent, with white as its contrast colour.
    expect(contrast(brand[500], "#ffffff")).toBeGreaterThanOrEqual(4.5);
    // Hover and active states (600, 700) only get darker, so they pass too.
    expect(contrast(brand[600], "#ffffff")).toBeGreaterThan(contrast(brand[500], "#ffffff"));
  });

  it("is mirrored by the quick-switch popup, which doesn't use PrimeVue", () => {
    const popup = readFileSync("src/quick-switch/QuickSwitch.vue", "utf8");
    expect(popup.match(/--primary: (#[0-9a-f]{6});/)?.[1]).toBe(brand[500]);
    // Light only: the popup has no dark variant.
    expect(popup).not.toContain("prefers-color-scheme: dark");
  });
});
