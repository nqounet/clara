<script lang="ts">
  import { configStore } from "$lib/stores";
</script>

<div class="header">
  <h1>CLARA <span class="app-version">v{configStore.appVersion}</span></h1>
</div>

<style>
  .header {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .header h1 {
    margin: 0;
    font-size: 1rem;
    color: var(--accent-green);
    white-space: nowrap;
  }

  .app-version {
    font-size: 0.7em;
    color: var(--text-muted);
    font-weight: normal;
    margin-left: 0.5rem;
  }
</style>
