<script lang="ts">
  import { currentPage, type Page } from "./router";

  let menuOpen = $state(false);
  let idle = $state(false);
  let idleTimer: ReturnType<typeof setTimeout> | null = null;

  const IDLE_DELAY = 2000;

  const pages: { id: Page; label: string }[] = [
    { id: "graphics", label: "Graphics" },
    { id: "clients", label: "Clients" },
    { id: "settings", label: "Settings" },
  ];

  function resetIdle() {
    idle = false;
    if (idleTimer) clearTimeout(idleTimer);
    if ($currentPage === "graphics" && !menuOpen) {
      idleTimer = setTimeout(() => { idle = true; }, IDLE_DELAY);
    }
  }

  // Start/stop idle tracking when page or menu changes
  $effect(() => {
    if ($currentPage === "graphics" && !menuOpen) {
      idleTimer = setTimeout(() => { idle = true; }, IDLE_DELAY);
    } else {
      idle = false;
      if (idleTimer) clearTimeout(idleTimer);
    }
    return () => { if (idleTimer) clearTimeout(idleTimer); };
  });

  function navigate(page: Page) {
    currentPage.set(page);
    menuOpen = false;
  }

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && menuOpen) {
      menuOpen = false;
    }
    resetIdle();
  }
</script>

<svelte:window onkeydown={handleKeydown} onmousemove={resetIdle} onmousedown={resetIdle} />

<nav class="navigation" class:idle>
  <button class="hamburger" onclick={toggleMenu} aria-label="Toggle menu">
    <span class="hamburger-line" class:open={menuOpen}></span>
    <span class="hamburger-line" class:open={menuOpen}></span>
    <span class="hamburger-line" class:open={menuOpen}></span>
  </button>

  {#if menuOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
    <div class="menu-backdrop" onclick={() => (menuOpen = false)}></div>
    <div class="menu">
      {#each pages as page}
        <button
          class="menu-item"
          class:active={$currentPage === page.id}
          onclick={() => navigate(page.id)}
        >
          {page.label}
        </button>
      {/each}
    </div>
  {/if}
</nav>

<style>
  .navigation {
    position: fixed;
    top: 1rem;
    left: 1rem;
    z-index: 100;
    opacity: 1;
    transition: opacity 0.5s ease;
  }

  .navigation.idle {
    opacity: 0;
    pointer-events: none;
  }

  .hamburger {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 5px;
    width: 40px;
    height: 40px;
    padding: 8px;
    background-color: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .hamburger:hover {
    background-color: rgba(var(--color-3), 0.3);
  }

  .hamburger-line {
    width: 20px;
    height: 2px;
    background-color: rgb(var(--color-2));
    transition: transform 0.2s ease, opacity 0.2s ease;
  }

  .hamburger-line.open:nth-child(1) {
    transform: translateY(7px) rotate(45deg);
  }

  .hamburger-line.open:nth-child(2) {
    opacity: 0;
  }

  .hamburger-line.open:nth-child(3) {
    transform: translateY(-7px) rotate(-45deg);
  }

  .menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.3);
    z-index: -1;
  }

  .menu {
    position: absolute;
    top: 50px;
    left: 0;
    min-width: 150px;
    background-color: rgba(0, 0, 0, 0.9);
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 6px;
    overflow: hidden;
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: 0.75rem 1rem;
    text-align: left;
    font-size: 1rem;
    color: #f8f8f2;
    background: none;
    border: none;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .menu-item:hover {
    background-color: rgba(var(--color-3), 0.2);
  }

  .menu-item.active {
    color: rgb(var(--color-1));
    background-color: rgba(var(--color-1), 0.1);
  }
</style>
