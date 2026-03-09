<script lang="ts">
  import { configStore, uiStore } from "$lib/stores";
</script>

<button
  class="workspace-display-btn"
  class:workspace-empty={!configStore.cliWorkingDir}
  onclick={() => uiStore.openModal('workspace')}
>
  <div class="workspace-label">🖥️ Workspace:</div>
  <div class="workspace-path">
    {#if configStore.cliWorkingDir}
      <strong>{configStore.cliWorkingDir}</strong>
    {:else}
      <span class="workspace-unset">未設定</span>
    {/if}
  </div>
  <div class="workspace-hint">クリックして{#if configStore.cliWorkingDir}変更{:else}設定{/if}</div>
</button>

<style>
  .workspace-display-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    width: 100%;
    font-size: 0.8rem;
    color: #555;
    margin-bottom: 0.4rem;
    padding: 0.4rem 0.6rem;
    background: #f0f4f8;
    border-radius: 4px;
    border: 1px solid transparent;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s, border-color 0.15s;
  }

  .workspace-display-btn:hover {
    background: #e4eaf0;
    border-color: #ccc;
  }

  .workspace-display-btn.workspace-empty {
    color: #aaa;
    border: 1px dashed #ccc;
    background: transparent;
  }

  .workspace-display-btn.workspace-empty:hover {
    border-color: #0366d6;
    color: #555;
  }

  .workspace-label {
    font-weight: 600;
    font-size: 0.75rem;
  }

  .workspace-path {
    word-break: break-all;
  }

  .workspace-unset {
    color: #aaa;
    font-style: italic;
  }

  .workspace-hint {
    font-size: 0.65rem;
    color: #aaa;
    transition: color 0.15s;
  }

  .workspace-display-btn:hover .workspace-hint {
    color: #0366d6;
  }
</style>
