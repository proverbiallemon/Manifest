<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import type { Report } from "../types";
  import { conflictCount, deriveZones, firstRowIndexes, zoneNameLists } from "../types";
  import { t } from "../copy.svelte";
  import LedgerRow from "./LedgerRow.svelte";
  import {
    applyDrag,
    rowShifts,
    settleY,
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
  let liftedName = $state<string | null>(null);
  let followY = $state<number | null>(null);
  let shifts = $state<Record<ZoneName, number[]> | null>(null);
  let settling = $state(false);
  let zoneEls: Record<ZoneName, HTMLElement | null> = {
    fore: null,
    free: null,
    aft: null,
  };
  let rowEls: Record<ZoneName, HTMLElement[]> = { fore: [], free: [], aft: [] };
  let sectionEl: HTMLElement | null = null;

  // Geometry is captured once per drag, never per pointermove: rebuilding it
  // on every move forced a full reflow of the whole ledger per event and
  // drowned the main thread on large libraries. All drag motion is CSS
  // transforms, which never disturb the captured layout; scrolling mid-drag
  // shifts the cached copy by the section's own displacement.
  let baseBands: ZoneBand[] = [];
  let baseSectionTop = 0;
  let cachedBands: ZoneBand[] = [];
  let sectionTop = 0;
  let grabOffsetInRow = 0;
  let raf = 0;
  let lastY = 0;

  const nameIdx = $derived({
    fore: new Map(names.fore.map((n, i) => [n, i])),
    free: new Map(names.free.map((n, i) => [n, i])),
    aft: new Map(names.aft.map((n, i) => [n, i])),
  });

  function wrapperTransform(zone: ZoneName, name: string): string | undefined {
    if (name === liftedName) {
      return followY !== null ? `translateY(${followY}px)` : undefined;
    }
    const s = shifts?.[zone][nameIdx[zone].get(name) ?? -1] ?? 0;
    return s ? `translateY(${s}px)` : undefined;
  }

  // A drag started before an in-flight reorder lands would otherwise apply
  // grab-time indexes to the freshly re-derived zones; bail out whenever the
  // report identity changes while a drag is active. Read dragging untracked
  // so this effect only reruns on report changes, not on every grab/drop.
  $effect(() => {
    report;
    untrack(() => {
      if (dragging) cancelDrag();
      else resetMotion();
    });
  });

  onDestroy(() => {
    endDragListeners();
  });

  // Row midpoints and tops are built from one representative row per unique
  // name (the first row of each duplicate block), so band slot indexes land
  // in the same deduped-name space as the reorder lists.
  function captureBands() {
    baseSectionTop = sectionEl?.getBoundingClientRect().top ?? 0;
    baseBands = (["fore", "free", "aft"] as ZoneName[]).map((zone) => {
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
    cachedBands = baseBands;
    sectionTop = baseSectionTop;
  }

  function grab(zone: ZoneName, index: number, clientY: number) {
    if (busy) return;
    captureBands();
    const band = cachedBands.find((b) => b.zone === zone);
    grabOffsetInRow = clientY - (band?.rowTops[index] ?? clientY);
    dragging = { zone, index };
    liftedName = names[zone][index] ?? null;
    settling = false;
    slot = null;
    indicatorY = null;
    followY = 0;
    shifts = null;
    lastY = clientY;
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
    const source = dragging;
    if (!source) return;
    slot = slotFromPointer(cachedBands, lastY);
    indicatorY = slotY(cachedBands, slot) - sectionTop;
    const band = cachedBands.find((b) => b.zone === source.zone);
    const sourceTop = band?.rowTops[source.index] ?? 0;
    followY = lastY - grabOffsetInRow - sourceTop;
    shifts = rowShifts(cachedBands, source, slot);
  }

  // The section itself never carries a transform, so its displacement is a
  // clean measure of how far the ledger scrolled since the grab.
  function onScroll() {
    const delta =
      (sectionEl?.getBoundingClientRect().top ?? baseSectionTop) -
      baseSectionTop;
    sectionTop = baseSectionTop + delta;
    cachedBands = baseBands.map((b) => ({
      ...b,
      top: b.top + delta,
      bottom: b.bottom + delta,
      rowMids: b.rowMids.map((m) => m + delta),
      rowTops: b.rowTops.map((t) => t + delta),
    }));
    if (dragging) applyTrack();
  }

  function endDragListeners() {
    window.removeEventListener("pointermove", track);
    window.removeEventListener("pointerup", drop);
    window.removeEventListener("pointercancel", cancelDrag);
    window.removeEventListener("scroll", onScroll, true);
    if (raf) cancelAnimationFrame(raf);
    raf = 0;
  }

  function resetMotion() {
    liftedName = null;
    followY = null;
    shifts = null;
    settling = false;
    slot = null;
    indicatorY = null;
  }

  function cancelDrag() {
    endDragListeners();
    dragging = null;
    resetMotion();
  }

  function drop(e: { clientY: number }) {
    endDragListeners();
    const source = dragging;
    dragging = null;
    const target = slot ?? slotFromPointer(cachedBands, e.clientY);
    slot = null;
    indicatorY = null;
    if (!source) {
      resetMotion();
      return;
    }
    const sameSlot =
      target.zone === source.zone &&
      (target.index === source.index || target.index === source.index + 1);
    if (sameSlot) {
      resetMotion();
      return;
    }
    // Glide the block into its gap and hold every slide in place until the
    // reordered report lands and the DOM catches up underneath.
    settling = true;
    followY = settleY(cachedBands, source, target);
    shifts = rowShifts(cachedBands, source, target);
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
      <div
        bind:this={rowEls.fore[i]}
        class="row-slide"
        class:follow={liftedName === mod.name && !settling}
        class:settle={liftedName === mod.name && settling}
        class:sliding={shifts !== null && liftedName !== mod.name}
        style:transform={wrapperTransform("fore", mod.name)}
      >
        <LedgerRow
          {mod}
          line={lineOf.get(mod.path) ?? null}
          conflicts={conflictCount(report, mod.name)}
          selected={mod.path === selectedPath}
          lifted={mod.name === liftedName}
          {onSelect}
          {onPin}
          onGrab={(e) => grab("fore", names.fore.indexOf(mod.name), e.clientY)}
        />
      </div>
    {/each}
  </div>
  {#if zones.fore.length > 0}
    <hr class="ledger-rule" />
  {/if}
  <div class="zone-box" bind:this={zoneEls.free}>
    {#each zones.free as mod, i (mod.path)}
      <div
        bind:this={rowEls.free[i]}
        class="row-slide"
        class:follow={liftedName === mod.name && !settling}
        class:settle={liftedName === mod.name && settling}
        class:sliding={shifts !== null && liftedName !== mod.name}
        style:transform={wrapperTransform("free", mod.name)}
      >
        <LedgerRow
          {mod}
          line={lineOf.get(mod.path) ?? null}
          conflicts={conflictCount(report, mod.name)}
          selected={mod.path === selectedPath}
          lifted={mod.name === liftedName}
          {onSelect}
          {onPin}
          onGrab={(e) => grab("free", names.free.indexOf(mod.name), e.clientY)}
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
      <div
        bind:this={rowEls.aft[i]}
        class="row-slide"
        class:follow={liftedName === mod.name && !settling}
        class:settle={liftedName === mod.name && settling}
        class:sliding={shifts !== null && liftedName !== mod.name}
        style:transform={wrapperTransform("aft", mod.name)}
      >
        <LedgerRow
          {mod}
          line={lineOf.get(mod.path) ?? null}
          conflicts={conflictCount(report, mod.name)}
          selected={mod.path === selectedPath}
          lifted={mod.name === liftedName}
          {onSelect}
          {onPin}
          onGrab={(e) => grab("aft", names.aft.indexOf(mod.name), e.clientY)}
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
  .row-slide.sliding {
    transition: transform 140ms ease;
  }
  .row-slide.follow {
    position: relative;
    z-index: 3;
    transition: none;
    pointer-events: none;
  }
  .row-slide.settle {
    position: relative;
    z-index: 3;
    transition: transform 160ms ease;
    pointer-events: none;
  }
</style>
