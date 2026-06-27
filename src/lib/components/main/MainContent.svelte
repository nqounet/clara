<script lang="ts">
  import AtomDetail from "./AtomDetail.svelte";
  import WorkspaceSelector from "./WorkspaceSelector.svelte";
  import PromptInput from "./PromptInput.svelte";
  import ActionControls from "./ActionControls.svelte";
  import { atomStore } from "$lib/stores";

  let scrollArea: HTMLDivElement;

  $effect(() => {
    // This effect runs when a new atom is loaded or sent
    if (atomStore.lastAtom && scrollArea) {
      scrollArea.scrollTop = 0;
    }
  });
</script>

<main class="main-content">
  <div class="scroll-area" bind:this={scrollArea}>
    <AtomDetail />
  </div>

  <div class="input-area">
    <WorkspaceSelector />
    <PromptInput />
    <ActionControls />
  </div>
</main>

<style>
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .scroll-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
    background: var(--bg-base);
  }

  .input-area {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    padding: 0.6rem 1.5rem;
    background: var(--bg-surface);
  }
</style>
