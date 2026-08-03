import { invoke } from "@tauri-apps/api/core";
import type { Report, StoredSettings } from "./types";

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

export function setPin(
  modName: string,
  position: "top" | "bottom" | null
): Promise<Report> {
  return invoke("set_pin", { modName, position });
}

export function pickFile(): Promise<string | null> {
  return invoke("pick_file");
}

export function pickFolder(): Promise<string | null> {
  return invoke("pick_folder");
}
