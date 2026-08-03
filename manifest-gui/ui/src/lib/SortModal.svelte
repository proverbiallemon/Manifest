<script lang="ts">
  import type { Report } from "../types";

  let {
    report,
    busy,
    onConfirm,
    onCancel,
  }: {
    report: Report;
    busy: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  const moved = $derived(new Set(report.moves.map((m) => m.name)));
  const reasons = $derived(new Map(report.moves.map((m) => [m.name, m.reason])));
</script>

<div class="overlay" role="dialog" aria-label="re-stow the hold">
  <div class="sheet">
    <div class="heading">RE-STOW THE HOLD</div>
    <hr class="ledger-rule double" />
    <div class="columns">
      <div>
        <div class="col-title faded">AS STOWED</div>
        {#each report.current_order as name}
          <div class:moved={moved.has(name)}>{name}</div>
        {/each}
      </div>
      <div>
        <div class="col-title faded">AS PROPOSED</div>
        {#each report.proposed_order as name}
          <div class:moved={moved.has(name)} title={reasons.get(name) ?? ""}>
            {name}
            {#if reasons.get(name)}
              <div class="fine-print faded reason">{reasons.get(name)}</div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
    <hr class="ledger-rule" />
    <div class="actions">
      <span class="fine-print faded">{report.moves.length} entries move; nothing is written until stamped</span>
      <span>
        <button onclick={onCancel} disabled={busy}>Belay that</button>
        <button class="confirm" onclick={onConfirm} disabled={busy}>Stamp it</button>
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
    width: min(860px, 92vw);
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
  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--pad);
    overflow-y: auto;
    padding: 8px 0;
    font-size: 12px;
  }
  .col-title {
    letter-spacing: 2px;
    font-size: 10px;
    padding-bottom: 4px;
  }
  .moved {
    color: var(--red-ink);
  }
  .reason {
    padding-left: 12px;
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
