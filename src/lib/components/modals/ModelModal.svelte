<script lang="ts">
  import { configStore, uiStore, closeModal } from "$lib/stores";
  import ModalOverlay from "./ModalOverlay.svelte";
</script>

<ModalOverlay title="モデル" onclose={() => closeModal()}>
  {#snippet children()}
    <p class="modal-desc">使用するAIモデルを指定します。空欄にするとCLIのデフォルトモデルが使われます。</p>
    <input type="text" bind:value={configStore.cliModel} placeholder="例: gemini-2.5-pro（空欄=CLIデフォルト）" />
    {#if configStore.claraSettingsMsg}
      <p class="settings-msg" class:settings-msg-error={configStore.claraSettingsMsgIsError}>{configStore.claraSettingsMsg}</p>
    {/if}
    <div class="modal-actions">
      <button class="modal-cancel" onclick={() => closeModal()} type="button">キャンセル</button>
      <button class="modal-save" onclick={() => configStore.handleUpdateClaraConfig(() => uiStore.closeModalSilent())} disabled={configStore.isSaving} type="button">保存</button>
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
    filter: brightness(90%);
  }

  .modal-save:disabled {
    background: var(--bg-elevated);
    color: var(--text-muted);
    cursor: not-allowed;
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
