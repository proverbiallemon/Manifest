<script lang="ts">
  import type { ReportMod } from "../types";
  import sealUrl from "../assets/seal.png";
  import glyphUrl from "../assets/glyph-error.png";

  let {
    mod,
    line,
    conflicts,
    selected = false,
    onSelect,
    onPin,
  }: {
    mod: ReportMod;
    line: number | null;
    conflicts: number;
    selected?: boolean;
    onSelect: (path: string) => void;
    onPin: (name: string, position: "top" | "bottom" | null) => void;
  } = $props();
</script>

<div
  class="row"
  class:selected
  class:notloaded={!mod.enabled}
  role="row"
  tabindex="0"
  onclick={() => onSelect(mod.path)}
  onkeydown={(e) => e.key === "Enter" && onSelect(mod.path)}
>
  <span class="line faded">{line ?? ""}</span>
  <span class="name">
    {#if mod.pinned}
      <img class="pixel seal" src={sealUrl} alt="pinned {mod.pinned}" title="pinned {mod.pinned}" />
    {/if}
    {mod.name}
  </span>
  <span class="count faded">{mod.asset_count}</span>
  <span class="status">
    {#if mod.error}
      <img class="pixel" src={glyphUrl} alt="unlistable" title={mod.error} />
    {:else if conflicts > 0}
      <span class="stamp">CONFLICT</span>
    {:else}
      <span class="faded">clear</span>
    {/if}
  </span>
  <span class="pins">
    <button title="pin to top (loads first)" onclick={(e) => { e.stopPropagation(); onPin(mod.name, "top"); }}>top</button>
    <button title="pin to bottom (loads last, wins)" onclick={(e) => { e.stopPropagation(); onPin(mod.name, "bottom"); }}>btm</button>
    {#if mod.pinned}
      <button title="unpin" onclick={(e) => { e.stopPropagation(); onPin(mod.name, null); }}>x</button>
    {/if}
  </span>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 3.5em 1fr 5em 8em 10em;
    gap: 8px;
    align-items: center;
    padding: 3px 8px;
    border-bottom: 1px solid var(--rule);
    cursor: pointer;
  }
  .row.selected {
    background: var(--paper-dark);
    outline: 2px solid var(--ink);
  }
  .row.notloaded {
    color: var(--ink-faded);
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    text-align: right;
  }
  .seal {
    width: 15px;
    height: 14px;
    vertical-align: -2px;
  }
  .pins {
    visibility: hidden;
    text-align: right;
  }
  .row:hover .pins,
  .row:focus-within .pins {
    visibility: visible;
  }
  .pins button {
    font-size: 10px;
    padding: 1px 5px;
    box-shadow: 1px 1px 0 var(--ink);
  }
</style>
