<script lang="ts">
  let {
    configPath,
    sortNeeded,
    moveCount,
    loading,
    onRescan,
    onRestow,
    onChooseConfig,
    onChooseMods,
  }: {
    configPath: string | null;
    sortNeeded: boolean;
    moveCount: number;
    loading: boolean;
    onRescan: () => void;
    onRestow: () => void;
    onChooseConfig: () => void;
    onChooseMods: () => void;
  } = $props();

  let pathShown = $state(false);
</script>

<header>
  <div class="masthead">MANIFEST</div>
  <hr class="ledger-rule double" />
  <div class="bar">
    <button class="pathbtn faded" onclick={() => (pathShown = !pathShown)} title={configPath ?? "no config"}>
      {#if configPath}
        {pathShown ? configPath : "reading: ..." + configPath.slice(-28)}
      {:else}
        no manifest on file
      {/if}
    </button>
    <span class="actions">
      <button onclick={onChooseConfig}>config</button>
      <button onclick={onChooseMods}>hold</button>
      <button onclick={onRescan} disabled={loading || !configPath}>take inventory</button>
      {#if sortNeeded}
        <button class="restow" onclick={onRestow} disabled={loading}>
          Re-stow the hold ({moveCount})
        </button>
      {:else if configPath}
        <span class="faded">hold is in order</span>
      {/if}
    </span>
  </div>
</header>

<style>
  header {
    padding: var(--pad) var(--pad) 0;
  }
  .masthead {
    font-size: 26px;
    letter-spacing: 10px;
    text-align: center;
    padding: 4px 0 8px;
  }
  .bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    flex-wrap: wrap;
  }
  .pathbtn {
    border: none;
    background: none;
    box-shadow: none;
    padding: 0;
    font-size: 11px;
  }
  .pathbtn:active {
    translate: none;
  }
  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .restow {
    border-color: var(--red-ink);
    color: var(--red-ink);
    box-shadow: 2px 2px 0 var(--red-ink);
  }
</style>
