import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import SettingsPanel from "./SettingsPanel.svelte";
import { appState } from "../state.svelte";

describe("SettingsPanel", () => {
  beforeEach(() => {
    appState.voice = "ship";
  });

  it("closes when clicking outside the panel", async () => {
    const onClose = vi.fn();
    render(SettingsPanel, { open: true, onClose, onCheckUpdates: vi.fn().mockResolvedValue("current") });
    await fireEvent.click(document.body);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("stays open when clicking inside the panel", async () => {
    const onClose = vi.fn();
    render(SettingsPanel, { open: true, onClose, onCheckUpdates: vi.fn().mockResolvedValue("current") });
    await fireEvent.click(screen.getByText(/plain language/));
    expect(onClose).not.toHaveBeenCalled();
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    render(SettingsPanel, { open: true, onClose, onCheckUpdates: vi.fn().mockResolvedValue("current") });
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("reports hold is current after a quiet manual check", async () => {
    const onCheckUpdates = vi.fn().mockResolvedValue("current");
    render(SettingsPanel, { open: true, onClose: vi.fn(), onCheckUpdates });
    await fireEvent.click(screen.getByText("check for updates"));
    expect(onCheckUpdates).toHaveBeenCalledOnce();
    expect(await screen.findByText("hold is current")).toBeTruthy();
  });

  it("clears the check note when the panel reopens", async () => {
    const onCheckUpdates = vi.fn().mockResolvedValue("current");
    const { rerender } = render(SettingsPanel, { open: true, onClose: vi.fn(), onCheckUpdates });
    await fireEvent.click(screen.getByText("check for updates"));
    expect(await screen.findByText("hold is current")).toBeTruthy();
    await rerender({ open: false });
    await rerender({ open: true });
    expect(screen.queryByText("hold is current")).toBeNull();
  });
});
