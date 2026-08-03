<script lang="ts">
  import type { Warning } from "../types";
  import { t } from "../copy.svelte";

  let { warnings }: { warnings: Warning[] } = $props();

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
      <div class="paper-slip">
        <div>{headline(w)}</div>
        <div class="fine-print faded action-slot">{guidance(w)}</div>
      </div>
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
</style>
