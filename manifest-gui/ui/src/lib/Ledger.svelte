<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import type { Report } from "../types";
  import { conflictCount, deriveZones, firstRowIndexes, zoneNameLists } from "../types";
  import { t } from "../copy.svelte";
  import LedgerRow from "./LedgerRow.svelte";
  import {
    applyDrag,
    slotFromPointer,
    slotY,
    type DropSlot,
    type ZoneBand,
    type ZoneName,
  } from "./dragPlan";

  let {
    report,
    selectedPath,
    onSelect,
    onPin,
    onReorder,
    busy = false,
  }: {
    report: Report;
    selectedPath: string | null;
    onSelect: (path: string) => void;
    onPin: (name: string, position: "top" | "bottom" | null) => void;
    onReorder: (fore: string[], free: string[], aft: string[]) => void;
    busy?: boolean;
  } = $props();

  const zones = $derived(deriveZones(report));
  const names = $derived(zoneNameLists(zones));
  const reps = $derived({
    fore: firstRowIndexes(zones.fore),
    free: firstRowIndexes(zones.free),
    aft: firstRowIndexes(zones.aft),
  });
  const loadedPaths = $derived(
    new Set([...zones.fore, ...zones.free, ...zones.aft].map((m) => m.path))
  );
  const rest = $derived(report.mods.filter((m) => !loadedPaths.has(m.path)));
  const lineOf = $derived.by(() => {
    const all = [...zones.fore, ...zones.free, ...zones.aft];
    return new Map(all.map((m, i) => [m.path, i + 1]));
  });

  let dragging = $state<DropSlot | null>(null);
  let slot = $state<DropSlot | null>(null);
  let indicatorY = $state<number | null>(null);
  let zoneEls: Record<ZoneName, HTMLElement | null> = {
    fore: null,
    free: null,
    aft: null,
  };
  let rowEls: Record<ZoneName, HTMLElement[]> = { fore: [], free: [], aft: [] };
  let sectionEl: HTMLElement | null = null;

  // Geometry is captured once per drag (and again on scroll), never per
  // pointermove: rebuilding it on every move forced a full reflow of the
  // whole ledger per event and drowned the main thread on large libraries.
  let cachedBands: ZoneBand[] = [];
  let sectionTop = 0;
  let raf = 0;
  let lastY = 0;

  const liftedName = $derived(
    dragging ? names[dragging.zone][dragging.index] : null
  );

  // A drag started before an in-flight reorder lands would otherwise apply
  // grab-time indexes to the freshly re-derived zones; bail out whenever the
  // report identity changes while a drag is active. Read dragging untracked
  // so this effect only reruns on report changes, not on every grab/drop.
  $effect(() => {
    report;
    if (untrack(() => dragging)) {
      cancelDrag();
    }
  });

  onDestroy(() => {
    endDragListeners();
  });

  // Row midpoints and tops are built from one representative row per unique
  // name (the first row of each duplicate block), so band slot indexes land
  // in the same deduped-name space as the reorder lists.
  function captureBands() {
    sectionTop = sectionEl?.getBoundingClientRect().top ?? 0;
    cachedBands = (["fore", "free", "aft"] as ZoneName[]).map((zone) => {
      const rect = zoneEls[zone]?.getBoundingClientRect();
      const rowRects = reps[zone]
        .map((i) => rowEls[zone][i])
        .filter(Boolean)
        .map((r) => r.getBoundingClientRect());
      return {
        zone,
        top: rect?.top ?? 0,
        bottom: rect?.bottom ?? 0,
        rowMids: rowRects.map((b) => (b.top + b.bottom) / 2),
        rowTops: rowRects.map((b) => b.top),
      };
    });
  }

  function grab(zone: ZoneName, index: number) {
    if (busy) return;
    captureBands();
    dragging = { zone, index };
    slot = null;
    indicatorY = null;
    window.addEventListener("pointermove", track);
    window.addEventListener("pointerup", drop);
    window.addEventListener("pointercancel", cancelDrag);
    window.addEventListener("scroll", onScroll, true);
  }

  // Pure math per frame against the captured geometry; at most one slot
  // update per animation frame no matter how fast the pointer reports.
  function track(e: { clientY: number }) {
    lastY = e.clientY;
    if (!raf) raf = requestAnimationFrame(applyTrack);
  }

  function applyTrack() {
    raf = 0;
    slot = slotFromPointer(cachedBands, lastY);
    indicatorY = slotY(cachedBands, slot) - sectionTop;
  }

  function onScroll() {
    captureBands();
    if (dragging) {
      slot = slotFromPointer(cachedBands, lastY);
      indicatorY = slotY(cachedBands, slot) - sectionTop;
    }
  }

  function endDragListeners() {
    window.removeEventListener("pointermove", track);
    window.removeEventListener("pointerup", drop);
    window.removeEventListener("pointercancel", cancelDrag);
    window.removeEventListener("scroll", onScroll, true);
    if (raf) cancelAnimationFrame(raf);
    raf = 0;
  }

  function cancelDrag() {
    endDragListeners();
    dragging = null;
    slot = null;
    indicatorY = null;
  }

  function drop(e: { clientY: number }) {
    endDragListeners();
    const source = dragging;
    dragging = null;
    const target = slot ?? slotFromPointer(cachedBands, e.clientY);
    slot = null;
    indicatorY = null;
    if (!source) return;
    const sameSlot =
      target.zone === source.zone &&
      (target.index === source.index || target.index === source.index + 1);
    if (sameSlot) return;
    const next = applyDrag(names, source, target);
    onReorder(next.fore, next.free, next.aft);
  }
