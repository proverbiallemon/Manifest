import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import Ledger from "./Ledger.svelte";
import { mkMod, mkReport } from "./testReport";
import { deriveZones } from "../types";

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
      props: { report: flat, selectedPath: null, onSelect: () => {}, onPin: () => {} },
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
