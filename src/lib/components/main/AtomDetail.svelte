<script lang="ts">
  import { atomStore } from "$lib/stores";
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';

  // Svelte 5 $derived to parse markdown and sanitize HTML for security (XSS prevention)
  const streamingResponseHtml = $derived(
    atomStore.streamingResponse ? DOMPurify.sanitize(marked.parse(atomStore.streamingResponse) as string) : ''
  );

  const responseHtml = $derived(
    atomStore.lastAtom?.response ? DOMPurify.sanitize(marked.parse(atomStore.lastAtom.response) as string) : ''
  );
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
      <div class="markdown-body">
        {@html streamingResponseHtml}
      </div>
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
      <div class="markdown-body">
        {@html responseHtml}
      </div>
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

  .markdown-body {
    font-size: 0.9rem;
    line-height: 1.6;
    color: var(--text-primary);
  }

  .markdown-body :global(p) {
    margin-top: 0;
    margin-bottom: 0.8rem;
  }

  .markdown-body :global(p:last-child) {
    margin-bottom: 0;
  }

  .markdown-body :global(h1),
  .markdown-body :global(h2),
  .markdown-body :global(h3),
  .markdown-body :global(h4) {
    margin-top: 1.2rem;
    margin-bottom: 0.6rem;
    font-weight: 600;
    line-height: 1.25;
    color: var(--text-primary);
  }

  .markdown-body :global(h1) { font-size: 1.4rem; border-bottom: 1px solid var(--border); padding-bottom: 0.3rem; }
  .markdown-body :global(h2) { font-size: 1.2rem; border-bottom: 1px solid var(--border); padding-bottom: 0.2rem; }
  .markdown-body :global(h3) { font-size: 1.05rem; }
  .markdown-body :global(h4) { font-size: 0.9rem; }

  .markdown-body :global(ul),
  .markdown-body :global(ol) {
    margin-top: 0;
    margin-bottom: 0.8rem;
    padding-left: 1.5rem;
  }

  .markdown-body :global(li) {
    margin-bottom: 0.25rem;
  }

  .markdown-body :global(code) {
    font-family: monospace;
    font-size: 0.85em;
    background: var(--bg-elevated);
    padding: 0.15rem 0.3rem;
    border-radius: 3px;
    color: var(--text-primary);
  }

  .markdown-body :global(pre) {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    padding: 0.8rem;
    border-radius: 4px;
    margin-top: 0;
    margin-bottom: 0.8rem;
    overflow-x: auto;
  }

  .markdown-body :global(pre code) {
    background: none;
    padding: 0;
    font-size: 0.85rem;
    border-radius: 0;
    color: inherit;
  }

  .markdown-body :global(blockquote) {
    margin: 0 0 0.8rem;
    padding: 0 1rem;
    color: var(--text-secondary);
    border-left: 0.25rem solid var(--border);
  }

  .markdown-body :global(a) {
    color: var(--accent-blue);
    text-decoration: none;
  }

  .markdown-body :global(a:hover) {
    text-decoration: underline;
  }

  .markdown-body :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 0.8rem;
    font-size: 0.85rem;
  }

  .markdown-body :global(th),
  .markdown-body :global(td) {
    border: 1px solid var(--border);
    padding: 0.4rem 0.6rem;
  }

  .markdown-body :global(th) {
    background: var(--bg-elevated);
    font-weight: 600;
  }
</style>
