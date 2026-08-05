import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import Ledger from "./Ledger.svelte";
import { mkMod, mkReport } from "./testReport";
import { deriveZones } from "../types";

vi.mock("./dragPlan", async (importOriginal) => {
  const real = await importOriginal<typeof import("./dragPlan")>();
  return { ...real, slotFromPointer: () => ({ zone: "aft", index: 0 }) };
});

describe("Ledger", () => {
  it("renders duplicate-named files as separate rows with their own paths", () => {
    const report = mkReport({
      mods: [
        mkMod({ name: "Vanilla Equipment", path: "/mods/a/Vanilla Equipment.otr" }),
        mkMod({ name: "Vanilla Equipment", path: "/mods/b/Vanilla Equipment.otr" }),
      ],
      current_order: ["Vanilla Equipment"],
    });
    render(Ledger, {
      report,
      selectedPath: null,
      onSelect: vi.fn(),
      onPin: vi.fn(),
      onReorder: vi.fn(),
    });
    expect(screen.getAllByText("Vanilla Equipment")).toHaveLength(2);
  });

  it("puts disabled and unordered mods in the NOT LOADED block", () => {
    const report = mkReport({
      mods: [
        mkMod({ name: "Loaded" }),
        mkMod({ name: "Parked", enabled: false }),
        mkMod({ name: "Stray" }),
      ],
      current_order: ["Loaded"],
    });
    render(Ledger, {
      report,
      selectedPath: null,
      onSelect: vi.fn(),
      onPin: vi.fn(),
      onReorder: vi.fn(),
    });
    expect(screen.getByText("NOT LOADED")).toBeTruthy();
    expect(screen.getAllByRole("row")).toHaveLength(3);
  });

  it("keeps each mod's row DOM node when the order changes", async () => {
    const mods = [mkMod({ name: "Small" }), mkMod({ name: "Big" })];
    const { rerender } = render(Ledger, {
      report: mkReport({ mods, current_order: ["Small", "Big"] }),
      selectedPath: null,
      onSelect: vi.fn(),
      onPin: vi.fn(),
      onReorder: vi.fn(),
    });
    const bigRowBefore = screen.getByText("Big").closest('[role="row"]');
    expect(bigRowBefore).toBeTruthy();
    await rerender({
      report: mkReport({ mods, current_order: ["Big", "Small"] }),
    });
    const bigRowAfter = screen.getByText("Big").closest('[role="row"]');
    expect(bigRowAfter).toBe(bigRowBefore);
  });
});

describe("three-zone ledger", () => {
  const zonedReport = mkReport({
    mods: [
      mkMod({ name: "Broad", pinned: "top" }),
      mkMod({ name: "Mid" }),
      mkMod({ name: "Fine", pinned: "bottom" }),
    ],
    current_order: ["Broad", "Mid", "Fine"],
  });

  it("renders fore and aft zone headings when those zones hold cargo", () => {
    render(Ledger, {
      props: {
        report: zonedReport,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder: () => {},
      },
    });
    expect(screen.getByText("LASHED FORE, LOADS FIRST")).toBeTruthy();
    expect(screen.getByText("LASHED AFT, PREVAILS")).toBeTruthy();
  });

  it("hides zone headings when no mods are pinned", () => {
    const flat = mkReport({
      mods: [mkMod({ name: "Only" })],
      current_order: ["Only"],
    });
    render(Ledger, {
      props: {
        report: flat,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder: () => {},
      },
    });
    expect(screen.queryByText("LASHED FORE, LOADS FIRST")).toBeNull();
    expect(screen.queryByText("LASHED AFT, PREVAILS")).toBeNull();
  });

  it("numbers lines continuously across zones in load order", () => {
    render(Ledger, {
      props: {
        report: zonedReport,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder: () => {},
      },
    });
    const rows = screen.getAllByRole("row");
    const texts = rows.map((r) => r.textContent ?? "");
    expect(texts[0]).toContain("Broad");
    expect(texts[0]).toContain("1");
    expect(texts[2]).toContain("Fine");
    expect(texts[2]).toContain("3");
  });
});

