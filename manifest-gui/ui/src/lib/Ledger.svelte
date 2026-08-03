<script lang="ts">
  import type { Report } from "../types";
  import { conflictCount } from "../types";
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

  const byName = $derived(new Map(report.mods.map((m) => [m.name, m])));
  const loaded = $derived(
    report.current_order
      .map((n) => byName.get(n))
      .filter((m) => m !== undefined)
  );
  const inOrder = $derived(new Set(report.current_order));
  const rest = $derived(report.mods.filter((m) => !inOrder.has(m.name)));
</script>

<section aria-label="cargo ledger">
  <div class="heading">LADING, IN ORDER OF LOAD</div>
  <hr class="ledger-rule double" />
  {#each loaded as mod, i}
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
    <div class="heading notloaded-gap">NOT LOADED</div>
    <hr class="ledger-rule" />
    {#each rest as mod}
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
