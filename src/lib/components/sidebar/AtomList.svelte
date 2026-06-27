<script lang="ts">
  import { atomStore, searchStore } from "$lib/stores";

  let stripEl: HTMLDivElement;

  function handleWheel(e: WheelEvent) {
    if (stripEl && Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      e.preventDefault();
      stripEl.scrollLeft += e.deltaY;
    }
  }
</script>

<div class="atom-strip-section">
  {#if searchStore.hasSearched}
    <span class="strip-label">検索結果 ({searchStore.searchResults.length})</span>
    {#if searchStore.searchResults.length === 0 && !searchStore.isSearching}
      <span class="empty-msg">一致なし</span>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="atom-strip" bind:this={stripEl} onwheel={handleWheel}>
        {#each searchStore.searchResults as atom}
          <button
            class="atom-card"
            class:atom-card-active={atomStore.lastAtom?.frontmatter.id === atom.id}
            onclick={() => atomStore.loadAtom(atom.id)}
          >
            <span class="atom-title">{atom.title}</span>
            {#if atom.snippet}
              <span class="atom-desc">{atom.snippet}</span>
            {/if}
            <div class="atom-meta">
              <span class="atom-id">{atom.id.includes('-') ? atom.id.split('-')[0] : atom.id}</span>
              <span class="atom-score">Score: {atom.score.toFixed(3)}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  {:else}
    <span class="strip-label">Atoms</span>
    {#if atomStore.isLoadingRecent}
      <span class="empty-msg">読み込み中...</span>
    {:else if atomStore.recentAtoms.length === 0}
      <span class="empty-msg">履歴なし</span>
    {:else}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="atom-strip" bind:this={stripEl} onwheel={handleWheel}>
        {#each atomStore.recentAtoms as atom}
          <button
            class="atom-card"
            class:atom-card-active={atomStore.lastAtom?.frontmatter.id === atom.id}
            onclick={() => atomStore.loadAtom(atom.id)}
          >
            <span class="atom-title">{atom.title}</span>
            <span class="atom-id">{atom.id.includes('-') ? atom.id.split('-')[0] : atom.id}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .atom-strip-section {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 1rem;
    background: var(--bg-surface);
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    overflow: hidden;
  }

  .strip-label {
    font-size: 0.7rem;
    color: var(--text-secondary);
    white-space: nowrap;
    flex-shrink: 0;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .atom-strip {
    display: flex;
    gap: 0.5rem;
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    flex: 1;
    padding: 0.25rem 0;
  }

  .atom-strip::-webkit-scrollbar {
    height: 4px;
  }

  .atom-strip::-webkit-scrollbar-track {
    background: transparent;
  }

  .atom-strip::-webkit-scrollbar-thumb {
    background: var(--bg-elevated);
    border-radius: 2px;
  }

  .atom-strip::-webkit-scrollbar-thumb:hover {
    background: var(--scrollbar-hover);
  }

  .atom-card {
    flex-shrink: 0;
    width: 180px;
    scroll-snap-align: start;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.4rem 0.5rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    text-align: left;
  }

  .atom-card:hover {
    background: var(--bg-elevated);
    border-color: var(--accent-blue);
  }

  .atom-card-active {
    background: var(--accent-blue-bg);
    border-color: var(--accent-blue);
    box-shadow: 0 0 8px var(--atom-active-shadow);
  }

  .atom-title {
    font-weight: 600;
    font-size: 0.75rem;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }

  .atom-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .atom-id {
    font-size: 0.6rem;
    color: var(--text-muted);
  }

  .atom-score {
    font-size: 0.6rem;
    color: var(--accent-blue);
    background: var(--accent-blue-bg);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-family: monospace;
  }

  .atom-desc {
    font-size: 0.65rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .empty-msg {
    color: var(--text-muted);
    font-size: 0.75rem;
    white-space: nowrap;
  }
</style>
