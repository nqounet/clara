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
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.7rem;
    font-weight: 600;
    width: auto;
    line-height: 1;
  }

  .font-btn:hover {
    background: var(--scrollbar-hover);
    color: var(--text-primary);
  }

  .font-size-label {
    font-size: 0.65rem;
    color: var(--text-muted);
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
    color: var(--text-secondary);
    background: var(--bg-elevated);
    padding: 0.15rem 0.45rem;
    border-radius: 12px;
    font-family: monospace;
    border: 1px solid transparent;
    cursor: pointer;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
    width: auto;
  }

  .cli-info-btn:hover {
    background: var(--accent-blue-bg);
    color: var(--accent-blue);
    border-color: var(--accent-blue);
  }

  .status-indicator {
    color: var(--accent-green);
    font-size: 0.7rem;
    animation: blink 1.5s infinite;
  }

  @keyframes blink {
    50% { opacity: 0.5; }
  }

  .send-btn {
    background: var(--accent-green);
    color: var(--bg-base);
    border: none;
    padding: 0.3rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 600;
    white-space: nowrap;
    width: auto;
    flex-shrink: 0;
  }

  .send-btn:disabled {
    background: var(--bg-elevated);
    color: var(--text-muted);
    cursor: not-allowed;
  }

  .send-btn:hover:not(:disabled) {
    filter: brightness(90%);
  }

  .send-btn-yolo {
    background: var(--accent-amber);
    color: var(--bg-base);
  }

  .send-btn-yolo:hover:not(:disabled) {
    filter: brightness(90%);
  }

  .yolo-toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.7rem;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--bg-surface);
    white-space: nowrap;
    transition: all 0.2s;
    user-select: none;
    flex-shrink: 0;
  }

  .yolo-toggle:hover {
    border-color: var(--accent-amber);
    color: var(--accent-amber);
  }

  .yolo-toggle.yolo-active {
    background: var(--accent-amber-bg);
    border-color: var(--accent-amber);
    color: var(--accent-amber);
    font-weight: 600;
  }

  .yolo-toggle input[type="checkbox"] {
    accent-color: var(--accent-amber);
    margin: 0;
    cursor: pointer;
  }

  .yolo-toggle input[type="checkbox"]:disabled {
    cursor: not-allowed;
  }
</style>
