export type ModalKey = 'vault' | 'workspace' | 'cli' | 'model';
export type Theme = 'dark' | 'light';

export class UiStore {
  activeModal = $state<ModalKey | null>(null);
  errorMsg = $state("");
  fontSize = $state(16);
  theme = $state<Theme>('dark');

  constructor() {
    if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem('clara-theme') as Theme | null;
      if (saved === 'dark' || saved === 'light') {
        this.theme = saved;
      }
      document.body.dataset.theme = this.theme;
    }
  }

  toggleTheme() {
    this.theme = this.theme === 'dark' ? 'light' : 'dark';
    if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
      document.body.dataset.theme = this.theme;
      localStorage.setItem('clara-theme', this.theme);
    }
  }

  openModal(key: ModalKey) {
    this.activeModal = key;
  }

  closeModal() {
    this.activeModal = null;
  }

  closeModalSilent() {
    this.activeModal = null;
  }
}
