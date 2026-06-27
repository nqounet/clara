<script lang="ts">
  import { configStore, uiStore } from "$lib/stores";

  let shortPath = $derived(() => {
    const full = configStore.rootDir;
    if (!full) return "読み込み中...";
    const parts = full.split("/").filter(Boolean);
    return parts.length > 0 ? parts[parts.length - 1] : full;
  });
</script>

<button
  class="vault-btn"
  onclick={() => uiStore.openModal('vault')}
  title={configStore.rootDir}
>
  Vault: {shortPath()}
</button>

<style>
  .vault-btn {
    background: var(--bg-surface);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.25rem 0.6rem;
    font-size: 0.75rem;
    cursor: pointer;
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background 0.15s, color 0.15s;
    flex-shrink: 0;
  }

  .vault-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }
</style>
