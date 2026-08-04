import type { Report } from "./types";

export const appState = $state({
  report: null as Report | null,
  configPath: null as string | null,
  modsDir: null as string | null,
  loading: false,
  error: null as string | null,
  selectedPath: null as string | null,
  voice: (localStorage.getItem("manifest-voice") === "plain" ? "plain" : "ship") as "ship" | "plain",
  recentlyDisabled: [] as { name: string; path: string }[],
});

export function setReport(report: Report) {
  appState.report = report;
  appState.error = null;
}

export function setError(message: string) {
  appState.error = message;
}

export function setVoice(voice: "ship" | "plain") {
  appState.voice = voice;
  localStorage.setItem("manifest-voice", voice);
}

const RECENT_LIMIT = 5;

export function rememberDisabled(entries: { name: string; path: string }[]) {
  const merged = [
    ...entries,
    ...appState.recentlyDisabled.filter((e) => !entries.some((n) => n.path === e.path)),
  ];
  appState.recentlyDisabled = merged.slice(0, RECENT_LIMIT);
}

export function forgetDisabled(path: string) {
  appState.recentlyDisabled = appState.recentlyDisabled.filter((e) => e.path !== path);
}
