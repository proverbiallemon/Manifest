import { describe, expect, it } from "vitest";
import { conflictCount, deriveZones, firstRowIndexes, overlapsFor, sortNeeded, zoneNameLists, zonesShifted, type Report } from "./types";
import { mkMod, mkReport } from "./lib/testReport";

const base: Report = {
  schema_version: 3,
  mods: [mkMod({ name: "Big" }), mkMod({ name: "Small" }), mkMod({ name: "Other" })],
  conflicts: [
    { asset: "a", providers: ["Big", "Small"], winner: "Small" },
    { asset: "b", providers: ["Big", "Other"], winner: "Other" },
  ],
  warnings: [],
  current_order: ["Big", "Small"],
  proposed_order: ["Big", "Small"],
  moves: [],
  sort_held_by_pins: false,
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

  it("ignores stale names of disabled mods left in the current order", () => {
    const report: Report = {
      ...base,
      mods: [
        mkMod({ name: "Big" }),
        mkMod({ name: "Small" }),
        mkMod({ name: "Parked", path: "/mods/Parked.disabled", enabled: false }),
      ],
      current_order: ["Big", "Parked", "Small"],
      proposed_order: ["Big", "Small"],
    };
    expect(sortNeeded(report)).toBe(false);
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

describe("deriveZones", () => {
  const report = mkReport({
    mods: [
      mkMod({ name: "Broad", pinned: "top" }),
      mkMod({ name: "Mid" }),
      mkMod({ name: "Fine", pinned: "bottom" }),
      mkMod({ name: "Loose" }),
      mkMod({ name: "Ashore", enabled: false }),
      mkMod({ name: "Stray" }), // enabled but not in order: NOT LOADED
    ],
    current_order: ["Mid", "Broad", "Fine", "Loose", "Ghost"],
  });

  it("groups loaded mods into fore, free, aft in current_order sequence", () => {
    const z = deriveZones(report);
    expect(z.fore.map((m) => m.name)).toEqual(["Broad"]);
    expect(z.free.map((m) => m.name)).toEqual(["Mid", "Loose"]);
    expect(z.aft.map((m) => m.name)).toEqual(["Fine"]);
  });

  it("excludes disabled mods, stale names, and mods missing from the order", () => {
    const all = Object.values(deriveZones(report)).flat();
    const names = all.map((m) => m.name);
    expect(names).not.toContain("Ashore");
    expect(names).not.toContain("Ghost");
    expect(names).not.toContain("Stray");
  });

  it("keeps duplicate-named rows but zoneNameLists dedupes them", () => {
    const dup = mkReport({
      mods: [
        mkMod({ name: "Twin", path: "/mods/a/Twin.otr" }),
        mkMod({ name: "Twin", path: "/mods/b/Twin.otr" }),
      ],
      current_order: ["Twin"],
    });
    const z = deriveZones(dup);
    expect(z.free.length).toBe(2);
    expect(zoneNameLists(z).free).toEqual(["Twin"]);
  });
});

describe("firstRowIndexes", () => {
  it("returns the row index of each unique name's first occurrence, in first-seen order", () => {
    const mods = [
      mkMod({ name: "Twin", path: "/mods/a/Twin.otr" }),
      mkMod({ name: "Twin", path: "/mods/b/Twin.otr" }),
      mkMod({ name: "Other" }),
    ];
    expect(firstRowIndexes(mods)).toEqual([0, 2]);
  });

  it("returns one index per row when there are no duplicates", () => {
    const mods = [mkMod({ name: "A" }), mkMod({ name: "B" }), mkMod({ name: "C" })];
    expect(firstRowIndexes(mods)).toEqual([0, 1, 2]);
  });

  it("returns an empty array for an empty zone", () => {
    expect(firstRowIndexes([])).toEqual([]);
  });
});

describe("zonesShifted", () => {
  it("is false when pinned mods already sit at the edges", () => {
    const r = mkReport({
      mods: [mkMod({ name: "A", pinned: "top" }), mkMod({ name: "B" })],
      current_order: ["A", "B"],
    });
    expect(zonesShifted(r)).toBe(false);
  });

  it("is true when a pinned mod sits mid-list in the actual order", () => {
    const r = mkReport({
      mods: [
        mkMod({ name: "A" }),
        mkMod({ name: "P", pinned: "top" }),
        mkMod({ name: "B" }),
      ],
      current_order: ["A", "P", "B"],
    });
    expect(zonesShifted(r)).toBe(true);
  });

  it("ignores stale names and duplicate rows when comparing", () => {
    const r = mkReport({
      mods: [
        mkMod({ name: "Twin", path: "/mods/a/Twin.otr" }),
        mkMod({ name: "Twin", path: "/mods/b/Twin.otr" }),
        mkMod({ name: "B", pinned: "bottom" }),
      ],
      current_order: ["Twin", "Stale", "B"],
    });
    expect(zonesShifted(r)).toBe(false);
  });
});
