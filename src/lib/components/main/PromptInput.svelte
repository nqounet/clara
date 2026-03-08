<script lang="ts">
  import { claraStore } from "$lib/clara.svelte";
</script>

{#if claraStore.yoloMode}
  <div class="yolo-warning-banner">
    ⚠️ YOLOモード有効: AIがファイル編集・コマンド実行を確認なしで実行します
  </div>
{/if}

<div class="textarea-wrapper">
  <textarea
    id="prompt"
    rows="5"
    bind:value={claraStore.prompt}
    onkeydown={(e) => claraStore.handleKeydown(e)}
    disabled={claraStore.isSending}
    class:textarea-yolo={claraStore.yoloMode}
    placeholder="AIに聞きたいことを入力してください... (⌘+Enter で送信)"
    style="font-size: {claraStore.fontSize}px"
  ></textarea>
</div>

{#if claraStore.errorMsg}
  <p class="error">{claraStore.errorMsg}</p>
{/if}

{#if claraStore.yoloSentMsg}
  <p class="yolo-sent-msg">{claraStore.yoloSentMsg}</p>
{/if}

<style>
  .yolo-warning-banner {
    background: #fff3e0;
    border: 1px solid #f59e0b;
    border-radius: 4px;
    padding: 0.35rem 0.6rem;
    font-size: 0.75rem;
    color: #92400e;
    font-weight: 600;
    margin-bottom: 0.4rem;
    animation: yoloPulse 2s ease-in-out infinite;
  }

  @keyframes yoloPulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .textarea-wrapper {
    margin-bottom: 0.4rem;
  }

  textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-family: inherit;
    box-sizing: border-box;
    resize: vertical;
    line-height: 1.5;
  }

  textarea:disabled {
    background-color: #e9ecef;
    cursor: not-allowed;
  }

  textarea.textarea-yolo {
    border-color: #f59e0b;
    box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.15);
  }

  .error {
    color: #e53e3e;
    font-weight: 600;
    font-size: 0.8rem;
    margin: 0.2rem 0;
  }

  .yolo-sent-msg {
    color: #d97706;
    font-weight: 600;
    font-size: 0.75rem;
    margin: 0.2rem 0;
    animation: fadeOut 4s ease-in-out forwards;
  }

  @keyframes fadeOut {
    0%, 80% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
