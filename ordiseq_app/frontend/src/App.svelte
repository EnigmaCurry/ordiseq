<script lang="ts">
  import Navigation from "./lib/Navigation.svelte";
  import MainPage from "./lib/pages/MainPage.svelte";
  import TestPage from "./lib/pages/TestPage.svelte";
  import SettingsPage from "./lib/pages/SettingsPage.svelte";
  import { currentPage } from "./lib/router";
  import { setUniforms, shaderConfig } from "./lib/shaderStore";

  // Store the user's overlay setting
  let userOverlay = 0.9;

  // Track page changes to toggle overlay
  $effect(() => {
    const page = $currentPage;
    const currentOverlay = $shaderConfig.uniforms.u_overlay as number;

    if (page === "main") {
      // Save current overlay and disable it
      if (currentOverlay > 0) {
        userOverlay = currentOverlay;
      }
      setUniforms({ u_overlay: 0 });
    } else {
      // Restore overlay on other pages
      if (($shaderConfig.uniforms.u_overlay as number) === 0) {
        setUniforms({ u_overlay: userOverlay });
      }
    }
  });
</script>

<Navigation />

<main>
  {#if $currentPage === "main"}
    <MainPage />
  {:else if $currentPage === "test"}
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
