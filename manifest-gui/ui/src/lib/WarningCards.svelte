<script lang="ts">
  import type { Warning } from "../types";

  let { warnings }: { warnings: Warning[] } = $props();

  function headline(w: Warning): string {
    switch (w.kind) {
      case "total_eclipse":
        return `${w.name} is fully covered by later cargo`;
      case "mutual_overlap":
        return `${w.names.join(", ")} carry identical cargo`;
      case "unlistable":
        return `${w.name} could not be inventoried`;
      case "duplicate_gamebanana_mod":
        return `${w.names.join(", ")} are the same shipment twice (GameBanana ${w.mod_id})`;
    }
  }

  function guidance(w: Warning): string {
    switch (w.kind) {
      case "total_eclipse":
        return "it does nothing where it sits; consider unloading it";
      case "mutual_overlap":
        return "only the last one prevails; consider keeping one";
      case "unlistable":
        return w.reason;
      case "duplicate_gamebanana_mod":
        return "consider removing the extra copy";
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
