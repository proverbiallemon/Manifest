<script lang="ts">
  import type { ReportMod } from "../types";
  import { t } from "../copy.svelte";

  let {
    mods,
    busy,
    onEnable,
    onClose,
  }: {
    mods: ReportMod[];
    busy: boolean;
    onEnable: (path: string) => void;
    onClose: () => void;
  } = $props();
</script>

<div class="overlay" role="dialog" aria-label="mods set ashore">
  <div class="sheet">
    <div class="heading">{t("ashoreTitle")}</div>
    <hr class="ledger-rule double" />
    <div class="rows">
      {#each mods as mod (mod.path)}
        <div class="row">
          <span class="name">
            {mod.name}
            <span class="fine-print faded">{mod.path}</span>
          </span>
          <button onclick={() => onEnable(mod.path)} disabled={busy}>
            {t("haulBack")}
          </button>
        </div>
      {/each}
    </div>
    <hr class="ledger-rule" />
    <div class="actions">
      <span></span>
      <button onclick={onClose} disabled={busy}>{t("done")}</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(43, 38, 34, 0.55);
    display: grid;
    place-items: center;
    z-index: 10;
  }
  .sheet {
    background: var(--paper);
    background-image: url("../assets/paper.png");
    image-rendering: pixelated;
    border: 3px solid var(--ink);
    box-shadow: 6px 6px 0 rgba(43, 38, 34, 0.5);
    width: min(640px, 92vw);
    max-height: 84vh;
    display: flex;
    flex-direction: column;
    padding: var(--pad);
  }
  .heading {
    letter-spacing: 4px;
    text-align: center;
    padding-bottom: 6px;
  }
  .rows {
    overflow-y: auto;
    overflow-x: hidden;
    padding: 8px 14px 8px 0;
    font-size: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 0;
  }
  .row .name {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
    display: flex;
    flex-direction: column;
  }
  .row button {
    flex-shrink: 0;
  }
  .actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 8px;
    gap: 8px;
  }
</style>
