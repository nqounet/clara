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
    
    // Type-safe complete mock object
    const mockAtom: import('../types/clara').ClaraAtom = {
      frontmatter: {
        id: 'test-id',
        title: 'Test Atom',
        created_at: new Date().toISOString(),
        parent_id: null,
        tags: [],
      },
      prompt: 'test prompt',
      response: 'test response',
    };
    
    store.lastAtom = mockAtom;
    store.clearContext();
    expect(store.lastAtom).toBeNull();
  });
});
