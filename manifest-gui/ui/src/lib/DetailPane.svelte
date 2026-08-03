<script lang="ts">
  import type { Report } from "../types";
  import { overlapsFor } from "../types";
  import { t } from "../copy.svelte";

  let { report, selectedPath }: { report: Report; selectedPath: string | null } =
    $props();

  const mod = $derived(report.mods.find((m) => m.path === selectedPath) ?? null);
  const overlaps = $derived(mod ? overlapsFor(report, mod.name) : null);
</script>

<aside aria-label="entry detail">
  {#if !mod}
    <p class="faded">{t("selectEntry")}</p>
  {:else}
    <div class="title">{mod.name}</div>
    <p class="fine-print faded">{mod.path}</p>
    <hr class="ledger-rule" />
    {#if mod.error}
      <p class="error">{t("unlistable")}: {mod.error}</p>
    {:else if overlaps && overlaps.contestedAssets.length === 0}
      <p class="faded">{t("clearMod")}</p>
    {:else if overlaps}
      {#each overlaps.prevailsOver as o}
        <p>{t("prevailsOver")} {o.name} on {o.count} {o.count === 1 ? t("entrySingular") : t("entryPlural")}</p>
      {/each}
      {#each overlaps.overriddenBy as o}
        <p class="overridden">{t("overriddenBy")} {o.name} on {o.count} {o.count === 1 ? t("entrySingular") : t("entryPlural")}</p>
      {/each}
      <hr class="ledger-rule" />
      <div class="fine-print contested">
        {#each overlaps.contestedAssets as asset}
          <div>{asset}</div>
        {/each}
      </div>
    {/if}
  {/if}
</aside>

<style>
  aside {
    padding: var(--pad);
    border-left: 2px solid var(--rule);
    overflow-y: auto;
    height: 100%;
  }
  .title {
    letter-spacing: 1px;
  }
  .error {
    color: var(--red-ink);
  }
  .overridden {
    color: var(--red-ink);
  }
  .contested {
    max-height: 40vh;
    overflow-y: auto;
  }
</style>
