import { describe, expect, it } from "vitest";
import { addDays, formatDay, formatPeriod, fromPickerDate, toPickerDate } from "./day";

describe("addDays", () => {
  it("steps over month and year ends", () => {
    expect(addDays("2026-10-18", 1)).toBe("2026-10-19");
    expect(addDays("2026-10-31", 1)).toBe("2026-11-01");
    expect(addDays("2027-01-01", -1)).toBe("2026-12-31");
    expect(addDays("2028-02-28", 1)).toBe("2028-02-29");
  });

  it("isn't shifted by the DST changes", () => {
    expect(addDays("2026-03-28", 1)).toBe("2026-03-29");
    expect(addDays("2026-03-29", 1)).toBe("2026-03-30");
    expect(addDays("2026-10-25", -1)).toBe("2026-10-24");
    expect(addDays("2026-10-25", 1)).toBe("2026-10-26");
  });
});

describe("picker dates", () => {
  it("round-trip", () => {
    for (const day of ["2026-01-01", "2026-03-29", "2026-10-25", "2026-12-31"]) {
      expect(fromPickerDate(toPickerDate(day))).toBe(day);
    }
  });
});

describe("formatDay", () => {
  // Punctuation differs between ICU versions, so only the parts are checked.
  it("names the weekday, day, month and year", () => {
    expect(formatDay("2026-10-18", "en-GB").split(/[ ,.]+/)).toEqual(["Sunday", "18", "October", "2026"]);
    expect(formatDay("2026-10-19", "de-DE").split(/[ ,.]+/)).toEqual(["Montag", "19", "Oktober", "2026"]);
  });
});

describe("formatPeriod", () => {
  // Punctuation differs between ICU versions, so only the parts are checked.
  const words = (text: string) => text.split(/[ ,.]+/).filter((w) => w !== "");
  it("shows the first and the last day (R2e)", () => {
    expect(words(formatPeriod("2026-03-02", "2026-10-18", "en-GB"))).toEqual(["2", "Mar", "2026", "–", "18", "Oct", "2026"]);
  });

  it("shows a single day once", () => {
    expect(words(formatPeriod("2026-10-18", "2026-10-18", "en-GB"))).toEqual(["18", "Oct", "2026"]);
  });
});
