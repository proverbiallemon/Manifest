import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import Header from "./Header.svelte";

describe("Header", () => {
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