</script>

<section aria-label="cargo ledger" bind:this={sectionEl}>
  <div class="heading">{t("ledgerHeading")}</div>
  <hr class="ledger-rule double" />
  {#if dragging && indicatorY !== null}
    <div class="drop-slot" style:top="{indicatorY}px"></div>
  {/if}
  <div class="zone-box" bind:this={zoneEls.fore}>
    {#if zones.fore.length > 0}
      <div class="heading zone">{t("zoneFore")}</div>
    {/if}
    {#each zones.fore as mod, i (mod.path)}
      <div bind:this={rowEls.fore[i]}>
        <LedgerRow
          {mod}
          line={lineOf.get(mod.path) ?? null}
          conflicts={conflictCount(report, mod.name)}
          selected={mod.path === selectedPath}
          lifted={mod.name === liftedName}
          {onSelect}
          {onPin}
          onGrab={() => grab("fore", names.fore.indexOf(mod.name))}
        />
      </div>
    {/each}
  </div>
  {#if zones.fore.length > 0}
    <hr class="ledger-rule" />
  {/if}
  <div class="zone-box" bind:this={zoneEls.free}>
    {#each zones.free as mod, i (mod.path)}
      <div bind:this={rowEls.free[i]}>
        <LedgerRow
          {mod}
          line={lineOf.get(mod.path) ?? null}
          conflicts={conflictCount(report, mod.name)}
          selected={mod.path === selectedPath}
          lifted={mod.name === liftedName}
          {onSelect}
          {onPin}
          onGrab={() => grab("free", names.free.indexOf(mod.name))}
        />
      </div>
    {/each}
  </div>
  {#if zones.aft.length > 0}
    <hr class="ledger-rule" />
  {/if}
  <div class="zone-box" bind:this={zoneEls.aft}>
    {#if zones.aft.length > 0}
      <div class="heading zone">{t("zoneAft")}</div>
    {/if}
    {#each zones.aft as mod, i (mod.path)}
      <div bind:this={rowEls.aft[i]}>
        <LedgerRow
          {mod}
          line={lineOf.get(mod.path) ?? null}
          conflicts={conflictCount(report, mod.name)}
          selected={mod.path === selectedPath}
          lifted={mod.name === liftedName}
          {onSelect}
          {onPin}
          onGrab={() => grab("aft", names.aft.indexOf(mod.name))}
        />
      </div>
    {/each}
  </div>
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
  section {
    position: relative;
  }
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
  .drop-slot {
    position: absolute;
    left: 8px;
    right: 8px;
    border-top: 2px dashed var(--ink);
    pointer-events: none;
    z-index: 1;
  }
  .zone-box {
    min-height: 8px;
  }
</style>
