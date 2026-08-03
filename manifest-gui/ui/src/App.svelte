<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "./api";
  import { appState, setReport, setError } from "./state.svelte";
  import { conflictCount, sortNeeded } from "./types";

  async function rescan() {
    if (!appState.configPath) return;
    appState.loading = true;
    try {
      setReport(await api.scan(appState.configPath, appState.modsDir));
    } catch (e) {
      setError(String(e));
    } finally {
      appState.loading = false;
    }
  }

  async function firstLoad() {
    const saved = await api.loadSettings();
    if (saved) {
      appState.configPath = saved.config_path;
      appState.modsDir = saved.mods_dir;
    } else {
      appState.configPath = await api.locateConfig();
    }
    if (appState.configPath) await rescan();
  }

  async function chooseConfig() {
    const picked = await api.pickFile();
    if (picked) {
      appState.configPath = picked;
      await rescan();
    }
  }

  async function chooseModsDir() {
    const picked = await api.pickFolder();
    if (picked) {
      appState.modsDir = picked;
      await rescan();
    }
  }

  async function applySort() {
    appState.loading = true;
    try {
      setReport(await api.applySort());
    } catch (e) {
      setError(String(e));
    } finally {
      appState.loading = false;
    }
  }

  async function pin(name: string, position: "top" | "bottom" | null) {
    try {
      setReport(await api.setPin(name, position));
    } catch (e) {
      setError(String(e));
    }
  }

  onMount(firstLoad);
</script>

<main>
  <h1>Manifest</h1>
  <p>
    config: {appState.configPath ?? "not found"}
    <button onclick={chooseConfig}>choose config</button>
    <button onclick={chooseModsDir}>choose mods folder</button>
    <button onclick={rescan} disabled={appState.loading}>rescan</button>
  </p>

  {#if appState.error}
    <p role="alert">error: {appState.error}</p>
  {/if}

  {#if appState.report}
    {@const report = appState.report}
    <p>
      {report.mods.length} mods, {report.conflicts.length} conflicts,
      {report.warnings.length} warnings
    </p>
    {#if sortNeeded(report)}
      <button onclick={applySort} disabled={appState.loading}>
        apply proposed sort ({report.moves.length} moves)
      </button>
    {:else}
      <p>hold is in order</p>
    {/if}
    <ul>
      {#each report.mods as mod}
        <li>
          {mod.name}
          {mod.enabled ? "" : "(not loaded)"}
          {mod.error ? `error: ${mod.error}` : ""}
          conflicts: {conflictCount(report, mod.name)}
          pinned: {mod.pinned ?? "no"}
          <button onclick={() => pin(mod.name, "top")}>pin top</button>
          <button onclick={() => pin(mod.name, "bottom")}>pin bottom</button>
          <button onclick={() => pin(mod.name, null)}>unpin</button>
        </li>
      {/each}
    </ul>
    <h2>warnings</h2>
    <ul>
      {#each report.warnings as warning}
        <li>{JSON.stringify(warning)}</li>
      {/each}
    </ul>
  {/if}
</main>
