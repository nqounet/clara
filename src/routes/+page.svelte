<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";
  import type { ClaraAtom, ClaraFrontmatter, AppConfig, ClaraConfig, SkrSearchResult } from "$lib/types/clara";

  let prompt = "";
  let isSending = false;
  let errorMsg = "";
  let lastAtom: ClaraAtom | null = null;
  let yoloMode = false;
  let yoloSentMsg = "";

  // Modal management
  type ModalKey = 'vault' | 'workspace' | 'cli' | 'model';
  let activeModal: ModalKey | null = null;

  // ClaraConfig settings
  let cliCommand = "gemini";
  let cliModel = "";
  let cliWorkingDir = "";
  let claraSettingsMsg = "";
  let claraSettingsMsgIsError = false;

  // Vault Settings
  let rootDir = "";
  let vaultMsg = "";
  let isVaultMsgError = false;

  // Sidebar (Recent Atoms)
  let recentAtoms: ClaraFrontmatter[] = [];
  let isLoadingRecent = false;

  // Search
  let searchQuery = "";
  let searchResults: SkrSearchResult[] = [];
  let isSearching = false;
  let hasSearched = false;
  let searchError = "";

  // Font size for textarea
  let fontSize = 16;

  // 保存中フラグ（連打防止）
  let isSaving = false;

  onMount(async () => {
    try {
      const [appConfig, claraConfig] = await Promise.all([
        invoke<AppConfig>("get_app_config"),
        invoke<ClaraConfig>("get_clara_config"),
      ]);
      rootDir = appConfig.root_dir;
      cliCommand = claraConfig.cli_command ?? "gemini";
      cliModel = claraConfig.model ?? "";
      cliWorkingDir = claraConfig.working_dir ?? "";
      await fetchRecentAtoms();
    } catch (e) {
      console.error("初期化に失敗しました", e);
    }
  });

  // グローバルEscapeキーでモーダルを閉じる
  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && activeModal) closeModal();
  }
  onMount(() => {
    document.addEventListener('keydown', handleGlobalKeydown);
  });
  onDestroy(() => {
    if (typeof document !== 'undefined') {
      document.removeEventListener('keydown', handleGlobalKeydown);
    }
  });

  function openModal(key: ModalKey) {
    if (isSending) return;
    activeModal = key;
  }

  function closeModal() {
    activeModal = null;
    // メッセージ状態をクリアして他モーダルへの漏れを防止
    claraSettingsMsg = "";
    claraSettingsMsgIsError = false;
    vaultMsg = "";
    isVaultMsgError = false;
  }

  /** タイマー経由で閉じる場合はメッセージをクリアしない（保存成功表示を保持） */
  function closeModalSilent() {
    activeModal = null;
  }

  function handleModalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') closeModal();
  }

  async function pickFolder(): Promise<string | null> {
    const selected = await openDialog({ directory: true, multiple: false });
    return typeof selected === "string" ? selected : null;
  }

  async function pickVaultDir() {
    const path = await pickFolder();
    if (path) rootDir = path;
  }

  async function pickWorkspaceDir() {
    const path = await pickFolder();
    if (path) cliWorkingDir = path;
  }

  async function fetchRecentAtoms() {
    isLoadingRecent = true;
    try {
      recentAtoms = await invoke("list_recent_atoms", { limit: 10 });
    } catch (e) {
      console.error("履歴の取得に失敗しました", e);
    } finally {
      isLoadingRecent = false;
    }
  }

  async function loadAtom(id: string) {
    try {
      lastAtom = await invoke("load_atom", { id });
      const scrollArea = document.querySelector('.scroll-area');
      if (scrollArea) scrollArea.scrollTop = 0;
    } catch (e) {
      errorMsg = `読み込みエラー: ${e}`;
    }
  }

  function clearContext() {
    lastAtom = null;
  }

  async function handleUpdateRootDir() {
    if (isSaving) return;
    isSaving = true;
    try {
      vaultMsg = "切り替え中...";
      isVaultMsgError = false;
      const config = await invoke<AppConfig>("update_root_dir", { newPath: rootDir });
      rootDir = config.root_dir;
      vaultMsg = "✓ Vaultを切り替えました";

      lastAtom = null;
      prompt = "";
      errorMsg = "";
      clearSearch();
      await fetchRecentAtoms();

      setTimeout(() => { vaultMsg = ""; closeModalSilent(); }, 1500);
    } catch (e) {
      isVaultMsgError = true;
      vaultMsg = String(e);
    } finally {
      isSaving = false;
    }
  }

  async function handleUpdateClaraConfig() {
    if (isSaving) return;
    isSaving = true;
    try {
      claraSettingsMsgIsError = false;
      claraSettingsMsg = "保存中...";
      const updated = await invoke<ClaraConfig>("update_clara_config", {
        cliCommand,
        model: cliModel.trim() || null,
        workingDir: cliWorkingDir.trim() || null,
      });
      cliCommand = updated.cli_command;
      cliModel = updated.model ?? "";
      cliWorkingDir = updated.working_dir ?? "";
      claraSettingsMsg = "✓ 保存しました";
      setTimeout(() => { claraSettingsMsg = ""; closeModalSilent(); }, 1500);
    } catch (e) {
      claraSettingsMsgIsError = true;
      claraSettingsMsg = String(e);
    } finally {
      isSaving = false;
    }
  }

  async function handleSend() {
    if (isSending) return;
    if (!prompt.trim()) {
      errorMsg = "プロンプトは必須です。";
      return;
    }

    isSending = true;
    errorMsg = "";

    try {
      const result: ClaraAtom = await invoke("create_and_send_prompt", {
        description: null,
        prompt: prompt.trim(),
        parentId: lastAtom?.frontmatter.id || null,
        yolo: yoloMode,
      });
      lastAtom = result;
      prompt = "";
      if (yoloMode) {
        yoloSentMsg = "🔥 YOLOモードで送信しました。次の送信もYOLOが有効です。";
        setTimeout(() => { yoloSentMsg = ""; }, 4000);
      }
      await fetchRecentAtoms();
      const scrollArea = document.querySelector('.scroll-area');
      if (scrollArea) scrollArea.scrollTop = 0;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      isSending = false;
    }
  }

  async function handleSearch() {
    const q = searchQuery.trim();
    if (!q || isSearching) return;
    isSearching = true;
    hasSearched = true;
    searchResults = [];
    searchError = "";
    try {
      searchResults = await invoke<SkrSearchResult[]>("search_skr", { query: q });
    } catch (e) {
      console.error("検索に失敗しました", e);
      searchError = String(e);
    } finally {
      isSearching = false;
    }
  }

  function clearSearch() {
    searchQuery = "";
    searchResults = [];
    hasSearched = false;
    searchError = "";
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleSearch();
    } else if (e.key === "Escape") {
      clearSearch();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleSend();
    }
  }
