import type { SkrSearchResult } from '$lib/types/clara';
import { invoke } from '@tauri-apps/api/core';

export class SearchStore {
  searchQuery = $state("");
  searchResults = $state<SkrSearchResult[]>([]);
  isSearching = $state(false);
  hasSearched = $state(false);
  searchError = $state("");
  isSearchComposing = $state(false);

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
}
