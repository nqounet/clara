import { describe, it, expect } from 'vitest';
import { SearchStore } from './search.svelte';

describe('SearchStore', () => {
  it('initializes with default values', () => {
    const store = new SearchStore();
    expect(store.searchQuery).toBe('');
    expect(store.searchResults).toEqual([]);
    expect(store.isSearching).toBe(false);
    expect(store.hasSearched).toBe(false);
    expect(store.searchError).toBe('');
    expect(store.isSearchComposing).toBe(false);
  });

  it('clearSearch resets search states', () => {
    const store = new SearchStore();
    store.searchQuery = 'test';
    store.searchResults = [{ id: '1', title: 'test', score: 1, snippet: 'test' }];
    store.hasSearched = true;
    store.searchError = 'error';

    store.clearSearch();

    expect(store.searchQuery).toBe('');
    expect(store.searchResults).toEqual([]);
    expect(store.hasSearched).toBe(false);
    expect(store.searchError).toBe('');
  });

  it('handles composition events', () => {
    const store = new SearchStore();
    store.handleSearchCompositionStart();
    expect(store.isSearchComposing).toBe(true);
    
    store.handleSearchCompositionEnd();
    expect(store.isSearchComposing).toBe(false);
  });

  it('handles escape key to clear search when not composing', () => {
    const store = new SearchStore();
    store.searchQuery = 'test';
    store.hasSearched = true;

    // Simulate keydown Escape
    store.handleSearchKeydown({ key: 'Escape', isComposing: false } as KeyboardEvent);
    expect(store.searchQuery).toBe('');
    expect(store.hasSearched).toBe(false);
  });

  it('does not clear search on escape if composing', () => {
    const store = new SearchStore();
    store.searchQuery = 'test';
    store.hasSearched = true;

    // Simulate keydown Escape while composing
    store.handleSearchKeydown({ key: 'Escape', isComposing: true } as KeyboardEvent);
    expect(store.searchQuery).toBe('test');
    expect(store.hasSearched).toBe(true);

    // Or if isSearchComposing is true
    store.isSearchComposing = true;
    store.handleSearchKeydown({ key: 'Escape', isComposing: false } as KeyboardEvent);
    expect(store.searchQuery).toBe('test');
  });
});
