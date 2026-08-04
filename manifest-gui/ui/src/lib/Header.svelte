<script lang="ts">
  import { t } from "../copy.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";

  let {
    configPath,
    sortNeeded,
    moveCount,
    loading,
    damaged,
    onRescan,
    onRestow,
    onChooseConfig,
    onChooseMods,
    onCheckUpdates,
  }: {
    configPath: string | null;
    sortNeeded: boolean;
    moveCount: number;
    loading: boolean;
    damaged: boolean;
    onRescan: () => void;
    onRestow: () => void;
    onChooseConfig: () => void;
    onChooseMods: () => void;
    onCheckUpdates: () => Promise<"update" | "current">;
  } = $props();

  let pathShown = $state(false);
  let settingsOpen = $state(false);
</script>

<header>
  <div class="masthead">MANIFEST</div>
  <hr class="ledger-rule double" />
  <div class="bar">
    <button class="pathbtn faded" onclick={() => (pathShown = !pathShown)} title={configPath ?? t("noConfigShort")}>
      {#if configPath}
        {pathShown ? configPath : t("reading") + "..." + configPath.slice(-28)}
      {:else}
        {t("noConfig")}
      {/if}
    </button>
    <span class="actions">
      <button onclick={onChooseConfig} title="choose the shipofharkinian.json config file to read">{t("chooseConfig")}</button>
      <button onclick={onChooseMods} title="choose the mods folder to scan">{t("chooseHold")}</button>
      <button
        onclick={onRescan}
        disabled={loading || !configPath}
        title="rescan the mods folder and rebuild the report"
      >
        {t("rescan")}
      </button>
      {#if sortNeeded && !damaged}
        <button
          class="restow"
          onclick={onRestow}
          disabled={loading}
          title="preview and apply the proposed load order"
        >
          {t("restow")} ({moveCount})
        </button>
      {:else if configPath && !damaged}
        <span class="faded">{t("inOrder")}</span>
      {/if}
      <span class="settings-wrap">
        <button
          onclick={() => (settingsOpen = !settingsOpen)}
          title="choose how the app speaks"
        >
          {t("settings")}
        </button>
        <SettingsPanel open={settingsOpen} onClose={() => (settingsOpen = false)} {onCheckUpdates} />
      </span>
    </span>
  </div>
</header>

<style>
  header {
    padding: var(--pad) var(--pad) 0;
  }
  .masthead {
    font-size: 26px;
    letter-spacing: 10px;
    text-align: center;
    padding: 4px 0 8px;
  }
  .bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    flex-wrap: wrap;
  }
  .pathbtn {
    border: none;
    background: none;
    box-shadow: none;
    padding: 0;
    font-size: 11px;
  }
  .pathbtn:active {
    translate: none;
  }
  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .settings-wrap {
    position: relative;
  }
  .restow {
    border-color: var(--red-ink);
    color: var(--red-ink);
    box-shadow: 2px 2px 0 var(--red-ink);
  }
</style>
