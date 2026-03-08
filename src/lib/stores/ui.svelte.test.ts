import { describe, it, expect } from 'vitest';
import { UiStore } from './ui.svelte';

describe('UiStore', () => {
  it('opens and closes modals', () => {
    const store = new UiStore();
    expect(store.activeModal).toBeNull();

    store.openModal('vault');
    expect(store.activeModal).toBe('vault');

    store.closeModal();
    expect(store.activeModal).toBeNull();
  });

  it('closeModalSilent closes without side effects', () => {
    const store = new UiStore();
    store.openModal('vault');
    store.closeModalSilent();
    expect(store.activeModal).toBeNull();
  });
});
