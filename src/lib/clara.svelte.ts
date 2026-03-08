import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ClaraAtom, ClaraFrontmatter, AppConfig, ClaraConfig, SkrSearchResult } from "$lib/types/clara";

export type ModalKey = 'vault' | 'workspace' | 'cli' | 'model';

export class ClaraStore {
  prompt = $state("");
  isSending = $state(false);
  streamingResponse = $state("");
  errorMsg = $state("");
  lastAtom = $state<ClaraAtom | null>(null);
  yoloMode = $state(false);
  yoloSentMsg = $state("");
  appVersion = $state("");

  // Modal management
  activeModal = $state<ModalKey | null>(null);

  // ClaraConfig settings
  cliCommand = $state("gemini");
  cliModel = $state("");
  cliWorkingDir = $state("");
  claraSettingsMsg = $state("");
  claraSettingsMsgIsError = $state(false);

  // Vault Settings
  rootDir = $state("");
  vaultMsg = $state("");
  isVaultMsgError = $state(false);

  // Sidebar (Recent Atoms)
  recentAtoms = $state<ClaraFrontmatter[]>([]);
  isLoadingRecent = $state(false);

  // Search
  searchQuery = $state("");
  searchResults = $state<SkrSearchResult[]>([]);
  isSearching = $state(false);
  hasSearched = $state(false);
  searchError = $state("");
  isSearchComposing = $state(false);

  // Font size for textarea
  fontSize = $state(16);

  // 保存中フラグ（連打防止）
  isSaving = $state(false);

  private unlistenStreaming: UnlistenFn | null = null;

  async init() {
    try {
      const [appConfig, claraConfig, version] = await Promise.all([
        invoke<AppConfig>("get_app_config"),
        invoke<ClaraConfig>("get_clara_config"),
        getVersion(),
      ]);
      this.rootDir = appConfig.root_dir;
      this.cliCommand = claraConfig.cli_command ?? "gemini";
      this.cliModel = claraConfig.model ?? "";
      this.cliWorkingDir = claraConfig.working_dir ?? "";
      this.appVersion = version;
      await this.fetchRecentAtoms();

      this.unlistenStreaming = await listen<string>("streaming-response", (event) => {
        if (this.isSending) {
          this.streamingResponse += event.payload;
        }
      });
    } catch (e) {
      console.error("初期化に失敗しました", e);
    }
  }

  destroy() {
    if (this.unlistenStreaming) {
      this.unlistenStreaming();
      this.unlistenStreaming = null;
    }
  }

  openModal(key: ModalKey) {
    if (this.isSending) return;
    this.activeModal = key;
  }

  closeModal() {
    this.activeModal = null;
    // メッセージ状態をクリアして他モーダルへの漏れを防止
    this.claraSettingsMsg = "";
    this.claraSettingsMsgIsError = false;
    this.vaultMsg = "";
    this.isVaultMsgError = false;
  }

  /** タイマー経由で閉じる場合はメッセージをクリアしない（保存成功表示を保持） */
  closeModalSilent() {
    this.activeModal = null;
  }

  handleModalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') this.closeModal();
  }

  private async pickFolder(): Promise<string | null> {
    const selected = await openDialog({ directory: true, multiple: false });
    return typeof selected === "string" ? selected : null;
  }

  async pickVaultDir() {
    const path = await this.pickFolder();
    if (path) this.rootDir = path;
  }

  async pickWorkspaceDir() {
    const path = await this.pickFolder();
    if (path) this.cliWorkingDir = path;
  }

  async fetchRecentAtoms() {
    this.isLoadingRecent = true;
    try {
      this.recentAtoms = await invoke<ClaraFrontmatter[]>("list_recent_atoms", { limit: 10 });
    } catch (e) {
      console.error("履歴の取得に失敗しました", e);
    } finally {
      this.isLoadingRecent = false;
    }
  }

  async loadAtom(id: string) {
    try {
      this.lastAtom = await invoke<ClaraAtom>("load_atom", { id });
      const scrollArea = document.querySelector('.scroll-area');
      if (scrollArea) scrollArea.scrollTop = 0;
    } catch (e) {
      this.errorMsg = `読み込みエラー: ${e}`;
    }
  }

  clearContext() {
    this.lastAtom = null;
  }

  async handleUpdateRootDir() {
    if (this.isSaving) return;
    this.isSaving = true;
    try {
      this.vaultMsg = "切り替え中...";
      this.isVaultMsgError = false;
      const config = await invoke<AppConfig>("update_root_dir", { newPath: this.rootDir });
      this.rootDir = config.root_dir;
      this.vaultMsg = "✓ Vaultを切り替えました";

      this.lastAtom = null;
      this.prompt = "";
      this.errorMsg = "";
      this.clearSearch();
      await this.fetchRecentAtoms();

      setTimeout(() => {
        this.vaultMsg = "";
        this.closeModalSilent();
      }, 1500);
    } catch (e) {
      this.isVaultMsgError = true;
      this.vaultMsg = String(e);
    } finally {
      this.isSaving = false;
    }
  }

  async handleUpdateClaraConfig() {
    if (this.isSaving) return;
    this.isSaving = true;
    try {
      this.claraSettingsMsgIsError = false;
      this.claraSettingsMsg = "保存中...";
      const updated = await invoke<ClaraConfig>("update_clara_config", {
        cliCommand: this.cliCommand,
        model: this.cliModel.trim() || null,
        workingDir: this.cliWorkingDir.trim() || null,
      });
      this.cliCommand = updated.cli_command;
      this.cliModel = updated.model ?? "";
      this.cliWorkingDir = updated.working_dir ?? "";
      this.claraSettingsMsg = "✓ 保存しました";
      setTimeout(() => {
        this.claraSettingsMsg = "";
        this.closeModalSilent();
      }, 1500);
    } catch (e) {
      this.claraSettingsMsgIsError = true;
      this.claraSettingsMsg = String(e);
    } finally {
      this.isSaving = false;
    }
  }

  async handleSend() {
    if (this.isSending) return;
    if (!this.prompt.trim()) {
      this.errorMsg = "プロンプトは必須です。";
      return;
    }

    this.isSending = true;
    this.errorMsg = "";
    this.streamingResponse = "";

    try {
      const result: ClaraAtom = await invoke("create_and_send_prompt", {
        description: null,
        prompt: this.prompt.trim(),
        parentId: this.lastAtom?.frontmatter.id || null,
        yolo: this.yoloMode,
      });
      this.lastAtom = result;
      this.prompt = "";
      if (this.yoloMode) {
        this.yoloSentMsg = "🔥 YOLOモードで送信しました。次の送信もYOLOが有効です。";
        setTimeout(() => {
          this.yoloSentMsg = "";
        }, 4000);
      }
      await this.fetchRecentAtoms();
      const scrollArea = document.querySelector('.scroll-area');
      if (scrollArea) scrollArea.scrollTop = 0;
    } catch (e) {
      this.errorMsg = String(e);
    } finally {
      this.isSending = false;
    }
  }

  async handleSearch() {
    const q = this.searchQuery.trim();
    if (!q || this.isSearching) return;
    this.isSearching = true;
    this.hasSearched = true;
    this.searchResults = [];
    this.searchError = "";
    try {
      this.searchResults = await invoke<SkrSearchResult[]>("search_skr", { query: q });
    } catch (e) {
      console.error("検索に失敗しました", e);
      this.searchError = String(e);
    } finally {
      this.isSearching = false;
    }
  }

  clearSearch() {
    this.searchQuery = "";
    this.searchResults = [];
    this.hasSearched = false;
    this.searchError = "";
  }

  handleSearchCompositionStart() {
    this.isSearchComposing = true;
  }

  handleSearchCompositionEnd() {
    this.isSearchComposing = false;
  }

  handleSearchKeydown(e: KeyboardEvent) {
    if (e.isComposing || this.isSearchComposing) return;
    if (e.key === "Escape") {
      this.clearSearch();
    }
  }

  handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      this.handleSend();
    }
  }
}

export const claraStore = new ClaraStore();
