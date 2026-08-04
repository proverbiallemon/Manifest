import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import WarningCards from "./WarningCards.svelte";

describe("WarningCards", () => {
  it("renders one slip per warning in manifest voice", () => {
    render(WarningCards, {
      warnings: [
        { kind: "total_eclipse", name: "Ghost" },
        { kind: "mutual_overlap", names: ["A", "B"] },
        { kind: "unlistable", name: "Broken", reason: "no (listfile)" },
        { kind: "duplicate_gamebanana_mod", mod_id: 7, names: ["X", "Y"] },
      ],
      onAct: vi.fn(),
    });
    expect(screen.getByText("Ghost is fully covered by later cargo")).toBeTruthy();
    expect(screen.getByText("A, B carry identical cargo")).toBeTruthy();
    expect(screen.getByText("no (listfile)")).toBeTruthy();
    expect(
      screen.getByText("X, Y are the same shipment twice (GameBanana 7)")
    ).toBeTruthy();
  });

  it("renders nothing when there are no warnings", () => {
    const { container } = render(WarningCards, { warnings: [], onAct: vi.fn() });
    expect(container.querySelector("section")).toBeNull();
  });
});
