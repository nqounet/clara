import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ClaraAtom, ClaraFrontmatter } from '$lib/types/clara';

export class AtomStore {
  prompt = $state("");
  isSending = $state(false);
  streamingResponse = $state("");
  lastAtom = $state<ClaraAtom | null>(null);
  
  recentAtoms = $state<ClaraFrontmatter[]>([]);
  isLoadingRecent = $state(false);

  private unlistenStreaming: UnlistenFn | null = null;

  async setupStreamingListener() {
    this.unlistenStreaming = await listen<string>("streaming-response", (event) => {
      if (this.isSending) {
        this.streamingResponse += event.payload;
      }
    });
  }

  destroy() {
    if (this.unlistenStreaming) {
      this.unlistenStreaming();
      this.unlistenStreaming = null;
    }
  }

  async fetchRecentAtoms() {
    this.isLoadingRecent = true;
    try {
      this.recentAtoms = await invoke<ClaraFrontmatter[]>("list_recent_atoms", { limit: 10 });
    } catch (e) {
      console.error("履歴の取得に失敗しました", e);
      throw e;
    } finally {
      this.isLoadingRecent = false;
    }
  }

  async loadAtom(id: string) {
    try {
      this.lastAtom = await invoke<ClaraAtom>("load_atom", { id });
    } catch (e) {
      console.error(`読み込みエラー: ${e}`);
      throw e;
    }
  }

  clearContext() {
    this.lastAtom = null;
  }

  async handleSend(yoloMode: boolean): Promise<{ success: boolean; errorMsg?: string }> {
    if (this.isSending) return { success: false };
    if (!this.prompt.trim()) {
      return { success: false, errorMsg: "プロンプトは必須です。" };
    }

    this.isSending = true;
    this.streamingResponse = "";

    try {
      const result: ClaraAtom = await invoke("create_and_send_prompt", {
        description: null,
        prompt: this.prompt.trim(),
        parentId: this.lastAtom?.frontmatter.id || null,
        yolo: yoloMode,
      });
      this.lastAtom = result;
      this.prompt = "";
      await this.fetchRecentAtoms();
      return { success: true };
    } catch (e) {
      return { success: false, errorMsg: String(e) };
    } finally {
      this.isSending = false;
    }
  }
}
