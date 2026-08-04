<script lang="ts">
  import type { ModToggle, Report, ReportMod, Warning } from "../types";
  import { t } from "../copy.svelte";

  let {
    warning,
    report,
    busy,
    onStamp,
    onCancel,
    onReveal,
  }: {
    warning: Warning;
    report: Report;
    busy: boolean;
    onStamp: (changes: ModToggle[]) => void;
    onCancel: () => void;
    onReveal: (path: string) => void;
  } = $props();

  const keepOne = $derived(
    warning.kind === "mutual_overlap" || warning.kind === "duplicate_gamebanana_mod"
  );
  const names = $derived(
    "names" in warning ? warning.names : [warning.name]
  );
  const rows: ReportMod[] = $derived(
    report.mods.filter((m) => names.includes(m.name))
  );

  function initialKeeper(): string | null {
    if (warning.kind !== "mutual_overlap") return null;
    // last name in current load order wins today; pre-stamp it
    const order = report.current_order;
    let winner: string | null = null;
    for (const n of order) if (names.includes(n)) winner = n;
    const row = rows.find((m) => m.name === winner);
    return row ? row.path : null;
  }

  let keeperPath = $state<string | null>(null);
  $effect(() => {
    warning;
    keeperPath = initialKeeper();
  });

  const changes: ModToggle[] = $derived(
    keepOne
      ? rows.filter((m) => m.path !== keeperPath).map((m) => ({ path: m.path, enabled: false }))
      : rows.map((m) => ({ path: m.path, enabled: false }))
  );
  const stampDisabled = $derived(busy || (keepOne && keeperPath === null));
</script>

<div class="overlay" role="dialog" aria-label="warning actions">
  <div class="sheet">
    <div class="heading">{t("modalContested")}</div>
    <hr class="ledger-rule double" />
    {#if keepOne}
      <p class="fine-print faded">{t("chooseKeeper")}</p>
    {/if}
    <div class="rows">
      {#each rows as mod (mod.path)}
        <div class="row" class:ashore={keepOne && keeperPath !== mod.path}>
          <span class="name">
            {mod.name}
            <span class="fine-print faded">{mod.path}</span>
          </span>
          {#if keepOne}
            {#if keeperPath === mod.path}
              <span class="stamp">{t("keepStamp")}</span>
            {:else}
              <button onclick={() => (keeperPath = mod.path)} disabled={busy}>
                {t("keepStamp")}
              </button>
            {/if}
          {:else}
            <span class="stamp">{t("setAshore")}</span>
          {/if}
          {#if warning.kind === "unlistable"}
            <button onclick={() => onReveal(mod.path)} disabled={busy} title="show this file in the system file manager">
              {t("revealItem")}
            </button>
          {/if}
        </div>
      {/each}
    </div>
    <hr class="ledger-rule" />
    <div class="actions">
      <span class="fine-print faded">{t("stampNote")}</span>
      <span>
        <button onclick={onCancel} disabled={busy}>{t("cancel")}</button>
        <button class="confirm" onclick={() => onStamp(changes)} disabled={stampDisabled}>
          {t("confirm")}
        </button>
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
    padding: 8px 0;
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
    display: flex;
    flex-direction: column;
  }
  .row.ashore .name {
    color: var(--ink-faded);
    text-decoration: line-through;
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
