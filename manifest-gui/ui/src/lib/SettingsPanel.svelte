<script lang="ts">
  import { appState, setVoice } from "../state.svelte";
  import { t } from "../copy.svelte";

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  let panelEl: HTMLDivElement | null = $state(null);

  function handleKeydown(e: KeyboardEvent) {
    if (open && e.key === "Escape") onClose();
  }

  function handleWindowClick(e: MouseEvent) {
    if (!open || !panelEl) return;
    // the parent wrap also holds the toggle button; its own handler owns those clicks
    const inside = panelEl.parentElement ?? panelEl;
    if (!inside.contains(e.target as Node)) onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} onclick={handleWindowClick} />

{#if open}
  <div class="paper-slip panel" role="dialog" aria-label="settings" bind:this={panelEl}>
    <div class="heading">{t("settings")}</div>
    <hr class="ledger-rule" />
    <div class="options">
      <button
        class="option"
        class:active={appState.voice === "ship"}
        onclick={() => setVoice("ship")}
      >
        {appState.voice === "ship" ? ">" : " "} {t("voiceShip")}
      </button>
      <button
        class="option"
        class:active={appState.voice === "plain"}
        onclick={() => setVoice("plain")}
      >
        {appState.voice === "plain" ? ">" : " "} {t("voicePlain")}
      </button>
    </div>
    <hr class="ledger-rule" />
    <div class="close-row">
      <button onclick={onClose}>{t("done")}</button>
    </div>
  </div>
{/if}

<style>
  .panel {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 20;
    width: 260px;
  }
  .heading {
    letter-spacing: 2px;
    font-size: 11px;
    padding-bottom: 4px;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 0;
  }
  .option {
    text-align: left;
    background: none;
    border: none;
    box-shadow: none;
    padding: 2px 0;
    color: var(--ink-faded);
  }
  .option:active {
    translate: none;
  }
  .option.active {
    color: var(--ink);
  }
  .close-row {
    display: flex;
    justify-content: flex-end;
    padding-top: 8px;
  }
</style>
