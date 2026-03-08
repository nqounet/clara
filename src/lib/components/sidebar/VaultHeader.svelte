<script lang="ts">
  import { claraStore } from "$lib/clara.svelte";
</script>

<div
  class="vault-header"
  role="button"
  tabindex="0"
  onclick={() => claraStore.openModal('vault')}
  onkeydown={(e) => e.key === 'Enter' && claraStore.openModal('vault')}
>
  <div class="vault-label">🏛️ Vault</div>
  <div class="vault-path" title={claraStore.rootDir}>
    {claraStore.rootDir || "読み込み中..."}
  </div>
  <div class="vault-hint">クリックして変更</div>
</div>

<style>
  .vault-header {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #ddd;
    background: #edf0f3;
    cursor: pointer;
    transition: background 0.15s;
  }

  .vault-header:hover {
    background: #e0e5ea;
  }

  .vault-label {
    font-size: 0.7rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.2rem;
  }

  .vault-path {
    font-size: 0.8rem;
    color: #333;
    font-weight: 600;
    word-break: break-all;
    margin-bottom: 0.25rem;
    line-height: 1.3;
  }

  .vault-hint {
    font-size: 0.65rem;
    color: #aaa;
    transition: color 0.15s;
  }

  .vault-header:hover .vault-hint {
    color: #0366d6;
  }
</style>
