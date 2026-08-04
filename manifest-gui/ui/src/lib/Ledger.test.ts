import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import Ledger from "./Ledger.svelte";
import { mkMod, mkReport } from "./testReport";

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
});
