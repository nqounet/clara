<script lang="ts">
  import { claraStore } from "$lib/clara.svelte";
</script>

<div class="search-section">
  <form class="search-bar" onsubmit={(e) => { e.preventDefault(); claraStore.handleSearch(); }}>
    <input
      type="text"
      bind:value={claraStore.searchQuery}
      onkeydown={(e) => claraStore.handleSearchKeydown(e)}
      oncompositionstart={() => claraStore.handleSearchCompositionStart()}
      oncompositionend={() => claraStore.handleSearchCompositionEnd()}
      placeholder="Vaultを検索..."
      disabled={claraStore.isSearching}
    />
    {#if claraStore.searchQuery || claraStore.hasSearched}
      <button type="button" class="search-clear-btn" onclick={() => claraStore.clearSearch()} title="検索をクリア">✕</button>
    {/if}
    <button type="submit" class="search-exec-btn" disabled={claraStore.isSearching || !claraStore.searchQuery.trim()} title="検索">🔍</button>
  </form>
  {#if claraStore.isSearching}
    <p class="search-status">検索中...</p>
  {/if}
  {#if claraStore.searchError}
    <p class="search-error">{claraStore.searchError}</p>
  {/if}
</div>

<style>
  .search-section {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid #ddd;
  }

  .search-bar {
    display: flex;
    gap: 0.3rem;
  }

  .search-bar input {
    flex: 1;
    padding: 0.35rem 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.8rem;
    box-sizing: border-box;
  }

  .search-bar input:focus {
    outline: none;
    border-color: #0366d6;
    box-shadow: 0 0 0 2px rgba(3, 102, 214, 0.15);
  }

  .search-bar input:disabled {
    background: #e9ecef;
  }

  .search-exec-btn,
  .search-clear-btn {
    background: #e8ebee;
    border: 1px solid #ccc;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0 0.4rem;
    flex-shrink: 0;
    width: auto;
  }

  .search-exec-btn:hover:not(:disabled),
  .search-clear-btn:hover {
    background: #ddd;
  }

  .search-exec-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .search-clear-btn {
    color: #888;
  }

  .search-clear-btn:hover {
    color: #e53e3e;
    border-color: #e53e3e;
  }

  .search-status {
    font-size: 0.7rem;
    color: #0366d6;
    margin: 0.3rem 0 0;
    animation: blink 1.5s infinite;
  }

  .search-error {
    font-size: 0.7rem;
    color: #e53e3e;
    margin: 0.3rem 0 0;
    line-height: 1.3;
    word-break: break-all;
  }

  @keyframes blink {
    50% { opacity: 0.5; }
  }
</style>
