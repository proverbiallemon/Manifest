import { describe, expect, it } from "vitest";
import {
  applyDrag,
  blockHeight,
  rowShifts,
  settleY,
  slotFromPointer,
  slotY,
  type ZoneBand,
} from "./dragPlan";

const bands: ZoneBand[] = [
  { zone: "fore", top: 0, bottom: 40, rowMids: [20], rowTops: [10] },
  {
    zone: "free",
    top: 40,
    bottom: 160,
    rowMids: [60, 100, 140],
    rowTops: [50, 90, 130],
  },
  { zone: "aft", top: 160, bottom: 200, rowMids: [180], rowTops: [170] },
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
      { zone: "fore", top: 0, bottom: 20, rowMids: [], rowTops: [] },
      { zone: "free", top: 20, bottom: 100, rowMids: [40, 80], rowTops: [30, 70] },
      { zone: "aft", top: 100, bottom: 120, rowMids: [], rowTops: [] },
    ];
    expect(slotFromPointer(withEmpty, 10)).toEqual({ zone: "fore", index: 0 });
    expect(slotFromPointer(withEmpty, 110)).toEqual({ zone: "aft", index: 0 });
  });
});

describe("slotY", () => {
  it("puts an interior slot at the top edge of the row it inserts before", () => {
    expect(slotY(bands, { zone: "free", index: 0 })).toBe(50);
    expect(slotY(bands, { zone: "free", index: 2 })).toBe(130);
  });

  it("puts the end-of-zone slot at the band bottom", () => {
    expect(slotY(bands, { zone: "free", index: 3 })).toBe(160);
    expect(slotY(bands, { zone: "aft", index: 1 })).toBe(200);
  });

  it("puts the only slot of an empty zone at the band top", () => {
    const withEmpty: ZoneBand[] = [
      { zone: "fore", top: 0, bottom: 20, rowMids: [], rowTops: [] },
      { zone: "free", top: 20, bottom: 100, rowMids: [40], rowTops: [30] },
      { zone: "aft", top: 100, bottom: 120, rowMids: [], rowTops: [] },
    ];
    expect(slotY(withEmpty, { zone: "fore", index: 0 })).toBe(0);
    expect(slotY(withEmpty, { zone: "aft", index: 0 })).toBe(100);
  });
});

describe("blockHeight", () => {
  it("measures a block from its top to the next block's top", () => {
    expect(blockHeight(bands, { zone: "free", index: 1 })).toBe(40);
  });

  it("measures the last block in a zone to the band bottom", () => {
    expect(blockHeight(bands, { zone: "free", index: 2 })).toBe(30);
    expect(blockHeight(bands, { zone: "aft", index: 0 })).toBe(30);
  });
});

describe("rowShifts", () => {
  it("slides rows between the source and a lower target up by the block height", () => {
    const shifts = rowShifts(
      bands,
      { zone: "free", index: 0 },
      { zone: "free", index: 3 }
    );
    expect(shifts.free).toEqual([0, -40, -40]);
    expect(shifts.fore).toEqual([0]);
    expect(shifts.aft).toEqual([0]);
  });

  it("slides rows between a higher target and the source down, across zones", () => {
    const shifts = rowShifts(
      bands,
      { zone: "free", index: 2 },
      { zone: "fore", index: 1 }
    );
    expect(shifts.free).toEqual([30, 30, 0]);
    expect(shifts.fore).toEqual([0]);
    expect(shifts.aft).toEqual([0]);
  });

  it("produces no shifts for the source's own slot or the next slot", () => {
    const same = rowShifts(
      bands,
      { zone: "free", index: 1 },
      { zone: "free", index: 1 }
    );
    expect(same.free).toEqual([0, 0, 0]);
    const next = rowShifts(
      bands,
      { zone: "free", index: 1 },
      { zone: "free", index: 2 }
    );
    expect(next.free).toEqual([0, 0, 0]);
  });
});

describe("settleY", () => {
  it("glides down to a lower target minus the block's own height", () => {
    expect(settleY(bands, { zone: "free", index: 0 }, { zone: "free", index: 3 })).toBe(70);
  });

  it("glides up to a higher target directly", () => {
    expect(settleY(bands, { zone: "free", index: 0 }, { zone: "fore", index: 1 })).toBe(-10);
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
