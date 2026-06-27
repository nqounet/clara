<script lang="ts">
  import { atomStore } from "$lib/stores";
</script>

{#if atomStore.lastAtom}
  <div class="context-badge">
    <span class="context-label">🔗</span>
    <span class="context-title" title={atomStore.lastAtom.frontmatter.title}>{atomStore.lastAtom.frontmatter.title}</span>
    <button class="unlink-btn" onclick={() => atomStore.clearContext()} title="コンテキストリンクを解除して新規として扱う">
      ✖
    </button>
  </div>
{:else}
  <div class="context-badge empty-context">
    <span class="context-label">✨ 新規</span>
  </div>
{/if}

<style>
  .context-badge {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.2rem 0.5rem;
    background: var(--accent-blue-bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 0.75rem;
    flex-shrink: 0;
    max-width: 250px;
    margin-left: auto;
  }

  .context-badge.empty-context {
    background: transparent;
    border-color: var(--border);
    color: var(--text-muted);
  }

  .context-label {
    white-space: nowrap;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .context-title {
    font-weight: 600;
    color: var(--accent-blue);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .unlink-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    padding: 0;
    cursor: pointer;
    font-size: 0.65rem;
    white-space: nowrap;
    width: auto;
    flex-shrink: 0;
    line-height: 1;
  }

  .unlink-btn:hover {
    color: var(--accent-red);
  }
</style>
