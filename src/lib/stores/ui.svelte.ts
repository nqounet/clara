export type ModalKey = 'vault' | 'workspace' | 'cli' | 'model';

export class UiStore {
  activeModal = $state<ModalKey | null>(null);
  errorMsg = $state("");
  fontSize = $state(16);

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
