<script lang="ts">
  import type { Report } from "../types";
  import { conflictCount, deriveZones } from "../types";
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

  const zones = $derived(deriveZones(report));
  const loadedPaths = $derived(
    new Set([...zones.fore, ...zones.free, ...zones.aft].map((m) => m.path))
  );
  const rest = $derived(report.mods.filter((m) => !loadedPaths.has(m.path)));
  const lineOf = $derived.by(() => {
    const all = [...zones.fore, ...zones.free, ...zones.aft];
    return new Map(all.map((m, i) => [m.path, i + 1]));
  });
</script>

<section aria-label="cargo ledger">
  <div class="heading">{t("ledgerHeading")}</div>
  <hr class="ledger-rule double" />
  {#if zones.fore.length > 0}
    <div class="heading zone">{t("zoneFore")}</div>
    {#each zones.fore as mod (mod.path)}
      <LedgerRow
        {mod}
        line={lineOf.get(mod.path) ?? null}
        conflicts={conflictCount(report, mod.name)}
        selected={mod.path === selectedPath}
        {onSelect}
        {onPin}
      />
    {/each}
    <hr class="ledger-rule" />
  {/if}
  {#each zones.free as mod (mod.path)}
    <LedgerRow
      {mod}
      line={lineOf.get(mod.path) ?? null}
      conflicts={conflictCount(report, mod.name)}
      selected={mod.path === selectedPath}
      {onSelect}
      {onPin}
    />
  {/each}
  {#if zones.aft.length > 0}
    <hr class="ledger-rule" />
    <div class="heading zone">{t("zoneAft")}</div>
    {#each zones.aft as mod (mod.path)}
      <LedgerRow
        {mod}
        line={lineOf.get(mod.path) ?? null}
        conflicts={conflictCount(report, mod.name)}
        selected={mod.path === selectedPath}
        {onSelect}
        {onPin}
      />
    {/each}
  {/if}
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
  .zone {
    color: var(--ink-faded);
    letter-spacing: 2px;
  }
</style>
