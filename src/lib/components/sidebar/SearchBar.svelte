<script lang="ts">
  import { searchStore } from "$lib/stores";
</script>

<div class="search-section">
  <form class="search-bar" onsubmit={(e) => { e.preventDefault(); searchStore.handleSearch(); }}>
    <input
      type="text"
      bind:value={searchStore.searchQuery}
      onkeydown={(e) => searchStore.handleSearchKeydown(e)}
      oncompositionstart={() => searchStore.handleSearchCompositionStart()}
      oncompositionend={() => searchStore.handleSearchCompositionEnd()}
      placeholder="Vaultを検索..."
      disabled={searchStore.isSearching}
    />
    {#if searchStore.searchQuery || searchStore.hasSearched}
      <button type="button" class="search-clear-btn" onclick={() => searchStore.clearSearch()} title="検索をクリア">✕</button>
    {/if}
    <button type="submit" class="search-exec-btn" disabled={searchStore.isSearching || !searchStore.searchQuery.trim()} title="検索">🔍</button>
  </form>
  {#if searchStore.isSearching}
    <p class="search-status">検索中...</p>
  {/if}
  {#if searchStore.searchError}
    <p class="search-error">{searchStore.searchError}</p>
  {/if}
</div>

<style>
  .search-section {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
  }

  .search-bar {
    display: flex;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
  }

  .search-bar input {
    flex: 1;
    min-width: 120px;
    max-width: 250px;
    padding: 0.3rem 0.5rem;
    background: var(--bg-base);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 0.75rem;
    box-sizing: border-box;
  }

  .search-bar input::placeholder {
    color: var(--text-muted);
  }

  .search-bar input:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .search-bar input:disabled {
    background: var(--bg-surface);
    color: var(--text-muted);
  }

  .search-exec-btn,
  .search-clear-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
    padding: 0 0.4rem;
    flex-shrink: 0;
    width: auto;
  }

  .search-exec-btn:hover:not(:disabled),
  .search-clear-btn:hover {
    background: var(--scrollbar-hover);
    color: var(--text-primary);
  }

  .search-exec-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .search-clear-btn:hover {
    color: var(--accent-red);
    border-color: var(--accent-red);
  }

  .search-status {
    font-size: 0.7rem;
    color: var(--accent-blue);
    margin: 0 0 0 0.5rem;
    white-space: nowrap;
    animation: blink 1.5s infinite;
  }

  .search-error {
    font-size: 0.7rem;
    color: var(--accent-red);
    margin: 0 0 0 0.5rem;
    white-space: nowrap;
  }

  @keyframes blink {
    50% { opacity: 0.5; }
  }
</style>
