<script lang="ts">
  import type { MidiInfo } from "./types";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import { resolveResource } from "@tauri-apps/api/path";

  let { midiInfo }: { midiInfo: MidiInfo } = $props();
  let error = $state("");

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
</script>

<div
  class="midi-widget"
  role="button"
  tabindex="0"
  onmousedown={handleMouseDown}
>
  <div class="icon">
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      width="48"
      height="48"
    >
      <path
        d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
      />
    </svg>
  </div>
  <div class="info">
    <div class="title">{midiInfo.title}</div>
    <div class="details">{midiInfo.note_count} notes</div>
    <div class="hint">Drag to export</div>
    {#if error}
      <div class="error">{error}</div>
    {/if}
  </div>
</div>

<style>
  .midi-widget {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background-color: #44475a;
    border: 2px dashed #6272a4;
    border-radius: 8px;
    cursor: grab;
    -webkit-user-select: none;
    user-select: none;
    transition: all 0.2s ease;
  }

  .midi-widget:hover {
    border-color: #ff79c6;
    background-color: #4d5066;
  }

  .midi-widget:active {
    cursor: grabbing;
    border-color: #50fa7b;
  }

  .icon {
    color: #ff79c6;
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
    color: #8be9fd;
  }

  .hint {
    font-size: 0.75rem;
    color: #6272a4;
    margin-top: 0.25rem;
  }

  .error {
    font-size: 0.7rem;
    color: #ff5555;
    margin-top: 0.25rem;
  }
</style>
