import { invoke } from "@tauri-apps/api/core";
import type { Report, StoredSettings, ModToggle } from "./types";

export function locateConfig(): Promise<string | null> {
  return invoke("locate_config");
}

export function loadSettings(): Promise<StoredSettings | null> {
  return invoke("load_settings");
}

export function scan(configPath: string, modsDir: string | null): Promise<Report> {
  return invoke("scan", { configPath, modsDir });
}

export function applySort(): Promise<Report> {
  return invoke("apply_sort");
}

export function reorder(
  fore: string[],
  free: string[],
  aft: string[]
): Promise<Report> {
  return invoke("reorder", { fore, free, aft });
}

export function setModsEnabled(changes: ModToggle[]): Promise<Report> {
  return invoke("set_mods_enabled", { changes });
}

export function revealItem(path: string): Promise<void> {
  return invoke("reveal_item", { path });
}

export function pickFile(): Promise<string | null> {
  return invoke("pick_file");
}

export function pickFolder(): Promise<string | null> {
  return invoke("pick_folder");
}
