<script lang="ts">
  import Navigation from "./lib/Navigation.svelte";
  import TestPage from "./lib/pages/TestPage.svelte";
  import SettingsPage from "./lib/pages/SettingsPage.svelte";
  import { currentPage } from "./lib/router";
</script>

<Navigation />

<main>
  {#if $currentPage === "test"}
    <TestPage />
  {:else if $currentPage === "settings"}
    <SettingsPage />
  {/if}
</main>

<style>
  main {
    text-align: center;
  }
</style>
