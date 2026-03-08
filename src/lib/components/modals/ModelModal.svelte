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
    color: #888;
    margin: 0 0 0.75rem;
    line-height: 1.4;
  }

  input[type="text"] {
    width: 100%;
    padding: 0.45rem 0.6rem;
    border: 1px solid #ccc;
    border-radius: 6px;
    box-sizing: border-box;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
  }

  input[type="text"]:focus {
    outline: none;
    border-color: #0366d6;
    box-shadow: 0 0 0 2px rgba(3, 102, 214, 0.15);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  .modal-cancel {
    background: transparent;
    border: 1px solid #ccc;
    color: #666;
    padding: 0.35rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    width: auto;
  }

  .modal-cancel:hover {
    background: #f0f0f0;
  }

  .modal-save {
    background: #0366d6;
    color: white;
    border: none;
    padding: 0.35rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    width: auto;
  }

  .modal-save:hover:not(:disabled) {
    background: #0256b9;
  }

  .modal-save:disabled {
    background: #a0c4e8;
    cursor: not-allowed;
  }

  .settings-msg {
    color: #28a745;
    font-weight: 600;
    font-size: 0.78rem;
    margin: 0.35rem 0 0;
    line-height: 1.4;
  }

  .settings-msg-error {
    color: #e53e3e;
  }
</style>
