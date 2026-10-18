import { describe, expect, it } from "vitest";
import { formatElapsed } from "./format";

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
