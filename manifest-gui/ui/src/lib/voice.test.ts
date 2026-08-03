import { afterEach, describe, expect, it } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";
import WarningCards from "./WarningCards.svelte";
import { setVoice } from "../state.svelte";

afterEach(() => {
  cleanup();
  setVoice("ship");
});

describe("voice toggle", () => {
  it("renders ship voice by default", () => {
    render(WarningCards, {
      warnings: [{ kind: "total_eclipse", name: "Ghost" }],
    });
    expect(screen.getByText("Ghost is fully covered by later cargo")).toBeTruthy();
  });

  it("switches to plain language and persists the choice", () => {
    setVoice("plain");
    render(WarningCards, {
      warnings: [{ kind: "total_eclipse", name: "Ghost" }],
    });
    expect(screen.getByText("Ghost is fully overridden by later mods")).toBeTruthy();
    expect(localStorage.getItem("manifest-voice")).toBe("plain");
  });
});
