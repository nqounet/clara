<script lang="ts">
  import VaultHeader from "./VaultHeader.svelte";
  import SearchBar from "./SearchBar.svelte";
  import AtomList from "./AtomList.svelte";
</script>

<aside class="sidebar">
  <VaultHeader />
  <SearchBar />
  <AtomList />
</aside>

<style>
  .sidebar {
    width: 260px;
    background: #f4f6f8;
    border-right: 1px solid #ddd;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
