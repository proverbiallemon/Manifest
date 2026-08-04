import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import LedgerRow from "./LedgerRow.svelte";
import { mkMod } from "./testReport";

describe("LedgerRow", () => {
  it("stamps CONFLICT when the mod has conflicts", () => {
    render(LedgerRow, {
      mod: mkMod({ name: "Big" }),
      line: 1,
      conflicts: 3,
      onSelect: vi.fn(),
      onPin: vi.fn(),
    });
    expect(screen.getByText("CONFLICT")).toBeTruthy();
  });

  it("shows clear for a conflict-free mod and the seal when pinned", () => {
    render(LedgerRow, {
      mod: mkMod({ name: "Solo", pinned: "top" }),
      line: 2,
      conflicts: 0,
      onSelect: vi.fn(),
      onPin: vi.fn(),
    });
    expect(screen.getByText("clear")).toBeTruthy();
    expect(screen.getByAltText("pinned top")).toBeTruthy();
  });

  it("shows the error glyph for an unlistable mod", () => {
    render(LedgerRow, {
      mod: mkMod({ name: "Broken", error: "no (listfile)" }),
      line: null,
      conflicts: 0,
      onSelect: vi.fn(),
      onPin: vi.fn(),
    });
    expect(screen.getByAltText("unlistable")).toBeTruthy();
  });
});
