import { describe, expect, it } from "vitest";
import { applyDrag, slotFromPointer, type ZoneBand } from "./dragPlan";

const bands: ZoneBand[] = [
  { zone: "fore", top: 0, bottom: 40, rowMids: [20] },
  { zone: "free", top: 40, bottom: 160, rowMids: [60, 100, 140] },
  { zone: "aft", top: 160, bottom: 200, rowMids: [180] },
];

describe("slotFromPointer", () => {
  it("finds the slot inside the band containing y", () => {
    expect(slotFromPointer(bands, 50)).toEqual({ zone: "free", index: 0 });
    expect(slotFromPointer(bands, 90)).toEqual({ zone: "free", index: 1 });
    expect(slotFromPointer(bands, 155)).toEqual({ zone: "free", index: 3 });
  });

  it("clamps to the nearest band when y is outside every band", () => {
    expect(slotFromPointer(bands, -50)).toEqual({ zone: "fore", index: 0 });
    expect(slotFromPointer(bands, 500)).toEqual({ zone: "aft", index: 1 });
  });

  it("lands in an empty zone band", () => {
    const withEmpty: ZoneBand[] = [
      { zone: "fore", top: 0, bottom: 20, rowMids: [] },
      { zone: "free", top: 20, bottom: 100, rowMids: [40, 80] },
      { zone: "aft", top: 100, bottom: 120, rowMids: [] },
    ];
    expect(slotFromPointer(withEmpty, 10)).toEqual({ zone: "fore", index: 0 });
    expect(slotFromPointer(withEmpty, 110)).toEqual({ zone: "aft", index: 0 });
  });
});

describe("applyDrag", () => {
  const names = { fore: ["P"], free: ["A", "B", "C"], aft: ["Q"] };

  it("reorders within a zone, adjusting for the removed row", () => {
    const out = applyDrag(
      names,
      { zone: "free", index: 0 },
      { zone: "free", index: 3 }
    );
    expect(out.free).toEqual(["B", "C", "A"]);
  });

  it("moving to its own slot or the next slot is a no-op", () => {
    const same = applyDrag(
      names,
      { zone: "free", index: 1 },
      { zone: "free", index: 1 }
    );
    expect(same.free).toEqual(["A", "B", "C"]);
    const next = applyDrag(
      names,
      { zone: "free", index: 1 },
      { zone: "free", index: 2 }
    );
    expect(next.free).toEqual(["A", "B", "C"]);
  });

  it("moves across zones, which changes pin membership", () => {
    const out = applyDrag(
      names,
      { zone: "free", index: 2 },
      { zone: "aft", index: 0 }
    );
    expect(out.free).toEqual(["A", "B"]);
    expect(out.aft).toEqual(["C", "Q"]);
  });

  it("does not mutate its input", () => {
    applyDrag(names, { zone: "free", index: 0 }, { zone: "aft", index: 1 });
    expect(names.free).toEqual(["A", "B", "C"]);
    expect(names.aft).toEqual(["Q"]);
  });
});