describe("drag reordering", () => {
  const report = mkReport({
    mods: [
      mkMod({ name: "A" }),
      mkMod({ name: "B" }),
      mkMod({ name: "Fine", pinned: "bottom" }),
    ],
    current_order: ["A", "B", "Fine"],
  });

  it("emits the post-drag zones through onReorder on drop", async () => {
    const onReorder = vi.fn();
    render(Ledger, {
      props: {
        report,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    const grips = screen.getAllByTitle("drag to reorder");
    await fireEvent.pointerDown(grips[0], { clientY: 10 });
    window.dispatchEvent(new MouseEvent("pointermove", { clientY: 200 }));
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 200 }));
    expect(onReorder).toHaveBeenCalledWith([], ["B"], ["A", "Fine"]);
  });

  it("lifts the grabbed row while dragging and settles it on drop", async () => {
    const onReorder = vi.fn();
    render(Ledger, {
      props: {
        report,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    const grips = screen.getAllByTitle("drag to reorder");
    await fireEvent.pointerDown(grips[1], { clientY: 10 });
    const rows = screen.getAllByRole("row");
    const bRow = rows.find((r) => r.textContent?.includes("B"))!;
    expect(bRow.classList.contains("lifted")).toBe(true);
    const others = rows.filter((r) => r !== bRow);
    expect(others.some((r) => r.classList.contains("lifted"))).toBe(false);
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 10 }));
    await Promise.resolve();
    expect(
      screen.getAllByRole("row").some((r) => r.classList.contains("lifted"))
    ).toBe(false);
  });

  it("a drop on the source slot does not call onReorder", async () => {
    const onReorder = vi.fn();
    render(Ledger, {
      props: {
        report: mkReport({
          mods: [mkMod({ name: "Solo", pinned: "bottom" })],
          current_order: ["Solo"],
        }),
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    const grip = screen.getByTitle("drag to reorder");
    await fireEvent.pointerDown(grip, { clientY: 10 });
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 10 }));
    expect(onReorder).not.toHaveBeenCalled();
  });

  it("dragging the second row of a duplicate-named pair moves that name, not the next one", async () => {
    const onReorder = vi.fn();
    const dupReport = mkReport({
      mods: [
        mkMod({ name: "Twin", path: "/mods/a/Twin.otr" }),
        mkMod({ name: "Twin", path: "/mods/b/Twin.otr" }),
        mkMod({ name: "Other" }),
      ],
      current_order: ["Twin", "Other"],
    });
    render(Ledger, {
      props: {
        report: dupReport,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    // slotFromPointer is mocked to always land at { zone: "aft", index: 0 }.
    const grips = screen.getAllByTitle("drag to reorder");
    await fireEvent.pointerDown(grips[1], { clientY: 10 });
    window.dispatchEvent(new MouseEvent("pointermove", { clientY: 200 }));
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 200 }));
    expect(onReorder).toHaveBeenCalledWith([], ["Other"], ["Twin"]);
  });

  it("dragging the row after a duplicate-named pair moves the correct name", async () => {
    const onReorder = vi.fn();
    const dupReport = mkReport({
      mods: [
        mkMod({ name: "Twin", path: "/mods/a/Twin.otr" }),
        mkMod({ name: "Twin", path: "/mods/b/Twin.otr" }),
        mkMod({ name: "Other" }),
      ],
      current_order: ["Twin", "Other"],
    });
    render(Ledger, {
      props: {
        report: dupReport,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    const grips = screen.getAllByTitle("drag to reorder");
    await fireEvent.pointerDown(grips[2], { clientY: 10 });
    window.dispatchEvent(new MouseEvent("pointermove", { clientY: 200 }));
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 200 }));
    expect(onReorder).toHaveBeenCalledWith([], ["Twin"], ["Other"]);
  });
});

describe("drag guards", () => {
  const report = mkReport({
    mods: [mkMod({ name: "A" }), mkMod({ name: "Fine", pinned: "bottom" })],
    current_order: ["A", "Fine"],
  });

  it("refuses to start a drag while busy", async () => {
    const onReorder = vi.fn();
    render(Ledger, {
      props: {
        report,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
        busy: true,
      },
    });
    const grip = screen.getAllByTitle("drag to reorder")[0];
    await fireEvent.pointerDown(grip, { clientY: 10 });
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 200 }));
    expect(onReorder).not.toHaveBeenCalled();
  });

  it("cancels an active drag when the report prop changes before drop", async () => {
    const onReorder = vi.fn();
    const { rerender } = render(Ledger, {
      props: {
        report,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    const grip = screen.getAllByTitle("drag to reorder")[0];
    await fireEvent.pointerDown(grip, { clientY: 10 });
    const nextReport = mkReport({
      mods: [mkMod({ name: "A" }), mkMod({ name: "Fine", pinned: "bottom" })],
      current_order: ["A", "Fine"],
    });
    await rerender({ report: nextReport });
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 200 }));
    expect(onReorder).not.toHaveBeenCalled();
  });

  it("aborts the drag on pointercancel without calling onReorder", async () => {
    const onReorder = vi.fn();
    render(Ledger, {
      props: {
        report,
        selectedPath: null,
        onSelect: () => {},
        onPin: () => {},
        onReorder,
      },
    });
    const grip = screen.getAllByTitle("drag to reorder")[0];
    await fireEvent.pointerDown(grip, { clientY: 10 });
    window.dispatchEvent(new Event("pointercancel"));
    window.dispatchEvent(new MouseEvent("pointerup", { clientY: 200 }));
    expect(onReorder).not.toHaveBeenCalled();
  });
});
