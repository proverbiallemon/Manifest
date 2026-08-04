import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import WarningModal from "./WarningModal.svelte";
import { mkMod, mkReport } from "./testReport";
import type { Warning } from "../types";

const dupWarning: Warning = {
  kind: "duplicate_gamebanana_mod",
  mod_id: 4321,
  names: ["Vanilla Equipment", "Vanilla Equipment"],
};

const dupReport = mkReport({
  mods: [
    mkMod({ name: "Vanilla Equipment", path: "/mods/a/Vanilla Equipment.otr" }),
    mkMod({ name: "Vanilla Equipment", path: "/mods/b/Vanilla Equipment.otr" }),
  ],
  current_order: ["Vanilla Equipment"],
});

describe("WarningModal", () => {
  it("keeps the stamp disabled for duplicates until a keeper is chosen, then stages the rest ashore", async () => {
    const onStamp = vi.fn();
    render(WarningModal, {
      warning: dupWarning, report: dupReport, busy: false,
      onStamp, onCancel: vi.fn(), onReveal: vi.fn(),
    });
    const stamp = screen.getByText("Stamp it") as HTMLButtonElement;
    expect(stamp.disabled).toBe(true);
    await fireEvent.click(screen.getAllByText("KEEP")[0]);
    expect(stamp.disabled).toBe(false);
    await fireEvent.click(stamp);
    expect(onStamp).toHaveBeenCalledWith([
      { path: "/mods/b/Vanilla Equipment.otr", enabled: false },
    ]);
  });

  it("pre-stamps the current winner for mutual overlap", async () => {
    const warning: Warning = { kind: "mutual_overlap", names: ["A", "B"] };
    const report = mkReport({
      mods: [mkMod({ name: "A" }), mkMod({ name: "B" })],
      current_order: ["A", "B"],
    });
    const onStamp = vi.fn();
    render(WarningModal, {
      warning, report, busy: false,
      onStamp, onCancel: vi.fn(), onReveal: vi.fn(),
    });
    // B is last in load order, so B is the pre-stamped keeper and A is staged ashore
    await fireEvent.click(screen.getByText("Stamp it"));
    expect(onStamp).toHaveBeenCalledWith([{ path: "/mods/A.otr", enabled: false }]);
  });

  it("offers only set-ashore for a total eclipse and stages every file of that name", async () => {
    const warning: Warning = { kind: "total_eclipse", name: "Dead" };
    const report = mkReport({
      mods: [mkMod({ name: "Dead" }), mkMod({ name: "Alive" })],
      current_order: ["Dead", "Alive"],
    });
    const onStamp = vi.fn();
    render(WarningModal, {
      warning, report, busy: false,
      onStamp, onCancel: vi.fn(), onReveal: vi.fn(),
    });
    expect(screen.queryByText("KEEP")).toBeNull();
    await fireEvent.click(screen.getByText("Stamp it"));
    expect(onStamp).toHaveBeenCalledWith([{ path: "/mods/Dead.otr", enabled: false }]);
  });

  it("offers reveal for unlistable files", async () => {
    const warning: Warning = { kind: "unlistable", name: "Broken", reason: "bad header" };
    const report = mkReport({
      mods: [mkMod({ name: "Broken", error: "bad header" })],
      current_order: [],
    });
    const onReveal = vi.fn();
    render(WarningModal, {
      warning, report, busy: false,
      onStamp: vi.fn(), onCancel: vi.fn(), onReveal,
    });
    await fireEvent.click(screen.getByText("open its berth"));
    expect(onReveal).toHaveBeenCalledWith("/mods/Broken.otr");
  });
});
