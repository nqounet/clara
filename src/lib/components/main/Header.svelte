<script lang="ts">
  import { configStore } from "$lib/stores";
</script>

<header class="header">
  <h1>CLARA <span class="app-version">v{configStore.appVersion}</span></h1>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    padding: 0.5rem 1.5rem;
    border-bottom: 1px solid #eee;
    flex-shrink: 0;
  }

  .header h1 {
    margin: 0;
    font-size: 1.1rem;
    color: #333;
  }

  .app-version {
    font-size: 0.7em;
    color: #888;
    font-weight: normal;
    margin-left: 0.5rem;
  }
</style>
