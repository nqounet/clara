<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { claraStore } from "$lib/clara.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import MainContent from "$lib/components/main/MainContent.svelte";
  import VaultModal from "$lib/components/modals/VaultModal.svelte";
  import WorkspaceModal from "$lib/components/modals/WorkspaceModal.svelte";
  import CliModal from "$lib/components/modals/CliModal.svelte";
  import ModelModal from "$lib/components/modals/ModelModal.svelte";

  onMount(() => {
    claraStore.init();

    // グローバルEscapeキーでモーダルを閉じる
    const handleGlobalKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && claraStore.activeModal) {
        claraStore.closeModal();
      }
    };
    document.addEventListener('keydown', handleGlobalKeydown);

    return () => {
      document.removeEventListener('keydown', handleGlobalKeydown);
      claraStore.destroy();
    };
  });
</script>

<div class="app-layout">
  <Sidebar />
  <MainContent />
</div>

<!-- ═══ Modals ═══ -->
{#if claraStore.activeModal === 'vault'}
  <VaultModal />
{:else if claraStore.activeModal === 'workspace'}
  <WorkspaceModal />
{:else if claraStore.activeModal === 'cli'}
  <CliModal />
{:else if claraStore.activeModal === 'model'}
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
