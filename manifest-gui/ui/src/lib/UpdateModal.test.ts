import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import UpdateModal from "./UpdateModal.svelte";

describe("UpdateModal", () => {
  it("shows the version and fires install and later", async () => {
    const onInstall = vi.fn();
    const onLater = vi.fn();
    render(UpdateModal, { version: "0.3.0", busy: false, onInstall, onLater });
    expect(screen.getByText("NEW SHIPMENT DOCKED")).toBeTruthy();
    expect(screen.getByText(/0\.3\.0/)).toBeTruthy();
    await fireEvent.click(screen.getByText("Bring it aboard"));
    expect(onInstall).toHaveBeenCalledOnce();
    await fireEvent.click(screen.getByText("Not now"));
    expect(onLater).toHaveBeenCalledOnce();
  });

  it("disables both actions and shows the busy line while installing", () => {
    render(UpdateModal, { version: "0.3.0", busy: true, onInstall: vi.fn(), onLater: vi.fn() });
    expect((screen.getByText("Bring it aboard") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByText("Not now") as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText("hauling the new shipment aboard...")).toBeTruthy();
  });
});
