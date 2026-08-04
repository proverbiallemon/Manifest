import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import DetailPane from "./DetailPane.svelte";
import { mkMod, mkReport } from "./testReport";

describe("DetailPane", () => {
  it("offers haul back aboard for a disabled mod", async () => {
    const report = mkReport({
      mods: [mkMod({ name: "Parked", path: "/mods/Parked.disabled", enabled: false })],
    });
    const onEnable = vi.fn();
    render(DetailPane, { report, selectedPath: "/mods/Parked.disabled", onEnable });
    await fireEvent.click(screen.getByText("haul back aboard"));
    expect(onEnable).toHaveBeenCalledWith("/mods/Parked.disabled");
  });

  it("does not offer it for an enabled mod", () => {
    const report = mkReport({ mods: [mkMod({ name: "Live" })], current_order: ["Live"] });
    render(DetailPane, { report, selectedPath: "/mods/Live.otr", onEnable: vi.fn() });
    expect(screen.queryByText("haul back aboard")).toBeNull();
  });
});
