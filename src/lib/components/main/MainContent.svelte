<script lang="ts">
  import Header from "./Header.svelte";
  import AtomDetail from "./AtomDetail.svelte";
  import WorkspaceSelector from "./WorkspaceSelector.svelte";
  import ContextBadge from "./ContextBadge.svelte";
  import PromptInput from "./PromptInput.svelte";
  import ActionControls from "./ActionControls.svelte";
</script>

<main class="main-content">
  <Header />

  <div class="scroll-area">
    <AtomDetail />
  </div>

  <div class="input-area">
    <WorkspaceSelector />
    <ContextBadge />
    <PromptInput />
    <ActionControls />
  </div>
</main>

<style>
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .scroll-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
  }

  .input-area {
    flex-shrink: 0;
    border-top: 1px solid #ddd;
    padding: 0.6rem 1.5rem;
    background: #fafbfc;
  }
</style>
