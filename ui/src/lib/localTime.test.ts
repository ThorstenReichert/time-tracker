import { describe, expect, it } from "vitest";
import { formatClock, formatSpan, joinLocal, parseTime, splitLocal } from "./localTime";

describe("parseTime", () => {
  it("reads the usual ways to type a time", () => {
    expect(parseTime("9")).toBe("09:00:00");
    expect(parseTime("9:05")).toBe("09:05:00");
    expect(parseTime("09.05")).toBe("09:05:00");
    expect(parseTime("905")).toBe("09:05:00");
    expect(parseTime("0905")).toBe("09:05:00");
    expect(parseTime(" 23:59:30 ")).toBe("23:59:30");
    expect(parseTime("0:00")).toBe("00:00:00");
  });

  it("rejects what isn't a time of day", () => {
    for (const text of ["", "24:00", "9:60", "9:05:60", "9:5", "noon", "12345", "-1"]) {
      expect(parseTime(text), text).toBeNull();
    }
  });
});

describe("local times", () => {
  it("split into the date and a time that keeps its seconds only if it has some", () => {
    expect(splitLocal("2026-10-18T09:00:00")).toEqual({ date: "2026-10-18", time: "09:00" });
    expect(splitLocal("2026-10-18T09:00:13")).toEqual({ date: "2026-10-18", time: "09:00:13" });
  });

  it("join a date and a typed time, and round-trip with split", () => {
    expect(joinLocal("2026-03-29", "2:30")).toBe("2026-03-29T02:30:00");
    expect(joinLocal("2026-03-29", "later")).toBeNull();
    for (const local of ["2026-10-18T09:00:00", "2026-10-18T09:00:13"]) {
      const { date, time } = splitLocal(local);
      expect(joinLocal(date, time)).toBe(local);
    }
  });

  it("format as clock times and spans", () => {
    expect(formatClock("2026-10-18T09:05:13")).toBe("09:05");
    expect(formatSpan("2026-10-18T09:00:00", "2026-10-18T10:22:00")).toBe("09:00–10:22");
    expect(formatSpan("2026-10-18T09:00:00", null)).toBe("09:00–now");
    expect(formatSpan("2026-10-18T22:00:00", "2026-10-19T01:30:00")).toBe("22:00–01:30 (next day)");
    expect(formatSpan("2026-10-18T22:00:00", "2026-10-20T01:30:00")).toBe("22:00–01:30 (2026-10-20)");
  });
});
