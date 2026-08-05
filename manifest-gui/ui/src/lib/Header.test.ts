import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import Header from "./Header.svelte";

describe("Header", () => {
  const base = {
    configPath: "/ship/shipofharkinian.json",
    sortNeeded: false,
    moveCount: 0,
    loading: false,
    damaged: false,
    onRescan: vi.fn(),
    onRestow: vi.fn(),
    onChooseConfig: vi.fn(),
    onChooseMods: vi.fn(),
    onCheckUpdates: vi.fn().mockResolvedValue("current"),
  };

  it("notes when pins are holding the order, with a plain tooltip", () => {
    render(Header, { ...base, sortHeld: true });
    const note = screen.getByText("hold is in order, held by its lashings");
    expect(note).toBeTruthy();
    expect(note.getAttribute("title")).toBe(
      "the sorter would move pinned mods; pins keep them where you put them"
    );
    expect(screen.queryByText("hold is in order")).toBeNull();
  });

  it("keeps the plain in-order note when nothing is held", () => {
    render(Header, { ...base, sortHeld: false });
    expect(screen.getByText("hold is in order")).toBeTruthy();
    expect(
      screen.queryByText("hold is in order, held by its lashings")
    ).toBeNull();
  });

  it("never shows the held note while a sort is proposed", () => {
    render(Header, { ...base, sortNeeded: true, moveCount: 3, sortHeld: true });
    expect(screen.getByText(/Re-stow the hold/)).toBeTruthy();
    expect(
      screen.queryByText("hold is in order, held by its lashings")
    ).toBeNull();
  });

  it("opens the settings panel on toggle click without the same click closing it", async () => {
    render(Header, {
      configPath: "/ship/shipofharkinian.json",
      sortNeeded: false,
      moveCount: 0,
      loading: false,
      damaged: false,
      onRescan: vi.fn(),
      onRestow: vi.fn(),
      onChooseConfig: vi.fn(),
      onChooseMods: vi.fn(),
      onCheckUpdates: vi.fn().mockResolvedValue("current"),
    });
    await fireEvent.click(screen.getByText("settings"));
    expect(screen.getByRole("dialog", { name: "settings" })).toBeTruthy();
  });
});
