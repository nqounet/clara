<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { ClaraSet } from "$lib/types/clara";

  let prompt = "";
  let isSending = false;
  let errorMsg = "";
  let lastSet: ClaraSet | null = null;

  async function handleSend() {
    if (!prompt.trim() || isSending) {
      errorMsg = "プロンプトは必須です。";
      return;
    }

    isSending = true;
    errorMsg = "";

    try {
      const result: ClaraSet = await invoke("create_and_send_prompt", {
        description: null, // 将来的に入力欄を追加予定
        prompt: prompt.trim(),
        parentId: lastSet?.frontmatter.id || null, // 現在は直前のIDを親として送信
      });
      lastSet = result;
      prompt = ""; // プロンプトをクリアして次を入力しやすくする
    } catch (e) {
      errorMsg = String(e);
    } finally {
      isSending = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Cmd + Enter (Mac) または Ctrl + Enter (Windows/Linux) で送信
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault(); // デフォルトの改行を防ぐ
      handleSend();
    }
  }
</script>

<main class="container">
  <h1>CLARA (開発版)</h1>

  <div class="input-area">
    <div class="field">
      <label for="prompt">プロンプト (Cmd + Enter で送信)</label>
      <textarea
        id="prompt"
        rows="5"
        bind:value={prompt}
        on:keydown={handleKeydown}
        placeholder="AIに聞きたいことを入力してください..."
      ></textarea>
    </div>

    {#if errorMsg}
      <p class="error">{errorMsg}</p>
    {/if}

    <button on:click={handleSend} disabled={isSending}>
      {isSending ? "送信中..." : "送信 (Markdownを生成して保存)"}
    </button>
  </div>

  {#if lastSet}
    <div class="result-area">
      <h2>{lastSet.frontmatter.title} (ID: {lastSet.frontmatter.id})</h2>
      <div class="box">
        <h3>User (プロンプト)</h3>
        <pre>{lastSet.prompt}</pre>
      </div>
      <div class="box">
        <h3>AI (回答)</h3>
        <pre>{lastSet.response}</pre>
      </div>
    </div>
  {/if}
</main>

<style>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    font-family: sans-serif;
  }

  h1 {
    text-align: center;
    color: #333;
  }

  .input-area {
    background: #f9f9f9;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .field {
    margin-bottom: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: bold;
    color: #555;
  }

  input,
  textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-family: inherit;
    box-sizing: border-box;
  }

  button {
    background: #007bff;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border-radius: 4px;
    cursor: pointer;
    width: 100%;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .error {
    color: red;
    font-weight: bold;
  }

  .result-area {
    border-top: 2px solid #eee;
    padding-top: 2rem;
  }

  .box {
    background: #f1f8ff;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .box h3 {
    margin-top: 0;
    color: #0366d6;
  }

  pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
  }
</style>
