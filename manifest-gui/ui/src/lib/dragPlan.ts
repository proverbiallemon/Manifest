export type ZoneName = "fore" | "free" | "aft";

export interface DropSlot {
  zone: ZoneName;
  index: number;
}

export interface ZoneBand {
  zone: ZoneName;
  top: number;
  bottom: number;
  rowMids: number[];
  rowTops: number[];
}

// The slot index is how many row midpoints in the band sit above y, so a
// drop between rows 1 and 2 yields index 2 (insert before the third row).
export function slotFromPointer(bands: ZoneBand[], y: number): DropSlot {
  let band = bands.find((b) => y >= b.top && y < b.bottom);
  if (!band) {
    band = bands.reduce((best, b) => {
      const d = y < b.top ? b.top - y : y - b.bottom;
      const bd = y < best.top ? best.top - y : y - best.bottom;
      return d < bd ? b : best;
    });
  }
  return { zone: band.zone, index: band.rowMids.filter((m) => y > m).length };
}

// Where the drop indicator sits for a slot: the top edge of the row the
// drop would insert before, or the band's own edge for empty and end slots.
export function slotY(bands: ZoneBand[], slot: DropSlot): number {
  const band = bands.find((b) => b.zone === slot.zone);
  if (!band) return 0;
  if (band.rowTops.length === 0) return band.top;
  return slot.index < band.rowTops.length
    ? band.rowTops[slot.index]
    : band.bottom;
}

type ZoneNames = { fore: string[]; free: string[]; aft: string[] };

// Slot indexes refer to the lists as rendered, with the dragged row still
// present; the insertion index shifts down by one when the row is removed
// from above the slot in the same zone.
export function applyDrag(
  names: ZoneNames,
  source: DropSlot,
  slot: DropSlot
): ZoneNames {
  const out: ZoneNames = {
    fore: [...names.fore],
    free: [...names.free],
    aft: [...names.aft],
  };
  const [moved] = out[source.zone].splice(source.index, 1);
  if (moved === undefined) return out;
  let at = slot.index;
  if (slot.zone === source.zone && slot.index > source.index) at -= 1;
  out[slot.zone].splice(at, 0, moved);
  return out;
}
