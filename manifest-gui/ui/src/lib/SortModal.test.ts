import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import SortModal from "./SortModal.svelte";
import { mkReport } from "./testReport";

const report = mkReport({
  current_order: ["Small", "Big"],
  proposed_order: ["Big", "Small"],
  moves: [
    { name: "Big", reason: "broad packs load first" },
    { name: "Small", reason: "specific cargo prevails" },
  ],
});

describe("SortModal", () => {
  it("shows both columns with reasons and fires the callbacks", async () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    render(SortModal, { report, busy: false, onConfirm, onCancel });
    expect(screen.getByText("AS STOWED")).toBeTruthy();
    expect(screen.getByText("AS PROPOSED")).toBeTruthy();
    expect(screen.getByText("specific cargo prevails")).toBeTruthy();
    await fireEvent.click(screen.getByText("Stamp it"));
    expect(onConfirm).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByText("Belay that"));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("disables both actions while busy", () => {
    render(SortModal, { report, busy: true, onConfirm: vi.fn(), onCancel: vi.fn() });
    expect((screen.getByText("Stamp it") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByText("Belay that") as HTMLButtonElement).disabled).toBe(true);
  });
});
