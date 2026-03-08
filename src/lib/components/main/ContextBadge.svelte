<script lang="ts">
  import { atomStore } from "$lib/stores";
</script>

{#if atomStore.lastAtom}
  <div class="context-badge">
    <div class="context-info">
      <span class="context-label">🔗 リンク先:</span>
      <span class="context-title">{atomStore.lastAtom.frontmatter.title}</span>
    </div>
    <button class="unlink-btn" onclick={() => atomStore.clearContext()} title="コンテキストリンクを解除して新規として扱う">
      ✖️ 解除
    </button>
  </div>
{:else}
  <div class="context-badge empty-context">
    <span class="context-label">✨ 新規の独立した思考として開始します</span>
  </div>
{/if}

<style>
  .context-badge {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.3rem 0.5rem;
    background: #e8f4fd;
    border-radius: 4px;
    margin-bottom: 0.4rem;
    font-size: 0.8rem;
  }

  .context-badge.empty-context {
    background: #f0f4f8;
    color: #888;
  }

  .context-info {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
  }

  .context-label {
    white-space: nowrap;
    color: #555;
  }

  .context-title {
    font-weight: 600;
    color: #0366d6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .unlink-btn {
    background: transparent;
    border: 1px solid #ccc;
    color: #888;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.7rem;
    white-space: nowrap;
    width: auto;
  }

  .unlink-btn:hover {
    background: #fee;
    border-color: #e53e3e;
    color: #e53e3e;
  }
</style>
