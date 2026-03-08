<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { uiStore, atomStore, initApp, closeModal } from "$lib/stores";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import MainContent from "$lib/components/main/MainContent.svelte";
  import VaultModal from "$lib/components/modals/VaultModal.svelte";
  import WorkspaceModal from "$lib/components/modals/WorkspaceModal.svelte";
  import CliModal from "$lib/components/modals/CliModal.svelte";
  import ModelModal from "$lib/components/modals/ModelModal.svelte";

  onMount(() => {
    initApp();

    // グローバルEscapeキーでモーダルを閉じる
    const handleGlobalKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && uiStore.activeModal) {
        closeModal();
      }
    };
    document.addEventListener('keydown', handleGlobalKeydown);

    return () => {
      document.removeEventListener('keydown', handleGlobalKeydown);
      atomStore.destroy();
    };
  });
</script>

<div class="app-layout">
  <Sidebar />
  <MainContent />
</div>

<!-- ═══ Modals ═══ -->
{#if uiStore.activeModal === 'vault'}
  <VaultModal />
{:else if uiStore.activeModal === 'workspace'}
  <WorkspaceModal />
{:else if uiStore.activeModal === 'cli'}
  <CliModal />
{:else if uiStore.activeModal === 'model'}
  <ModelModal />
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
</style>
