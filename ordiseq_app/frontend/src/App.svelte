<script lang="ts">
  import Navigation from "./lib/Navigation.svelte";
  import MainPage from "./lib/pages/MainPage.svelte";
  import SettingsPage from "./lib/pages/SettingsPage.svelte";
  import ClientsPage from "./lib/pages/ClientsPage.svelte";
  import { currentPage } from "./lib/router";
  import { setUniforms, shaderConfig } from "./lib/shaderStore";
  import { beatPosition, transportBpm, transportPlaying } from "./lib/transportStore";
  import { get } from "svelte/store";

  // Store the user's overlay setting
  let userOverlay = 0.9;
  let previousPage: string | null = null;

  // Track page changes to toggle overlay (only react to page changes, not shader changes)
  $effect(() => {
    const page = $currentPage;

    // Only act on actual page changes
    if (page === previousPage) return;

    if (page === "graphics") {
      // Save current overlay and disable it (use get() to avoid reactive dependency)
      const currentOverlay = get(shaderConfig).uniforms.u_overlay as number;
      if (currentOverlay > 0) {
        userOverlay = currentOverlay;
      }
      setUniforms({ u_overlay: 0 });
    } else if (previousPage === "graphics") {
      // Only restore when leaving graphics page
      setUniforms({ u_overlay: userOverlay });
    }

    previousPage = page;
  });

  // Push transport sync state into shader uniforms
  $effect(() => {
    const beat = $beatPosition;
    const bpm = $transportBpm;
    const playing = $transportPlaying;
    setUniforms({
      u_beat: beat,
      u_bpm: bpm,
      u_playing: playing ? 1.0 : 0.0,
    });
  });
</script>

<Navigation />

<main>
  {#if $currentPage === "graphics"}
    <MainPage />
  {:else if $currentPage === "settings"}
    <SettingsPage />
  {:else if $currentPage === "clients"}
    <ClientsPage />
  {/if}
</main>

<style>
  main {
    text-align: center;
  }
</style>
