<script lang="ts">
  import { claraStore } from "$lib/clara.svelte";
</script>

<div class="atom-list-section">
  {#if claraStore.hasSearched}
    <h3>検索結果 ({claraStore.searchResults.length}件)</h3>
    {#if claraStore.searchResults.length === 0 && !claraStore.isSearching}
      <p class="empty-msg">一致するAtomが見つかりませんでした</p>
    {:else}
      <ul class="recent-list">
        {#each claraStore.searchResults as atom}
          <li>
            <button
              class="atom-btn"
              class:atom-btn-active={claraStore.lastAtom?.frontmatter.id === atom.id}
              onclick={() => claraStore.loadAtom(atom.id)}
            >
              <span class="atom-title">{atom.title}</span>
              {#if atom.snippet}
                <span class="atom-desc">{atom.snippet}</span>
              {/if}
              <span class="atom-id">{atom.id.includes('-') ? atom.id.split('-')[0] : atom.id}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <h3>過去の思考 (Atom)</h3>
    {#if claraStore.isLoadingRecent}
      <p class="empty-msg">読み込み中...</p>
    {:else if claraStore.recentAtoms.length === 0}
      <p class="empty-msg">まだ履歴がありません</p>
    {:else}
      <ul class="recent-list">
        {#each claraStore.recentAtoms as atom}
          <li>
            <button
              class="atom-btn"
              class:atom-btn-active={claraStore.lastAtom?.frontmatter.id === atom.id}
              onclick={() => claraStore.loadAtom(atom.id)}
            >
              <span class="atom-title">{atom.title}</span>
              <span class="atom-id">{atom.id.includes('-') ? atom.id.split('-')[0] : atom.id}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .atom-list-section {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem 1rem;
  }

  .atom-list-section h3 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    color: #555;
    border-bottom: 1px solid #ddd;
    padding-bottom: 0.4rem;
  }

  .recent-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .recent-list li {
    margin-bottom: 0.35rem;
  }

  .atom-btn {
    width: 100%;
    text-align: left;
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
    padding: 0.4rem 0.5rem;
    cursor: pointer;
    transition: background 0.15s;
    display: flex;
    flex-direction: column;
  }

  .atom-btn:hover {
    background: #e1ecf4;
    border-color: #0366d6;
  }

  .atom-btn-active {
    background: #e1ecf4;
    border-color: #0366d6;
    border-left: 3px solid #0366d6;
  }

  .atom-title {
    font-weight: 600;
    font-size: 0.8rem;
    color: #333;
    margin-bottom: 0.1rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .atom-id {
    font-size: 0.65rem;
    color: #999;
  }

  .atom-desc {
    font-size: 0.7rem;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .empty-msg {
    color: #999;
    font-size: 0.8rem;
  }
</style>
