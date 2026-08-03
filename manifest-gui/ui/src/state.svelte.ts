import type { Report } from "./types";

export const appState = $state({
  report: null as Report | null,
  configPath: null as string | null,
  modsDir: null as string | null,
  loading: false,
  error: null as string | null,
  selectedPath: null as string | null,
  voice: (localStorage.getItem("manifest-voice") === "plain" ? "plain" : "ship") as "ship" | "plain",
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
