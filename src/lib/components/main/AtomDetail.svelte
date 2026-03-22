<script lang="ts">
  import { atomStore } from "$lib/stores";
</script>

{#if atomStore.isSending}
  <div class="atom-detail">
    <div class="breadcrumb">⏳ 応答を生成中...</div>
    <div class="box">
      <h3>User</h3>
      <pre>{atomStore.prompt}</pre>
    </div>
    <div class="box">
      <h3>AI</h3>
      <pre>{atomStore.streamingResponse}</pre>
    </div>
  </div>
{:else if atomStore.lastAtom}
  <div class="atom-detail">
    {#if atomStore.lastAtom.frontmatter.parent_id}
      <div class="breadcrumb">
        🔗 親ノード: <code>{atomStore.lastAtom.frontmatter.parent_id}</code>
        <button
          class="nav-btn"
          onclick={() => atomStore.loadAtom(atomStore.lastAtom!.frontmatter.parent_id!)}
        >
          遡る
        </button>
      </div>
    {/if}
    <h2>
      {atomStore.lastAtom.frontmatter.title}
      <span class="id-text">(ID: {atomStore.lastAtom.frontmatter.id})</span>
    </h2>
    <div class="created-at">
      {new Date(atomStore.lastAtom.frontmatter.created_at).toLocaleString('ja-JP')}
    </div>
    <div class="tags">
      {#each atomStore.lastAtom.frontmatter.tags as tag}
        <span class="tag">#{tag}</span>
      {/each}
    </div>

    {#if atomStore.lastAtom.frontmatter.description}
      <p class="description"><strong>概要:</strong> {atomStore.lastAtom.frontmatter.description}</p>
    {/if}

    <div class="exec-meta">
      {#if atomStore.lastAtom.frontmatter.cli_command}
        <span class="exec-meta-item">⚡ {atomStore.lastAtom.frontmatter.cli_command}</span>
      {/if}
      {#if atomStore.lastAtom.frontmatter.model}
        <span class="exec-meta-item">🤖 {atomStore.lastAtom.frontmatter.model}</span>
      {/if}
      {#if atomStore.lastAtom.frontmatter.workspace}
        <span class="exec-meta-item">📂 {atomStore.lastAtom.frontmatter.workspace}</span>
      {/if}
      {#if atomStore.lastAtom.frontmatter.yolo}
        <span class="exec-meta-item exec-meta-yolo">🔥 YOLO</span>
      {/if}
    </div>

    <div class="box">
      <h3>User</h3>
      <pre>{atomStore.lastAtom.prompt}</pre>
    </div>
    <div class="box">
      <h3>AI</h3>
      <pre>{atomStore.lastAtom.response}</pre>
    </div>
  </div>
{:else}
  <div class="empty-state">
    <p>Atomストリップから選択するか、新しいメッセージを送信してください</p>
  </div>
{/if}

<style>
  .breadcrumb {
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
  }

  .breadcrumb code {
    font-size: 0.8rem;
    color: var(--text-primary);
  }

  .nav-btn {
    background: var(--accent-blue-bg);
    color: var(--accent-blue);
    border: 1px solid var(--accent-blue);
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.7rem;
    margin-left: 0.5rem;
    width: auto;
  }

  .nav-btn:hover {
    filter: brightness(120%);
  }

  .atom-detail h2 {
    margin: 0 0 0.5rem;
    font-size: 1.05rem;
    color: var(--text-primary);
  }

  .id-text {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-weight: normal;
  }

  .created-at {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.4rem;
  }

  .tags {
    margin-bottom: 0.5rem;
  }

  .tag {
    display: inline-block;
    background: var(--accent-blue-bg);
    color: var(--accent-blue);
    padding: 0.1rem 0.4rem;
    border-radius: 12px;
    font-size: 0.7rem;
    margin-right: 0.3rem;
  }

  .description {
    background: var(--accent-amber-bg);
    padding: 0.4rem 0.6rem;
    border-left: 3px solid var(--accent-amber);
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
  }

  .exec-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 0.5rem;
  }

  .exec-meta-item {
    font-size: 0.7rem;
    color: var(--text-secondary);
    background: var(--bg-elevated);
    padding: 0.15rem 0.4rem;
    border-radius: 12px;
    font-family: monospace;
  }

  .exec-meta-yolo {
    background: var(--accent-amber-bg);
    color: var(--accent-amber);
    font-weight: 600;
  }

  .box {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    padding: 0.75rem;
    border-radius: 4px;
    margin-bottom: 0.5rem;
  }

  .box h3 {
    margin: 0 0 0.4rem;
    font-size: 0.85rem;
    color: var(--accent-blue);
  }

  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
    font-family: monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    color: var(--text-primary);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 0.9rem;
  }
</style>
