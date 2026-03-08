<script lang="ts">
  import { claraStore } from "$lib/clara.svelte";
</script>

{#if claraStore.isSending}
  <div class="atom-detail">
    <div class="breadcrumb">⏳ 応答を生成中...</div>
    <div class="box">
      <h3>User</h3>
      <pre>{claraStore.prompt}</pre>
    </div>
    <div class="box">
      <h3>AI</h3>
      <pre>{claraStore.streamingResponse}</pre>
    </div>
  </div>
{:else if claraStore.lastAtom}
  <div class="atom-detail">
    {#if claraStore.lastAtom.frontmatter.parent_id}
      <div class="breadcrumb">
        🔗 親ノード: <code>{claraStore.lastAtom.frontmatter.parent_id}</code>
        <button
          class="nav-btn"
          onclick={() => claraStore.loadAtom(claraStore.lastAtom!.frontmatter.parent_id!)}
        >
          遡る
        </button>
      </div>
    {/if}
    <h2>
      {claraStore.lastAtom.frontmatter.title}
      <span class="id-text">(ID: {claraStore.lastAtom.frontmatter.id})</span>
    </h2>
    <div class="created-at">
      {new Date(claraStore.lastAtom.frontmatter.created_at).toLocaleString('ja-JP')}
    </div>
    <div class="tags">
      {#each claraStore.lastAtom.frontmatter.tags as tag}
        <span class="tag">#{tag}</span>
      {/each}
    </div>

    {#if claraStore.lastAtom.frontmatter.description}
      <p class="description"><strong>概要:</strong> {claraStore.lastAtom.frontmatter.description}</p>
    {/if}

    <div class="exec-meta">
      {#if claraStore.lastAtom.frontmatter.cli_command}
        <span class="exec-meta-item">⚡ {claraStore.lastAtom.frontmatter.cli_command}</span>
      {/if}
      {#if claraStore.lastAtom.frontmatter.model}
        <span class="exec-meta-item">🤖 {claraStore.lastAtom.frontmatter.model}</span>
      {/if}
      {#if claraStore.lastAtom.frontmatter.workspace}
        <span class="exec-meta-item">📂 {claraStore.lastAtom.frontmatter.workspace}</span>
      {/if}
      {#if claraStore.lastAtom.frontmatter.yolo}
        <span class="exec-meta-item exec-meta-yolo">🔥 YOLO</span>
      {/if}
    </div>

    <div class="box">
      <h3>User</h3>
      <pre>{claraStore.lastAtom.prompt}</pre>
    </div>
    <div class="box">
      <h3>AI</h3>
      <pre>{claraStore.lastAtom.response}</pre>
    </div>
  </div>
{:else}
  <div class="empty-state">
    <p>左のリストからAtomを選択するか、新しいメッセージを送信してください</p>
  </div>
{/if}

<style>
  .breadcrumb {
    font-size: 0.8rem;
    color: #666;
    margin-bottom: 0.5rem;
  }

  .breadcrumb code {
    font-size: 0.8rem;
  }

  .nav-btn {
    background: #e1ecf4;
    color: #0366d6;
    border: 1px solid #0366d6;
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.7rem;
    margin-left: 0.5rem;
    width: auto;
  }

  .atom-detail h2 {
    margin: 0 0 0.5rem;
    font-size: 1.05rem;
    color: #333;
  }

  .id-text {
    font-size: 0.8rem;
    color: #999;
    font-weight: normal;
  }

  .created-at {
    font-size: 0.75rem;
    color: #888;
    margin-bottom: 0.4rem;
  }

  .tags {
    margin-bottom: 0.5rem;
  }

  .tag {
    display: inline-block;
    background: #e1ecf4;
    color: #0366d6;
    padding: 0.1rem 0.4rem;
    border-radius: 12px;
    font-size: 0.7rem;
    margin-right: 0.3rem;
  }

  .description {
    background: #fffbdd;
    padding: 0.4rem 0.6rem;
    border-left: 3px solid #f6e05e;
    font-size: 0.85rem;
    color: #555;
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
    color: #666;
    background: #f0f0f0;
    padding: 0.15rem 0.4rem;
    border-radius: 12px;
    font-family: monospace;
  }

  .exec-meta-yolo {
    background: #fff3e0;
    color: #d97706;
    font-weight: 600;
  }

  .box {
    background: #f8fafb;
    border: 1px solid #e8edf1;
    padding: 0.75rem;
    border-radius: 4px;
    margin-bottom: 0.5rem;
  }

  .box h3 {
    margin: 0 0 0.4rem;
    font-size: 0.85rem;
    color: #0366d6;
  }

  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
    font-family: monospace;
    font-size: 0.85rem;
    line-height: 1.5;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #bbb;
    font-size: 0.9rem;
  }
</style>
