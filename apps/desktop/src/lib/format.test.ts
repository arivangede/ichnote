import { describe, expect, it } from "vitest";
import { displayTitle, formatNoteDate } from "./format";

describe("formatNoteDate", () => {
  const now = new Date(2026, 10, 19, 15, 0);
  it("shows the time for today", () => {
    expect(formatNoteDate(new Date(2026, 10, 19, 9, 5).getTime(), now, "en-GB")).toMatch(
      /^0?9:05$/,
    );
  });
  it("shows day and month this year", () => {
    expect(formatNoteDate(new Date(2026, 2, 3).getTime(), now, "en-GB")).toBe("3 Mar");
  });
  it("adds the year for older notes", () => {
    expect(formatNoteDate(new Date(2024, 2, 3).getTime(), now, "en-GB")).toBe("3 Mar 2024");
  });
});

describe("displayTitle", () => {
  it("falls back for blank titles", () => {
    expect(displayTitle("  ")).toBe("Untitled");
    expect(displayTitle("Plan")).toBe("Plan");
  });
});
