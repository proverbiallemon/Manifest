import { describe, expect, it } from "vitest";
import { conflictCount, overlapsFor, sortNeeded, type Report } from "./types";

const base: Report = {
  schema_version: 2,
  mods: [],
  conflicts: [
    { asset: "a", providers: ["Big", "Small"], winner: "Small" },
    { asset: "b", providers: ["Big", "Other"], winner: "Other" },
  ],
  warnings: [],
  current_order: ["Big", "Small"],
  proposed_order: ["Big", "Small"],
  moves: [],
};

describe("conflictCount", () => {
  it("counts conflicts a mod participates in", () => {
    expect(conflictCount(base, "Big")).toBe(2);
    expect(conflictCount(base, "Small")).toBe(1);
    expect(conflictCount(base, "Clean")).toBe(0);
  });
});

describe("sortNeeded", () => {
  it("is false when orders match and true when they differ", () => {
    expect(sortNeeded(base)).toBe(false);
    expect(
      sortNeeded({ ...base, proposed_order: ["Small", "Big"] })
    ).toBe(true);
  });
});

describe("overlapsFor", () => {
  it("splits conflicts into prevails and overridden with counts", () => {
    const r: Report = {
      ...base,
      conflicts: [
        { asset: "a", providers: ["Big", "Small"], winner: "Small" },
        { asset: "b", providers: ["Big", "Small"], winner: "Small" },
        { asset: "c", providers: ["Big", "Other"], winner: "Other" },
        { asset: "d", providers: ["Big", "Late"], winner: "Big" },
      ],
    };
    const big = overlapsFor(r, "Big");
    expect(big.overriddenBy).toEqual([
      { name: "Small", count: 2 },
      { name: "Other", count: 1 },
    ]);
    expect(big.prevailsOver).toEqual([{ name: "Late", count: 1 }]);
    expect(big.contestedAssets).toEqual(["a", "b", "c", "d"]);
    expect(overlapsFor(r, "Clean")).toEqual({
      prevailsOver: [],
      overriddenBy: [],
      contestedAssets: [],
    });
  });
});
