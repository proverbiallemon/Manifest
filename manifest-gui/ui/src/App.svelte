<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "./api";
  import { appState, setReport, setError } from "./state.svelte";
  import { sortNeeded } from "./types";
  import Header from "./lib/Header.svelte";
  import WarningCards from "./lib/WarningCards.svelte";
  import Ledger from "./lib/Ledger.svelte";
  import DetailPane from "./lib/DetailPane.svelte";
  import SortModal from "./lib/SortModal.svelte";

  let modalOpen = $state(false);

  async function rescan() {
    if (!appState.configPath || appState.loading) return;
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
    try {
      const saved = await api.loadSettings();
      if (saved) {
        appState.configPath = saved.config_path;
        appState.modsDir = saved.mods_dir;
      } else {
        appState.configPath = await api.locateConfig();
      }
      if (appState.configPath) await rescan();
    } catch (e) {
      setError(String(e));
    }
  }

  async function chooseConfig() {
    const picked = await api.pickFile();
    if (picked) {
      appState.configPath = picked;
      appState.modsDir = null;
      appState.selectedPath = null;
      await rescan();
    }
  }

  async function chooseModsDir() {
    const picked = await api.pickFolder();
    if (picked) {
      appState.modsDir = picked;
      appState.selectedPath = null;
      await rescan();
    }
  }

  async function confirmSort() {
    if (appState.loading) return;
    appState.loading = true;
    try {
      setReport(await api.applySort());
      modalOpen = false;
    } catch (e) {
      setError(String(e));
    } finally {
      appState.loading = false;
    }
  }

  async function pin(name: string, position: "top" | "bottom" | null) {
    if (appState.loading) return;
    appState.loading = true;
    try {
      setReport(await api.setPin(name, position));
    } catch (e) {
      setError(String(e));
    } finally {
      appState.loading = false;
    }
  }

  onMount(firstLoad);
</script>

<div class="frame">
  <Header
    configPath={appState.configPath}
    sortNeeded={appState.report ? sortNeeded(appState.report) : false}
    moveCount={appState.report?.moves.length ?? 0}
    loading={appState.loading}
    onRescan={rescan}
    onRestow={() => (modalOpen = true)}
    onChooseConfig={chooseConfig}
    onChooseMods={chooseModsDir}
  />

  {#if appState.error}
    <section class="damaged" role="alert">
      <div class="heading">DAMAGED MANIFEST</div>
      <p class="fine-print">{appState.error}</p>
      <p class="fine-print faded">nothing was changed</p>
      <p>
        <button onclick={chooseConfig}>locate the manifest</button>
        <button onclick={chooseModsDir}>locate the hold</button>
        <button onclick={() => { appState.error = null; rescan(); }}>try again</button>
      </p>
    </section>
  {:else if appState.report}
    <WarningCards warnings={appState.report.warnings} />
    <main>
      <div class="ledger-scroll">
        <Ledger
          report={appState.report}
          selectedPath={appState.selectedPath}
          onSelect={(p) => (appState.selectedPath = p)}
          onPin={pin}
        />
      </div>
      <DetailPane report={appState.report} selectedPath={appState.selectedPath} />
    </main>
  {:else if appState.loading}
    <p class="centered faded">taking inventory of the hold...</p>
  {:else}
    <section class="centered">
      <p class="faded">no manifest on file</p>
      <p>
        <button onclick={chooseConfig}>locate the manifest</button>
      </p>
    </section>
  {/if}

  {#if modalOpen && appState.report}
    <SortModal
      report={appState.report}
      busy={appState.loading}
      onConfirm={confirmSort}
      onCancel={() => (modalOpen = false)}
    />
  {/if}
</div>

<style>
  .frame {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }
  main {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr minmax(260px, 34%);
    min-height: 0;
  }
  .ledger-scroll {
    overflow-y: auto;
    padding: 0 0 var(--pad) var(--pad);
  }
  .centered {
    text-align: center;
    padding-top: 18vh;
  }
  .damaged {
    text-align: center;
    padding-top: 12vh;
  }
  .damaged .heading {
    letter-spacing: 4px;
    color: var(--red-ink);
  }
</style>
