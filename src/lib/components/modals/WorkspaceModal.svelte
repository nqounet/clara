<script lang="ts">
  import { configStore, uiStore, closeModal } from "$lib/stores";
  import ModalOverlay from "./ModalOverlay.svelte";
  import { onMount } from "svelte";

  let tempWorkingDir = $state(configStore.cliWorkingDir);
  let inputElement: HTMLInputElement;
  let activeIndex = $state(-1);

  onMount(() => {
    tempWorkingDir = configStore.cliWorkingDir;
    if (inputElement) {
      inputElement.focus();
      inputElement.select();
    }
  });

  let filteredHistory = $derived(
    configStore.workspaceHistory.filter(p => {
      const pattern = tempWorkingDir.toLowerCase().replace(/\s+/g, '');
      const text = p.toLowerCase();
      if (!pattern) return true;
      let j = 0;
      for (let i = 0; i < text.length && j < pattern.length; i++) {
        if (pattern[j] === text[i]) j++;
      }
      return j === pattern.length;
    })
  );

  // Reset activeIndex when filter changes
  $effect(() => {
    if (filteredHistory) {
      activeIndex = -1;
    }
  });

  function selectHistory(path: string) {
    tempWorkingDir = path;
    if (inputElement) {
      inputElement.focus();
    }
  }

  function removeHistory(path: string, e: Event) {
    e.stopPropagation();
    configStore.removeWorkspaceHistory(path);
  }

  async function handleSave() {
    configStore.cliWorkingDir = tempWorkingDir;
    await configStore.handleUpdateClaraConfig(() => uiStore.closeModalSilent());
  }

  async function handlePickDir() {
    const path = await configStore.pickFolder();
    if (path) {
      tempWorkingDir = path;
      if (inputElement) {
        inputElement.focus();
      }
    }
  }

  function handleClose() {
    configStore.claraSettingsMsg = '';
    configStore.claraSettingsMsgIsError = false;
    closeModal();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (filteredHistory.length > 0) {
        activeIndex = (activeIndex + 1) % filteredHistory.length;
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (filteredHistory.length > 0) {
        activeIndex = activeIndex <= 0 ? filteredHistory.length - 1 : activeIndex - 1;
      }
    } else if (e.key === 'Enter') {
      if (activeIndex >= 0 && activeIndex < filteredHistory.length) {
        e.preventDefault();
        selectHistory(filteredHistory[activeIndex]);
        activeIndex = -1;
      } else {
        handleSave();
      }
    } else if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<ModalOverlay title="Workspace" onclose={handleClose}>
  {#snippet children()}
    <p class="modal-desc">CLIを実行するディレクトリを設定します。必須項目です。</p>
    <div class="path-row">
      <input
        bind:this={inputElement}
        type="text"
        bind:value={tempWorkingDir}
        onkeydown={handleKeydown}
        placeholder="CLIの作業ディレクトリ (履歴から検索・選択できます)"
      />
      <button class="pick-btn" onclick={handlePickDir} type="button">📂</button>
    </div>

    {#if configStore.workspaceHistory.length > 0}
      <div class="history-list">
        {#each filteredHistory as path, i}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="history-item" class:active={i === activeIndex} onclick={() => selectHistory(path)}>
            <span class="history-path" title={path}>{path}</span>
            <button class="history-delete" onclick={(e) => removeHistory(path, e)} type="button" title="削除">✕</button>
          </div>
        {/each}
        {#if filteredHistory.length === 0}
          <div class="history-empty">一致する履歴がありません</div>
        {/if}
      </div>
    {/if}

    {#if configStore.claraSettingsMsg}
      <p class="settings-msg" class:settings-msg-error={configStore.claraSettingsMsgIsError}>{configStore.claraSettingsMsg}</p>
    {/if}
    <div class="modal-actions">
      <button class="modal-cancel" onclick={handleClose} type="button">キャンセル</button>
      <button class="modal-save" onclick={handleSave} disabled={configStore.isSaving} type="button">保存</button>
    </div>
  {/snippet}
</ModalOverlay>

<style>
  .modal-desc {
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin: 0 0 0.75rem;
    line-height: 1.4;
  }

  input[type="text"] {
    width: 100%;
    padding: 0.45rem 0.6rem;
    background: var(--bg-base);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-sizing: border-box;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
  }

  input[type="text"]:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .history-list {
    margin-top: 0.5rem;
    max-height: 200px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-base);
  }

  .history-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem 0.6rem;
    border-bottom: 1px solid var(--bg-surface);
    cursor: pointer;
    transition: background 0.15s;
  }

  .history-item:last-child {
    border-bottom: none;
  }

  .history-item:hover, .history-item.active {
    background: var(--bg-surface);
  }

  .history-path {
    font-size: 0.8rem;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    margin-right: 0.5rem;
  }

  .history-delete {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0 0.3rem;
    line-height: 1;
    border-radius: 4px;
  }

  .history-delete:hover {
    color: var(--accent-red);
    background: var(--accent-red-bg);
  }

  .history-empty {
    padding: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-muted);
    text-align: center;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .modal-cancel {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 0.35rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    width: auto;
  }

  .modal-cancel:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .modal-save {
    background: var(--accent-blue);
    color: var(--text-primary);
    border: none;
    padding: 0.35rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    width: auto;
  }

  .modal-save:hover:not(:disabled) {
    background: #2563EB;
  }

  .modal-save:disabled {
    background: var(--bg-elevated);
    color: var(--text-muted);
    cursor: not-allowed;
  }

  .path-row {
    display: flex;
    gap: 0.35rem;
    margin-bottom: 0.35rem;
  }

  .path-row input {
    flex: 1;
    margin-bottom: 0 !important;
  }

  .pick-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    font-size: 1rem;
    padding: 0 0.5rem;
    flex-shrink: 0;
  }

  .pick-btn:hover {
    background: var(--scrollbar-hover);
  }

  .settings-msg {
    color: var(--accent-green);
    font-weight: 600;
    font-size: 0.78rem;
    margin: 0.35rem 0 0;
    line-height: 1.4;
  }

  .settings-msg-error {
    color: var(--accent-red);
  }
</style>
