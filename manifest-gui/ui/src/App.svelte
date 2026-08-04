<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "./api";
  import { appState, setReport, setError } from "./state.svelte";
  import { t } from "./copy.svelte";
  import { sortNeeded } from "./types";
  import type { ModToggle, Warning } from "./types";
  import Header from "./lib/Header.svelte";
  import WarningCards from "./lib/WarningCards.svelte";
  import Ledger from "./lib/Ledger.svelte";
  import DetailPane from "./lib/DetailPane.svelte";
  import SortModal from "./lib/SortModal.svelte";
  import WarningModal from "./lib/WarningModal.svelte";
  import ParkedModal from "./lib/ParkedModal.svelte";

  let modalOpen = $state(false);
  let activeWarning = $state<Warning | null>(null);
  let parkedOpen = $state(false);

  const disabledMods = $derived(
    appState.report?.mods.filter((m) => !m.enabled) ?? []
  );

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
    try {
      const picked = await api.pickFile();
      if (picked) {
        appState.configPath = picked;
        appState.modsDir = null;
        appState.selectedPath = null;
        await rescan();
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function chooseModsDir() {
    try {
      const picked = await api.pickFolder();
      if (picked) {
        appState.modsDir = picked;
        appState.selectedPath = null;
        await rescan();
      }
    } catch (e) {
      setError(String(e));
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
      modalOpen = false;
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

  async function stampWarning(changes: ModToggle[]) {
    if (appState.loading) return;
    appState.loading = true;
    try {
      setReport(await api.setModsEnabled(changes));
      activeWarning = null;
    } catch (e) {
      setError(String(e));
      activeWarning = null;
    } finally {
      appState.loading = false;
    }
  }

  async function reveal(path: string) {
    try {
      await api.revealItem(path);
    } catch (e) {
      setError(String(e));
      activeWarning = null;
    }
  }

  async function haulBack(path: string) {
    if (appState.loading) return;
    appState.loading = true;
    try {
      setReport(await api.setModsEnabled([{ path, enabled: true }]));
      if (!appState.report?.mods.some((m) => !m.enabled)) parkedOpen = false;
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
    damaged={appState.error !== null}
    onRescan={rescan}
    onRestow={() => (modalOpen = true)}
    onChooseConfig={chooseConfig}
    onChooseMods={chooseModsDir}
  />

  {#if appState.error}
    <section class="damaged" role="alert">
      <div class="heading">{t("damagedTitle")}</div>
      <p class="fine-print">{appState.error}</p>
      <p class="fine-print faded">{t("nothingChanged")}</p>
      <p>
        <button onclick={chooseConfig}>{t("locateConfig")}</button>
        <button onclick={chooseModsDir}>{t("locateHold")}</button>
        <button onclick={() => { appState.error = null; rescan(); }}>{t("tryAgain")}</button>
      </p>
    </section>
  {:else if appState.report}
    <WarningCards warnings={appState.report.warnings} onAct={(w) => (activeWarning = w)} />
    {#if disabledMods.length > 0}
      <div class="ashore-strip fine-print">
        <button onclick={() => (parkedOpen = true)}>
          {t("ashoreList")} ({disabledMods.length})
        </button>
      </div>
    {/if}
    <main>
      <div class="ledger-scroll">
        <Ledger
          report={appState.report}
          selectedPath={appState.selectedPath}
          onSelect={(p) => (appState.selectedPath = p)}
          onPin={pin}
        />
      </div>
      <DetailPane report={appState.report} selectedPath={appState.selectedPath} onEnable={haulBack} />
    </main>
  {:else if appState.loading}
    <p class="centered faded">{t("loading")}</p>
  {:else}
    <section class="centered">
      <p class="faded">{t("noConfig")}</p>
      <p>
        <button onclick={chooseConfig}>{t("locateConfig")}</button>
      </p>
    </section>
  {/if}

  {#if modalOpen && appState.report && !appState.error}
    <SortModal
      report={appState.report}
      busy={appState.loading}
      onConfirm={confirmSort}
      onCancel={() => (modalOpen = false)}
    />
  {/if}

  {#if parkedOpen && !appState.error && disabledMods.length > 0}
    <ParkedModal
      mods={disabledMods}
      busy={appState.loading}
      onEnable={haulBack}
      onClose={() => (parkedOpen = false)}
    />
  {/if}

  {#if activeWarning && appState.report && !appState.error}
    <WarningModal
      warning={activeWarning}
      report={appState.report}
      busy={appState.loading}
      onStamp={stampWarning}
      onCancel={() => (activeWarning = null)}
      onReveal={reveal}
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
  .ashore-strip {
    display: flex;
    justify-content: flex-end;
    padding: 2px var(--pad) 6px;
  }
</style>
