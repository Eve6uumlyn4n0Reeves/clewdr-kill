import { describe, it, expect } from "vitest";
import { formatTimeElapsed, formatIsoTimestamp } from "./formatters";

describe("formatters", () => {
  it("formats elapsed seconds", () => {
    expect(formatTimeElapsed(45)).toBe("45 sec");
    expect(formatTimeElapsed(125)).toBe("2 min 5 sec");
    expect(formatTimeElapsed(3661)).toBe("1 hr 1 min");
  });

  it("formats ISO timestamp", () => {
    const iso = "2024-01-01T00:00:00Z";
    expect(formatIsoTimestamp(iso)).toContain("2024");
  });
});
