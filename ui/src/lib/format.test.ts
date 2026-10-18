import { describe, expect, it } from "vitest";
import { decimalMark, formatDiff, formatElapsed, formatHours, formatRule } from "./format";

describe("formatElapsed", () => {
  it("shows hours, minutes and seconds", () => {
    expect(formatElapsed(0)).toBe("0:00:00");
    expect(formatElapsed(59)).toBe("0:00:59");
    expect(formatElapsed(4920)).toBe("1:22:00");
    expect(formatElapsed(36 * 3600 + 61)).toBe("36:01:01");
  });

  it("never shows a negative time", () => {
    expect(formatElapsed(-5)).toBe("0:00:00");
  });
});

describe("formatHours", () => {
  it("shows decimal hours with 2 decimals (R15)", () => {
    expect(formatHours(0, ".")).toBe("0.00");
    expect(formatHours(900, ".")).toBe("0.25");
    expect(formatHours(1, ".")).toBe("0.00");
    expect(formatHours(1800, ".")).toBe("0.50");
    expect(formatHours(4920, ".")).toBe("1.37");
    expect(formatHours(4500, ".")).toBe("1.25");
    expect(formatHours(36 * 3600, ".")).toBe("36.00");
  });

  it("rounds half away from zero", () => {
    // 18 s is exactly half a hundredth of an hour.
    expect(formatHours(18, ".")).toBe("0.01");
    expect(formatHours(17, ".")).toBe("0.00");
    expect(formatHours(-18, ".")).toBe("-0.01");
    expect(formatHours(-17, ".")).toBe("0.00");
  });

  it("shows negative values", () => {
    expect(formatHours(-900, ".")).toBe("-0.25");
    expect(formatHours(-420, ".")).toBe("-0.12");
  });

  it("uses the decimal mark", () => {
    expect(formatHours(4500, ",")).toBe("1,25");
    expect(formatHours(-900, ",")).toBe("-0,25");
  });
});

describe("formatDiff", () => {
  it("signs the difference", () => {
    expect(formatDiff(120, ".")).toBe("+0.03");
    expect(formatDiff(-420, ",")).toBe("-0,12");
    expect(formatDiff(0, ".")).toBe("0.00");
    expect(formatDiff(1, ".")).toBe("0.00");
  });
});

describe("decimalMark", () => {
  it("follows the setting", () => {
    expect(decimalMark("dot", "de-DE")).toBe(".");
    expect(decimalMark("comma", "en-US")).toBe(",");
  });

  it("follows the locale on auto (AC-3)", () => {
    expect(decimalMark("auto", "de-DE")).toBe(",");
    expect(decimalMark("auto", "en-US")).toBe(".");
  });
});

describe("formatRule", () => {
  it("names unit, direction and level", () => {
    expect(formatRule({ unit: 15, direction: "nearest", level: "project" })).toBe("15 min · nearest · per project");
  });
});
