<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { uiStore, atomStore, initApp, closeModal } from "$lib/stores";
  import Header from "$lib/components/main/Header.svelte";
  import VaultHeader from "$lib/components/sidebar/VaultHeader.svelte";
  import SearchBar from "$lib/components/sidebar/SearchBar.svelte";
  import AtomList from "$lib/components/sidebar/AtomList.svelte";
  import ContextBadge from "$lib/components/main/ContextBadge.svelte";
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
  <header class="cockpit-header">
    <Header />
    <VaultHeader />
    <SearchBar />
    <ContextBadge />
    <button
      class="theme-switch"
      class:theme-light={uiStore.theme === 'light'}
      onclick={() => uiStore.toggleTheme()}
      title="テーマ切替"
      aria-label={uiStore.theme === 'dark' ? 'ライトモードに切替' : 'ダークモードに切替'}
    >
      <span class="ts-label sun">☀︎</span>
      <span class="ts-track">
        <span class="ts-thumb">{uiStore.theme === 'dark' ? '☽' : '☀︎'}</span>
      </span>
      <span class="ts-label moon">☽</span>
    </button>
  </header>
  <AtomList />
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
    background: var(--bg-base);
    color: var(--text-primary);
  }

  :global(body[data-theme="dark"]) {
    --bg-base: #0F172A;
    --bg-surface: #1E293B;
    --bg-elevated: #334155;
    --border: #334155;
    --text-primary: #F8FAFC;
    --text-secondary: #94A3B8;
    --text-muted: #64748B;
    --accent-green: #22C55E;
    --accent-blue: #3B82F6;
    --accent-blue-bg: rgba(59, 130, 246, 0.15);
    --accent-amber: #F59E0B;
    --accent-amber-bg: rgba(245, 158, 11, 0.1);
    --accent-red: #EF4444;
    --accent-red-bg: rgba(239, 68, 68, 0.1);
    --atom-active-bg: #1E3A5F;
    --atom-active-shadow: rgba(59, 130, 246, 0.3);
    --btn-send-text: #0F172A;
    --scrollbar-thumb: #334155;
    --scrollbar-hover: #475569;
    --modal-overlay: rgba(0, 0, 0, 0.6);
    --modal-shadow: rgba(0, 0, 0, 0.4);
  }

  :global(body[data-theme="light"]) {
    --bg-base: #F8FAFC;
    --bg-surface: #FFFFFF;
    --bg-elevated: #E2E8F0;
    --border: #CBD5E1;
    --text-primary: #0F172A;
    --text-secondary: #475569;
    --text-muted: #94A3B8;
    --accent-green: #16A34A;
    --accent-blue: #2563EB;
    --accent-blue-bg: rgba(37, 99, 235, 0.1);
    --accent-amber: #D97706;
    --accent-amber-bg: rgba(217, 119, 6, 0.08);
    --accent-red: #DC2626;
    --accent-red-bg: rgba(220, 38, 38, 0.08);
    --atom-active-bg: #DBEAFE;
    --atom-active-shadow: rgba(37, 99, 235, 0.2);
    --btn-send-text: #FFFFFF;
    --scrollbar-thumb: #CBD5E1;
    --scrollbar-hover: #94A3B8;
    --modal-overlay: rgba(0, 0, 0, 0.3);
    --modal-shadow: rgba(0, 0, 0, 0.1);
  }

  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .cockpit-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 1rem;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .theme-switch {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    flex-shrink: 0;
  }

  .ts-label {
    font-size: 1rem;
    line-height: 1;
    transition: opacity 0.2s;
    opacity: 0.3;
  }

  .ts-label.sun {
    color: var(--accent-amber);
  }

  .ts-label.moon {
    color: var(--accent-blue);
  }

  .theme-light .ts-label.sun {
    opacity: 1;
  }

  .theme-switch:not(.theme-light) .ts-label.moon {
    opacity: 1;
  }

  .ts-track {
    position: relative;
    width: 36px;
    height: 20px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    transition: background 0.2s;
  }

  .theme-switch:hover .ts-track {
    border-color: var(--accent-blue);
  }

  .ts-thumb {
    position: absolute;
    top: 1px;
    left: 1px;
    width: 16px;
    height: 16px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.65rem;
    line-height: 1;
    transition: transform 0.2s;
    transform: translateX(16px);
  }

  .theme-light .ts-thumb {
    transform: translateX(0);
  }

  .theme-switch:not(.theme-light) .ts-thumb {
    color: var(--accent-blue);
  }

  .theme-light .ts-thumb {
    color: var(--accent-amber);
  }
</style>