</script>

<div class="app-layout">
  <aside class="sidebar">
    <div class="vault-header" role="button" tabindex="0" on:click={() => openModal('vault')} on:keydown={(e) => e.key === 'Enter' && openModal('vault')}>
      <div class="vault-label">🏛️ Vault</div>
      <div class="vault-path" title={rootDir}>{rootDir || "読み込み中..."}</div>
      <div class="vault-hint">クリックして変更</div>
    </div>

    <div class="search-section">
      <div class="search-bar">
        <input
          type="text"
          bind:value={searchQuery}
          on:keydown={handleSearchKeydown}
          placeholder="Vaultを検索..."
          disabled={isSearching}
        />
        {#if searchQuery || hasSearched}
          <button class="search-clear-btn" on:click={clearSearch} title="検索をクリア">✕</button>
        {/if}
        <button class="search-exec-btn" on:click={handleSearch} disabled={isSearching || !searchQuery.trim()} title="検索">🔍</button>
      </div>
      {#if isSearching}
        <p class="search-status">検索中...</p>
      {/if}
      {#if searchError}
        <p class="search-error">{searchError}</p>
      {/if}
    </div>

    <div class="atom-list-section">
      {#if hasSearched}
        <h3>検索結果 ({searchResults.length}件)</h3>
        {#if searchResults.length === 0 && !isSearching}
          <p class="empty-msg">一致するAtomが見つかりませんでした</p>
        {:else}
          <ul class="recent-list">
            {#each searchResults as atom}
              <li>
                <button class="atom-btn" class:atom-btn-active={lastAtom?.frontmatter.id === atom.id} on:click={() => loadAtom(atom.id)}>
                  <span class="atom-title">{atom.title}</span>
                  {#if atom.snippet}
                    <span class="atom-desc">{atom.snippet}</span>
                  {/if}
                  <span class="atom-id">{atom.id.includes('-') ? atom.id.split('-')[0] : atom.id}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {:else}
        <h3>過去の思考 (Atom)</h3>
        {#if isLoadingRecent}
          <p class="empty-msg">読み込み中...</p>
        {:else if recentAtoms.length === 0}
          <p class="empty-msg">まだ履歴がありません</p>
        {:else}
          <ul class="recent-list">
            {#each recentAtoms as atom}
              <li>
                <button class="atom-btn" class:atom-btn-active={lastAtom?.frontmatter.id === atom.id} on:click={() => loadAtom(atom.id)}>
                  <span class="atom-title">{atom.title}</span>
                  <span class="atom-id">{atom.id.includes('-') ? atom.id.split('-')[0] : atom.id}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  </aside>

  <main class="main-content">
    <header class="header">
      <h1>CLARA</h1>
    </header>

    <div class="scroll-area">
      {#if lastAtom}
        <div class="atom-detail">
          {#if lastAtom.frontmatter.parent_id}
            <div class="breadcrumb">
              🔗 親ノード: <code>{lastAtom.frontmatter.parent_id}</code>
              <button class="nav-btn" on:click={() => loadAtom(lastAtom!.frontmatter.parent_id!)}>遡る</button>
            </div>
          {/if}
          <h2>{lastAtom.frontmatter.title} <span class="id-text">(ID: {lastAtom.frontmatter.id})</span></h2>
          <div class="created-at">{new Date(lastAtom.frontmatter.created_at).toLocaleString('ja-JP')}</div>
          <div class="tags">
            {#each lastAtom.frontmatter.tags as tag}
              <span class="tag">#{tag}</span>
            {/each}
          </div>

          {#if lastAtom.frontmatter.description}
            <p class="description"><strong>概要:</strong> {lastAtom.frontmatter.description}</p>
          {/if}

          <div class="exec-meta">
            {#if lastAtom.frontmatter.cli_command}
              <span class="exec-meta-item">⚡ {lastAtom.frontmatter.cli_command}</span>
            {/if}
            {#if lastAtom.frontmatter.model}
              <span class="exec-meta-item">🤖 {lastAtom.frontmatter.model}</span>
            {/if}
            {#if lastAtom.frontmatter.workspace}
              <span class="exec-meta-item">📂 {lastAtom.frontmatter.workspace}</span>
            {/if}
            {#if lastAtom.frontmatter.yolo}
              <span class="exec-meta-item exec-meta-yolo">🔥 YOLO</span>
            {/if}
          </div>

          <div class="box">
            <h3>User</h3>
            <pre>{lastAtom.prompt}</pre>
          </div>
          <div class="box">
            <h3>AI</h3>
            <pre>{lastAtom.response}</pre>
          </div>
        </div>
      {:else}
        <div class="empty-state">
          <p>左のリストからAtomを選択するか、新しいメッセージを送信してください</p>
        </div>
      {/if}
    </div>

    <div class="input-area">
      <button
        class="workspace-display-btn"
        class:workspace-empty={!cliWorkingDir}
        on:click={() => openModal('workspace')}
      >
        <div class="workspace-label">🖥️ Workspace:</div>
        <div class="workspace-path">
          {#if cliWorkingDir}
            <strong>{cliWorkingDir}</strong>
          {:else}
            <span class="workspace-unset">未設定</span>
          {/if}
        </div>
        <div class="workspace-hint">クリックして{#if cliWorkingDir}変更{:else}設定{/if}</div>
      </button>

      {#if lastAtom}
        <div class="context-badge">
          <div class="context-info">
            <span class="context-label">🔗 リンク先:</span>
            <span class="context-title">{lastAtom.frontmatter.title}</span>
          </div>
          <button class="unlink-btn" on:click={clearContext} title="コンテキストリンクを解除して新規として扱う">
            ✖️ 解除
          </button>
        </div>
      {:else}
        <div class="context-badge empty-context">
          <span class="context-label">✨ 新規の独立した思考として開始します</span>
        </div>
      {/if}

      {#if yoloMode}
        <div class="yolo-warning-banner">
          ⚠️ YOLOモード有効: AIがファイル編集・コマンド実行を確認なしで実行します
        </div>
      {/if}

      <div class="textarea-wrapper">
        <textarea
          id="prompt"
          rows="5"
          bind:value={prompt}
          on:keydown={handleKeydown}
          disabled={isSending}
          class:textarea-yolo={yoloMode}
          placeholder="AIに聞きたいことを入力してください... (⌘+Enter で送信)"
          style="font-size: {fontSize}px"
        ></textarea>
        <div class="font-controls">
          <button class="font-btn" on:click={() => fontSize = Math.max(10, fontSize - 2)} title="文字を小さく">A−</button>
          <span class="font-size-label">{fontSize}px</span>
          <button class="font-btn" on:click={() => fontSize = Math.min(32, fontSize + 2)} title="文字を大きく">A+</button>
        </div>
      </div>

      {#if errorMsg}
        <p class="error">{errorMsg}</p>
      {/if}

      {#if yoloSentMsg}
        <p class="yolo-sent-msg">{yoloSentMsg}</p>
      {/if}

      <div class="bottom-bar">
        <div class="cli-info">
          <button class="cli-info-btn" on:click={() => openModal('cli')} title="CLIコマンドを変更">
            ⚡ {cliCommand || "gemini"}
          </button>
          <button class="cli-info-btn" on:click={() => openModal('model')} title="モデルを変更">
            🤖 {cliModel || "(デフォルト)"}
          </button>
          {#if isSending}
            <span class="status-indicator">AIが思考中...</span>
          {/if}
        </div>
        <label class="yolo-toggle" class:yolo-active={yoloMode} for="yolo-checkbox" title="YOLOモード: AIがファイル編集・コマンド実行を確認なしで実行">
          <input id="yolo-checkbox" type="checkbox" bind:checked={yoloMode} disabled={isSending} />
          {yoloMode ? '🔥' : '🔒'} YOLO
        </label>
        <button class="send-btn" class:send-btn-yolo={yoloMode} on:click={handleSend} disabled={isSending}>
          {isSending ? "..." : "送信"}
        </button>
      </div>
    </div>
  </main>
</div>

<!-- ═══ Modals ═══ -->

{#if activeModal === 'vault'}
  <div class="modal-overlay" on:click={closeModal} on:keydown={handleModalKeydown} role="presentation">
    <div class="modal-body" on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1" aria-label="Vault設定">
      <h2>🏛️ Vault</h2>
      <p class="modal-desc">Atom を保存する Vault のパスを設定します。変更するとコンテキストはリセットされます。</p>
      <div class="path-row">
        <input type="text" bind:value={rootDir} placeholder="~/.clara/atoms" />
        <button class="pick-btn" on:click={pickVaultDir}>📂</button>
      </div>
      {#if vaultMsg}
        <p class="settings-msg" class:settings-msg-error={isVaultMsgError}>{vaultMsg}</p>
      {/if}
      <div class="modal-actions">
        <button class="modal-cancel" on:click={closeModal}>キャンセル</button>
        <button class="modal-save" on:click={handleUpdateRootDir} disabled={isSaving}>変更する</button>
      </div>
    </div>
  </div>
{/if}

{#if activeModal === 'workspace'}
  <div class="modal-overlay" on:click={closeModal} on:keydown={handleModalKeydown} role="presentation">
    <div class="modal-body" on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1" aria-label="Workspace設定">
      <h2>🖥️ Workspace</h2>
      <p class="modal-desc">CLIを実行するディレクトリを設定します。空欄にするとCLIのデフォルトが使われます。</p>
      <div class="path-row">
        <input type="text" bind:value={cliWorkingDir} placeholder="CLIの作業ディレクトリ" />
        <button class="pick-btn" on:click={pickWorkspaceDir}>📂</button>
      </div>
      {#if claraSettingsMsg}
        <p class="settings-msg" class:settings-msg-error={claraSettingsMsgIsError}>{claraSettingsMsg}</p>
      {/if}
      <div class="modal-actions">
        <button class="modal-clear" on:click={() => { cliWorkingDir = ""; handleUpdateClaraConfig(); }}>クリア</button>
        <button class="modal-cancel" on:click={closeModal}>キャンセル</button>
        <button class="modal-save" on:click={handleUpdateClaraConfig} disabled={isSaving}>保存</button>
      </div>
    </div>
  </div>
{/if}

{#if activeModal === 'cli'}
  <div class="modal-overlay" on:click={closeModal} on:keydown={handleModalKeydown} role="presentation">
    <div class="modal-body" on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1" aria-label="CLIコマンド設定">
      <h2>⚡ CLIコマンド</h2>
      <p class="modal-desc">AIに接続するCLIコマンドを設定します。</p>
      <input type="text" bind:value={cliCommand} placeholder="例: gemini" />
      {#if claraSettingsMsg}
        <p class="settings-msg" class:settings-msg-error={claraSettingsMsgIsError}>{claraSettingsMsg}</p>
      {/if}
      <div class="modal-actions">
        <button class="modal-cancel" on:click={closeModal}>キャンセル</button>
        <button class="modal-save" on:click={handleUpdateClaraConfig} disabled={isSaving}>保存</button>
      </div>
    </div>
  </div>
{/if}

{#if activeModal === 'model'}
  <div class="modal-overlay" on:click={closeModal} on:keydown={handleModalKeydown} role="presentation">
    <div class="modal-body" on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1" aria-label="モデル設定">
      <h2>🤖 モデル</h2>
      <p class="modal-desc">使用するAIモデルを指定します。空欄にするとCLIのデフォルトモデルが使われます。</p>
      <input type="text" bind:value={cliModel} placeholder="例: gemini-2.5-pro（空欄=CLIデフォルト）" />
      {#if claraSettingsMsg}
        <p class="settings-msg" class:settings-msg-error={claraSettingsMsgIsError}>{claraSettingsMsg}</p>
      {/if}
      <div class="modal-actions">
        <button class="modal-cancel" on:click={closeModal}>キャンセル</button>
        <button class="modal-save" on:click={handleUpdateClaraConfig} disabled={isSaving}>保存</button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    background: #fff;
  }

  .app-layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  /* ── Sidebar ── */
  .sidebar {
    width: 260px;
    background: #f4f6f8;
    border-right: 1px solid #ddd;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .vault-header {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #ddd;
    background: #edf0f3;
    cursor: pointer;
    transition: background 0.15s;
  }

  .vault-header:hover {
    background: #e0e5ea;
  }

  .vault-label {
    font-size: 0.7rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.2rem;
  }

  .vault-path {
    font-size: 0.8rem;
    color: #333;
    font-weight: 600;
    word-break: break-all;
    margin-bottom: 0.25rem;
    line-height: 1.3;
  }

  .vault-hint {
    font-size: 0.65rem;
    color: #aaa;
    transition: color 0.15s;
  }

  .vault-header:hover .vault-hint {
    color: #0366d6;
  }

  .search-section {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid #ddd;
  }

  .search-bar {
    display: flex;
    gap: 0.3rem;
  }

  .search-bar input {
    flex: 1;
    padding: 0.35rem 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.8rem;
    box-sizing: border-box;
  }

  .search-bar input:focus {
    outline: none;
    border-color: #0366d6;
    box-shadow: 0 0 0 2px rgba(3, 102, 214, 0.15);
  }

  .search-bar input:disabled {
    background: #e9ecef;
  }

  .search-exec-btn,
  .search-clear-btn {
    background: #e8ebee;
    border: 1px solid #ccc;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0 0.4rem;
    flex-shrink: 0;
    width: auto;
  }

  .search-exec-btn:hover:not(:disabled),
  .search-clear-btn:hover {
    background: #ddd;
  }

  .search-exec-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .search-clear-btn {
    color: #888;
  }

  .search-clear-btn:hover {
    color: #e53e3e;
    border-color: #e53e3e;
  }

  .search-status {
    font-size: 0.7rem;
    color: #0366d6;
    margin: 0.3rem 0 0;
    animation: blink 1.5s infinite;
  }

  .search-error {
    font-size: 0.7rem;
    color: #e53e3e;
    margin: 0.3rem 0 0;
    line-height: 1.3;
    word-break: break-all;
  }

  .atom-list-section {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem 1rem;
  }

  .atom-list-section h3 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    color: #555;
    border-bottom: 1px solid #ddd;
    padding-bottom: 0.4rem;
  }

  .recent-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .recent-list li {
    margin-bottom: 0.35rem;
  }

  .atom-btn {
    width: 100%;
    text-align: left;
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
    padding: 0.4rem 0.5rem;
    cursor: pointer;
    transition: background 0.15s;
    display: flex;
    flex-direction: column;
  }

  .atom-btn:hover {
    background: #e1ecf4;
    border-color: #0366d6;
  }

  .atom-btn-active {
    background: #e1ecf4;
    border-color: #0366d6;
    border-left: 3px solid #0366d6;
  }

  .atom-title {
    font-weight: 600;
    font-size: 0.8rem;
    color: #333;
    margin-bottom: 0.1rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .atom-id {
    font-size: 0.65rem;
    color: #999;
  }

  .atom-desc {
    font-size: 0.7rem;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .empty-msg {
    color: #999;
    font-size: 0.8rem;
  }

  /* ── Main content ── */
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    padding: 0.5rem 1.5rem;
    border-bottom: 1px solid #eee;
    flex-shrink: 0;
  }

  .header h1 {
    margin: 0;
    font-size: 1.1rem;
    color: #333;
  }

  /* ── Modal ── */
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal-body {
    background: #fff;
    border-radius: 10px;
    padding: 1.5rem;
    width: 90%;
    max-width: 420px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.18);
    animation: slideUp 0.2s ease-out;
  }

  @keyframes slideUp {
    from { transform: translateY(12px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .modal-body h2 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
    color: #333;
  }

  .modal-desc {
    font-size: 0.8rem;
    color: #888;
    margin: 0 0 0.75rem;
    line-height: 1.4;
  }

  .modal-body input[type="text"] {
    width: 100%;
    padding: 0.45rem 0.6rem;
    border: 1px solid #ccc;
    border-radius: 6px;
    box-sizing: border-box;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
  }

  .modal-body input[type="text"]:focus {
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

  .modal-clear {
    background: transparent;
    border: 1px solid #ccc;
    color: #888;
    padding: 0.35rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    width: auto;
    margin-right: auto;
  }

  .modal-clear:hover {
    border-color: #e53e3e;
    color: #e53e3e;
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
    background: #e8ebee;
    border: 1px solid #ccc;
    border-radius: 6px;
    cursor: pointer;
    font-size: 1rem;
    padding: 0 0.5rem;
    flex-shrink: 0;
  }

  .pick-btn:hover {
    background: #ddd;
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

  /* ── Scroll area ── */
  .scroll-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
  }

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

  /* ── Fixed bottom input area ── */
  .input-area {
    flex-shrink: 0;
    border-top: 1px solid #ddd;
    padding: 0.6rem 1.5rem;
    background: #fafbfc;
  }

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

  .context-badge {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.3rem 0.5rem;
    background: #e8f4fd;
    border-radius: 4px;
    margin-bottom: 0.4rem;
    font-size: 0.8rem;
  }

  .context-badge.empty-context {
    background: #f0f4f8;
    color: #888;
  }

  .context-info {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
  }

  .context-label {
    white-space: nowrap;
    color: #555;
  }

  .context-title {
    font-weight: 600;
    color: #0366d6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .unlink-btn {
    background: transparent;
    border: 1px solid #ccc;
    color: #888;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    cursor: pointer;
    font-size: 0.7rem;
    white-space: nowrap;
    width: auto;
  }

  .unlink-btn:hover {
    background: #fee;
    border-color: #e53e3e;
    color: #e53e3e;
  }

  .textarea-wrapper {
    margin-bottom: 0.4rem;
  }

  .font-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-top: 0.25rem;
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

  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
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

  /* ── YOLO toggle ── */
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