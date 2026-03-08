import { describe, it, expect } from 'vitest';
import { AtomStore } from './atom.svelte';

describe('AtomStore', () => {
  it('initializes with default values', () => {
    const store = new AtomStore();
    expect(store.prompt).toBe('');
    expect(store.isSending).toBe(false);
    expect(store.streamingResponse).toBe('');
    expect(store.lastAtom).toBeNull();
    expect(store.recentAtoms).toEqual([]);
    expect(store.isLoadingRecent).toBe(false);
  });

  it('clearContext resets lastAtom', () => {
    const store = new AtomStore();
    // @ts-ignore - mock lastAtom for test
    store.lastAtom = { prompt: 'test' };
    store.clearContext();
    expect(store.lastAtom).toBeNull();
  });
});
