<script lang="ts">
  import { configStore, atomStore, uiStore, handleSend } from "$lib/stores";
</script>

<div class="bottom-bar">
  <div class="bottom-left">
    <div class="font-controls">
      <button class="font-btn" onclick={() => uiStore.fontSize = Math.max(10, uiStore.fontSize - 2)} title="文字を小さく">A−</button>
      <span class="font-size-label">{uiStore.fontSize}px</span>
      <button class="font-btn" onclick={() => uiStore.fontSize = Math.min(32, uiStore.fontSize + 2)} title="文字を大きく">A+</button>
    </div>
    <div class="cli-info">
      <button class="cli-info-btn" onclick={() => uiStore.openModal('cli')} title="CLIコマンドを変更">
        ⚡ {configStore.cliCommand || "gemini"}
      </button>
      <button class="cli-info-btn" onclick={() => uiStore.openModal('model')} title="モデルを変更">
        🤖 {configStore.cliModel || "(デフォルト)"}
      </button>
      {#if atomStore.isSending}
        <span class="status-indicator">AIが思考中...</span>
      {/if}
    </div>
  </div>
  <div class="bottom-right">
    <label class="yolo-toggle" class:yolo-active={configStore.yoloMode} for="yolo-checkbox" title="YOLOモード: AIがファイル編集・コマンド実行を確認なしで実行">
      <input id="yolo-checkbox" type="checkbox" bind:checked={configStore.yoloMode} disabled={atomStore.isSending} />
      {configStore.yoloMode ? '🔥' : '🔒'} YOLO
    </label>
    <button class="send-btn" class:send-btn-yolo={configStore.yoloMode} onclick={() => handleSend()} disabled={atomStore.isSending}>
      {atomStore.isSending ? "..." : "送信"}
    </button>
  </div>
</div>

<style>
  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .bottom-left, .bottom-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .font-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .font-btn {
    background: #e8ebee;
    border: 1px solid #ccc;
    color: #555;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.7rem;
    font-weight: 600;
    width: auto;
    line-height: 1;
  }

  .font-btn:hover {
    background: #ddd;
  }

  .font-size-label {
    font-size: 0.65rem;
    color: #888;
    min-width: 2.5rem;
    text-align: center;
  }

  .cli-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .cli-info-btn {
    font-size: 0.65rem;
    color: #888;
    background: #f0f0f0;
    padding: 0.15rem 0.45rem;
    border-radius: 12px;
    font-family: monospace;
    border: 1px solid transparent;
    cursor: pointer;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
    width: auto;
  }

  .cli-info-btn:hover {
    background: #e1ecf4;
    color: #0366d6;
    border-color: #0366d6;
  }

  .status-indicator {
    color: #0366d6;
    font-size: 0.7rem;
    animation: blink 1.5s infinite;
  }

  @keyframes blink {
    50% { opacity: 0.5; }
  }

  .send-btn {
    background: #007bff;
    color: white;
    border: none;
    padding: 0.3rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
    white-space: nowrap;
    width: auto;
    flex-shrink: 0;
  }

  .send-btn:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .send-btn:hover:not(:disabled) {
    background: #0069d9;
  }

  .send-btn-yolo {
    background: #f59e0b;
  }

  .send-btn-yolo:hover:not(:disabled) {
    background: #d97706;
  }

  .yolo-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.7rem;
    color: #888;
    cursor: pointer;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #ddd;
    background: #f8f8f8;
    white-space: nowrap;
    transition: all 0.2s;
    user-select: none;
    flex-shrink: 0;
  }

  .yolo-toggle:hover {
    border-color: #e8a020;
    color: #b07010;
  }

  .yolo-toggle.yolo-active {
    background: #fff3e0;
    border-color: #f59e0b;
    color: #d97706;
    font-weight: 600;
  }

  .yolo-toggle input[type="checkbox"] {
    accent-color: #f59e0b;
    margin: 0;
    cursor: pointer;
  }

  .yolo-toggle input[type="checkbox"]:disabled {
    cursor: not-allowed;
  }
</style>
