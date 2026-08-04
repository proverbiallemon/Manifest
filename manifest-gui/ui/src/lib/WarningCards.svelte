<script lang="ts">
  import type { Warning } from "../types";
  import { t } from "../copy.svelte";

  let { warnings, onAct }: { warnings: Warning[]; onAct: (w: Warning) => void } = $props();

  function headline(w: Warning): string {
    switch (w.kind) {
      case "total_eclipse":
        return `${w.name} ${t("eclipseHeadline")}`;
      case "mutual_overlap":
        return `${w.names.join(", ")} ${t("overlapHeadline")}`;
      case "unlistable":
        return `${w.name} ${t("unlistableHeadline")}`;
      case "duplicate_gamebanana_mod":
        return `${w.names.join(", ")} ${t("duplicateHeadline")} (GameBanana ${w.mod_id})`;
    }
  }

  function guidance(w: Warning): string {
    switch (w.kind) {
      case "total_eclipse":
        return t("eclipseGuidance");
      case "mutual_overlap":
        return t("overlapGuidance");
      case "unlistable":
        return w.reason;
      case "duplicate_gamebanana_mod":
        return t("duplicateGuidance");
    }
  }
</script>

{#if warnings.length > 0}
  <section class="cards" aria-label="warnings">
    {#each warnings as w}
      <button class="paper-slip card" onclick={() => onAct(w)}>
        <div>{headline(w)}</div>
        <div class="fine-print faded action-slot">{guidance(w)}</div>
        <div class="fine-print act-hint">{t("cardAct")} ▸</div>
      </button>
    {/each}
  </section>
{/if}

<style>
  .cards {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    padding: 4px var(--pad) 8px;
  }
  .paper-slip {
    min-width: 240px;
    max-width: 320px;
    flex-shrink: 0;
  }
  .card {
    text-align: left;
    font: inherit;
    display: block;
    background: var(--paper-dark);
    box-shadow: 2px 2px 0 rgba(43, 38, 34, 0.25);
    border: 2px solid var(--rule);
    border-left: 6px solid var(--red);
  }
  .act-hint {
    color: var(--red-ink);
    letter-spacing: 1px;
    padding-top: 4px;
  }
</style>
