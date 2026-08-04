<script lang="ts">
  import type { Report } from "../types";
  import { conflictCount } from "../types";
  import { t } from "../copy.svelte";
  import LedgerRow from "./LedgerRow.svelte";

  let {
    report,
    selectedPath,
    onSelect,
    onPin,
  }: {
    report: Report;
    selectedPath: string | null;
    onSelect: (path: string) => void;
    onPin: (name: string, position: "top" | "bottom" | null) => void;
  } = $props();

  const orderIndex = $derived(
    new Map(report.current_order.map((n, i) => [n, i]))
  );
  const loaded = $derived(
    report.mods
      .filter((m) => m.enabled && orderIndex.has(m.name))
      .slice()
      .sort(
        (a, b) =>
          orderIndex.get(a.name)! - orderIndex.get(b.name)! ||
          a.path.localeCompare(b.path)
      )
  );
  const loadedPaths = $derived(new Set(loaded.map((m) => m.path)));
  const rest = $derived(report.mods.filter((m) => !loadedPaths.has(m.path)));
</script>

<section aria-label="cargo ledger">
  <div class="heading">{t("ledgerHeading")}</div>
  <hr class="ledger-rule double" />
  {#each loaded as mod, i (mod.path)}
    <LedgerRow
      {mod}
      line={i + 1}
      conflicts={conflictCount(report, mod.name)}
      selected={mod.path === selectedPath}
      {onSelect}
      {onPin}
    />
  {/each}
  {#if rest.length > 0}
    <div class="heading notloaded-gap">{t("notLoaded")}</div>
    <hr class="ledger-rule" />
    {#each rest as mod (mod.path)}
      <LedgerRow
        {mod}
        line={null}
        conflicts={0}
        selected={mod.path === selectedPath}
        {onSelect}
        {onPin}
      />
    {/each}
  {/if}
</section>

<style>
  .heading {
    letter-spacing: 3px;
    font-size: 11px;
    padding: 6px 8px 2px;
  }
  .notloaded-gap {
    margin-top: 16px;
    color: var(--ink-faded);
  }
</style>
