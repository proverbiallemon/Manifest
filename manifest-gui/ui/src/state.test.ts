import { afterEach, describe, expect, it } from "vitest";
import { appState, rememberDisabled } from "./state.svelte";

afterEach(() => {
  appState.recentlyDisabled = [];
});

describe("rememberDisabled", () => {
  it("does not duplicate an entry with a path already present", () => {
    rememberDisabled([{ name: "ModA", path: "/mods/a.otr" }]);
    rememberDisabled([{ name: "ModA", path: "/mods/a.otr" }]);

    expect(appState.recentlyDisabled).toEqual([{ name: "ModA", path: "/mods/a.otr" }]);
  });

  it("keeps the cap at RECENT_LIMIT (5) even with repeated paths", () => {
    rememberDisabled([
      { name: "Mod1", path: "/mods/1.otr" },
      { name: "Mod2", path: "/mods/2.otr" },
      { name: "Mod3", path: "/mods/3.otr" },
      { name: "Mod4", path: "/mods/4.otr" },
      { name: "Mod5", path: "/mods/5.otr" },
    ]);
    rememberDisabled([{ name: "Mod1", path: "/mods/1.otr" }]);

    expect(appState.recentlyDisabled.length).toBe(5);
    expect(appState.recentlyDisabled[0]).toEqual({ name: "Mod1", path: "/mods/1.otr" });
  });
});
