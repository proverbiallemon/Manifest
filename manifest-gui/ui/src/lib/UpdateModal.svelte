<script lang="ts">
  import { t } from "../copy.svelte";

  let {
    version,
    busy,
    onInstall,
    onLater,
  }: {
    version: string;
    busy: boolean;
    onInstall: () => void;
    onLater: () => void;
  } = $props();
</script>

<div class="overlay" role="dialog" aria-label="update available">
  <div class="sheet">
    <div class="heading">{t("updateTitle")}</div>
    <hr class="ledger-rule double" />
    <p class="fine-print">{t("updateBody")}: {version}</p>
    {#if busy}
      <p class="fine-print faded">{t("updateBusy")}</p>
    {/if}
    <hr class="ledger-rule" />
    <div class="actions">
      <span></span>
      <span>
        <button onclick={onLater} disabled={busy}>{t("updateLater")}</button>
        <button class="confirm" onclick={onInstall} disabled={busy}>{t("updateInstall")}</button>
      </span>
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
    width: min(440px, 92vw);
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
  .actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 8px;
    gap: 8px;
  }
  .confirm {
    border-color: var(--red-ink);
    color: var(--red-ink);
    box-shadow: 2px 2px 0 var(--red-ink);
  }
</style>
