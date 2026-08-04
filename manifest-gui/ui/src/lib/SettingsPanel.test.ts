import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import SettingsPanel from "./SettingsPanel.svelte";

describe("SettingsPanel", () => {
  it("closes when clicking outside the panel", async () => {
    const onClose = vi.fn();
    render(SettingsPanel, { open: true, onClose });
    await fireEvent.click(document.body);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("stays open when clicking inside the panel", async () => {
    const onClose = vi.fn();
    render(SettingsPanel, { open: true, onClose });
    await fireEvent.click(screen.getByText(/plain language/));
    expect(onClose).not.toHaveBeenCalled();
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    render(SettingsPanel, { open: true, onClose });
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });
});
