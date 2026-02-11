<script lang="ts">
  import type { MidiInfo } from "./types";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import { resolveResource } from "@tauri-apps/api/path";
  import { invoke } from "@tauri-apps/api/core";

  let { midiInfo, compact = false }: { midiInfo: MidiInfo; compact?: boolean } = $props();
  let error = $state("");
  let isPlaying = $state(false);

  async function handleMouseDown(event: MouseEvent) {
    event.preventDefault();
    error = "";
    try {
      const iconPath = await resolveResource("icons/32x32.png");
      await startDrag({
        item: [midiInfo.path],
        icon: iconPath,
      });
    } catch (e) {
      error = String(e);
    }
  }

  async function handlePlay() {
    error = "";
    if (isPlaying) {
      try {
        await invoke("stop_midi");
        isPlaying = false;
      } catch (e) {
        error = String(e);
      }
    } else {
      try {
        await invoke("play_midi", { params: { midi_path: midiInfo.path } });
        isPlaying = true;
        pollPlaybackStatus();
      } catch (e) {
        error = String(e);
      }
    }
  }

  async function pollPlaybackStatus() {
    while (isPlaying) {
      await new Promise((resolve) => setTimeout(resolve, 200));
      try {
        const status = (await invoke("get_playback_status")) as {
          playing: boolean;
        };
        if (!status.playing) {
          isPlaying = false;
          break;
        }
      } catch {
        isPlaying = false;
        break;
      }
    }
  }
</script>

{#if compact}
  <div class="compact-widget" role="button" tabindex="0" onmousedown={handleMouseDown}>
    <button
      class="compact-play"
      class:playing={isPlaying}
      onclick={(e) => { e.stopPropagation(); handlePlay(); }}
      title={isPlaying ? "Stop" : "Play"}
    >
      {#if isPlaying}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="14" height="14">
          <rect x="6" y="6" width="12" height="12" />
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="14" height="14">
          <path d="M8 5v14l11-7z" />
        </svg>
      {/if}
    </button>
    <svg class="compact-note" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
      <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
    </svg>
    <span class="compact-detail">{midiInfo.note_count} notes</span>
  </div>
  {#if error}
    <div class="error">{error}</div>
  {/if}
{:else}
  <div class="midi-widget">
    <button
      class="play-button"
      class:playing={isPlaying}
      onclick={handlePlay}
      title={isPlaying ? "Stop" : "Play"}
    >
      {#if isPlaying}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="40" height="40">
          <rect x="6" y="6" width="12" height="12" />
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="40" height="40">
          <path d="M8 5v14l11-7z" />
        </svg>
      {/if}
    </button>

    <div
      class="drag-area"
      role="button"
      tabindex="0"
      onmousedown={handleMouseDown}
    >
      <div class="icon">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="48" height="48">
          <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
        </svg>
      </div>
      <div class="info">
        <div class="title">{midiInfo.title}</div>
        <div class="details">{midiInfo.note_count} notes</div>
        <div class="hint">Drag to export</div>
      </div>
    </div>
  </div>
  {#if error}
    <div class="error">{error}</div>
  {/if}
{/if}

<style>
  /* --- Compact layout --- */
  .compact-widget {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0 0.5rem;
    height: 40px;
    box-sizing: border-box;
    background-color: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(var(--color-3), 0.4);
    border-radius: 4px;
    cursor: grab;
    -webkit-user-select: none;
    user-select: none;
    transition: background-color 0.15s ease;
  }

  .compact-widget:hover {
    background-color: rgba(var(--color-3), 0.1);
  }

  .compact-widget:active {
    cursor: grabbing;
    background-color: rgba(var(--color-3), 0.2);
  }

  .compact-play {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    margin: 0;
    background-color: rgb(var(--color-4));
    border: none;
    border-radius: 3px;
    cursor: pointer;
    color: rgb(var(--color-2));
    flex-shrink: 0;
  }

  .compact-play:hover {
    filter: brightness(1.2);
  }

  .compact-play.playing {
    background-color: rgb(var(--color-1));
    color: rgb(var(--text-on-1));
  }

  .compact-note {
    color: rgb(var(--color-1));
    flex-shrink: 0;
  }

  .compact-label {
    font-weight: 600;
    font-size: 0.8rem;
    color: #f8f8f2;
  }

  .compact-sep {
    color: #666;
    font-size: 0.75rem;
  }

  .compact-detail {
    font-size: 0.75rem;
    color: rgb(var(--color-2));
  }

  .compact-hint {
    font-size: 0.7rem;
    color: rgba(var(--color-3), 0.7);
  }

  /* --- Full layout --- */
  .midi-widget {
    display: grid;
    grid-template-columns: 72px 1fr;
    background-color: rgba(0, 0, 0, 0.5);
    border: 2px solid rgba(var(--color-3), 0.5);
    border-radius: 8px;
    overflow: hidden;
  }

  .play-button {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem 0;
    margin: 0;
    background-color: rgb(var(--color-4));
    border: none;
    border-radius: 0;
    cursor: pointer;
    color: rgb(var(--color-2));
    transition: all 0.2s ease;
  }

  .play-button:hover {
    filter: brightness(1.2);
  }

  .play-button.playing {
    background-color: rgb(var(--color-1));
    color: rgb(var(--text-on-1));
  }

  .play-button.playing:hover {
    filter: brightness(1.2);
  }

  .drag-area {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    cursor: grab;
    -webkit-user-select: none;
    user-select: none;
    transition: all 0.2s ease;
    border-left: 2px dashed rgba(var(--color-3), 0.5);
  }

  .drag-area:hover {
    background-color: rgba(var(--color-3), 0.1);
  }

  .drag-area:active {
    cursor: grabbing;
    background-color: rgba(var(--color-3), 0.2);
  }

  .icon {
    color: rgb(var(--color-1));
    flex-shrink: 0;
  }

  .info {
    flex: 1;
    min-width: 0;
  }

  .title {
    font-weight: 600;
    font-size: 1.1rem;
    color: #f8f8f2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .details {
    font-size: 0.875rem;
    color: rgb(var(--color-2));
  }

  .hint {
    font-size: 0.75rem;
    color: rgba(var(--color-3), 0.7);
    margin-top: 0.25rem;
  }

  .error {
    font-size: 0.7rem;
    color: rgb(var(--color-1));
    margin-top: 0.5rem;
    padding: 0 0.5rem;
  }
</style>
