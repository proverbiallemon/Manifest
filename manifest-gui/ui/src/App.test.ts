import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import App from "./App.svelte";
import { appState } from "./state.svelte";
import { mkMod, mkReport } from "./lib/testReport";

vi.mock("./api", () => ({
  locateConfig: vi.fn(),
  loadSettings: vi.fn(),
  scan: vi.fn(),
  applySort: vi.fn(),
  setPin: vi.fn(),
  pickFile: vi.fn(),
  pickFolder: vi.fn(),
  setModsEnabled: vi.fn(),
  revealItem: vi.fn(),
}));

import * as api from "./api";

const mocked = vi.mocked(api);

const unsortedReport = mkReport({
  mods: [mkMod({ name: "Small" }), mkMod({ name: "Big" })],
  current_order: ["Small", "Big"],
  proposed_order: ["Big", "Small"],
  moves: [{ name: "Big", reason: "broad packs load first" }],
});

beforeEach(() => {
  vi.resetAllMocks();
  appState.report = null;
  appState.configPath = null;
  appState.modsDir = null;
  appState.loading = false;
  appState.error = null;
  appState.selectedPath = null;
  mocked.loadSettings.mockResolvedValue({
    config_path: "/ship/shipofharkinian.json",
    mods_dir: "/ship/mods",
  });
});

describe("App", () => {
  it("closes the sort modal and shows the error page when applying fails", async () => {
    mocked.scan.mockResolvedValue(unsortedReport);
    mocked.applySort.mockRejectedValue("the hold is flooded");
    render(App);
    await fireEvent.click(await screen.findByText("Re-stow the hold (1)"));
    expect(await screen.findByText("RE-STOW THE HOLD")).toBeTruthy();
    await fireEvent.click(screen.getByText("Stamp it"));
    expect(await screen.findByText("DAMAGED MANIFEST")).toBeTruthy();
    expect(screen.getByText("the hold is flooded")).toBeTruthy();
    expect(screen.queryByText("RE-STOW THE HOLD")).toBeNull();
    expect(screen.queryByText("Stamp it")).toBeNull();
  });

  it("hides the Re-stow button and order note while the damaged page is shown", async () => {
    mocked.scan.mockResolvedValueOnce(unsortedReport);
    mocked.scan.mockRejectedValueOnce("the manifest is waterlogged");
    render(App);
    await screen.findByText("Re-stow the hold (1)");
    await fireEvent.click(screen.getByText("take inventory"));
    expect(await screen.findByText("DAMAGED MANIFEST")).toBeTruthy();
    expect(screen.queryByText(/Re-stow the hold/)).toBeNull();
    expect(screen.queryByText("hold is in order")).toBeNull();
  });

  it("hides the in-order note while the damaged page is shown", async () => {
    const sortedReport = mkReport({
      mods: [mkMod({ name: "Small" })],
      current_order: ["Small"],
      proposed_order: ["Small"],
    });
    mocked.scan.mockResolvedValueOnce(sortedReport);
    mocked.scan.mockRejectedValueOnce("the manifest is waterlogged");
    render(App);
    await screen.findByText("hold is in order");
    await fireEvent.click(screen.getByText("take inventory"));
    expect(await screen.findByText("DAMAGED MANIFEST")).toBeTruthy();
    expect(screen.queryByText("hold is in order")).toBeNull();
  });

  it("shows the error page when the config picker fails", async () => {
    mocked.scan.mockResolvedValue(unsortedReport);
    mocked.pickFile.mockRejectedValue("dialog plumbing burst");
    render(App);
    await screen.findByText("Re-stow the hold (1)");
    await fireEvent.click(screen.getByText("config"));
    expect(await screen.findByText("DAMAGED MANIFEST")).toBeTruthy();
    expect(screen.getByText("dialog plumbing burst")).toBeTruthy();
  });

  it("shows the error page when the mods folder picker fails", async () => {
    mocked.scan.mockResolvedValue(unsortedReport);
    mocked.pickFolder.mockRejectedValue("no folder for you");
    render(App);
    await screen.findByText("Re-stow the hold (1)");
    await fireEvent.click(screen.getByText("hold"));
    expect(await screen.findByText("DAMAGED MANIFEST")).toBeTruthy();
    expect(screen.getByText("no folder for you")).toBeTruthy();
  });

  it("opens the warning modal from a card and refreshes after stamping", async () => {
    const warnReport = mkReport({
      mods: [
        mkMod({ name: "Vanilla Equipment", path: "/mods/a/Vanilla Equipment.otr" }),
        mkMod({ name: "Vanilla Equipment", path: "/mods/b/Vanilla Equipment.otr" }),
      ],
      current_order: ["Vanilla Equipment"],
      proposed_order: ["Vanilla Equipment"],
      warnings: [
        { kind: "duplicate_gamebanana_mod", mod_id: 4321, names: ["Vanilla Equipment", "Vanilla Equipment"] },
      ],
    });
    mocked.scan.mockResolvedValue(warnReport);
    mocked.setModsEnabled.mockResolvedValue(
      mkReport({
        mods: [
          mkMod({ name: "Vanilla Equipment", path: "/mods/a/Vanilla Equipment.otr" }),
          mkMod({ name: "Vanilla Equipment", path: "/mods/b/Vanilla Equipment.disabled", enabled: false }),
        ],
        current_order: ["Vanilla Equipment"],
        proposed_order: ["Vanilla Equipment"],
      })
    );
    render(App);
    await fireEvent.click(await screen.findByText(/are the same shipment twice/));
    expect(await screen.findByText("CONTESTED CARGO")).toBeTruthy();
    await fireEvent.click(screen.getAllByText("KEEP")[0]);
    await fireEvent.click(screen.getByText("Stamp it"));
    expect(mocked.setModsEnabled).toHaveBeenCalledWith([
      { path: "/mods/b/Vanilla Equipment.otr", enabled: false },
    ]);
    expect(screen.queryByText("CONTESTED CARGO")).toBeNull();
    // the ashore button reflects the disabled mod from the fresh report
    const ashoreButton = await screen.findByText("set ashore (1)");
    await fireEvent.click(ashoreButton);
    expect(await screen.findByText("SET ASHORE")).toBeTruthy();
    expect(screen.getByText("/mods/b/Vanilla Equipment.disabled")).toBeTruthy();
    await fireEvent.click(screen.getByText("haul back aboard"));
    expect(mocked.setModsEnabled).toHaveBeenLastCalledWith([
      { path: "/mods/b/Vanilla Equipment.disabled", enabled: true },
    ]);
  });

  it("shows the damaged page when stamping fails", async () => {
    const warnReport = mkReport({
      mods: [mkMod({ name: "Dead" })],
      current_order: ["Dead"],
      proposed_order: ["Dead"],
      warnings: [{ kind: "total_eclipse", name: "Dead" }],
    });
    mocked.scan.mockResolvedValue(warnReport);
    mocked.setModsEnabled.mockRejectedValue("the crane jammed");
    render(App);
    await fireEvent.click(await screen.findByText(/fully covered by later cargo/));
    await fireEvent.click(await screen.findByText("Stamp it"));
    expect(await screen.findByText("DAMAGED MANIFEST")).toBeTruthy();
    expect(screen.queryByText("CONTESTED CARGO")).toBeNull();
  });
});
