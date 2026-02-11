<script lang="ts">
  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const BAR_WIDTH = 100;

  interface SequenceChord {
    id: string;
    root: number;
    chordType: string;
    bars: number;
    customNotes?: number[];
    customLabel?: string;
  }

  interface Props {
    sequence: SequenceChord[];
    activeChordIndex: number;
    dropTargetIndex: number;
    replaceTargetIndex: number;
    dropIndicatorLeft: number;
    selectedSeqIndex: number;
    dragSourceIndex: number;
    dragActive: boolean;
    totalBars: number;
    onstartreorderdrag: (e: MouseEvent, index: number) => void;
    onselect: (index: number) => void;
    onclear: () => void;
    onbindel: (el: HTMLElement) => void;
  }

  let {
    sequence, activeChordIndex, dropTargetIndex, replaceTargetIndex, dropIndicatorLeft,
    selectedSeqIndex, dragSourceIndex, dragActive, totalBars,
    onstartreorderdrag, onselect, onclear, onbindel,
  }: Props = $props();

  let sequenceEl: HTMLElement | undefined = $state(undefined);

  $effect(() => {
    if (sequenceEl) onbindel(sequenceEl);
  });
</script>

<div class="sequence-section">
  <div class="sequence-header">
    <span class="sequence-title">Sequence</span>
    {#if sequence.length > 0}
      <span class="seq-info">{totalBars} bar{totalBars !== 1 ? 's' : ''}</span>
      <button class="clear-seq-btn" onclick={onclear}>Clear</button>
    {/if}
  </div>
  <div
    class="sequence-track"
    class:drag-active={dropTargetIndex >= 0 || replaceTargetIndex >= 0}
    bind:this={sequenceEl}
  >
    {#if sequence.length === 0 && dropTargetIndex < 0}
      <div class="seq-empty">Drag chords here to build a sequence</div>
    {:else}
      {#each sequence as chord, i (chord.id)}
        <div
          class="seq-chord"
          class:dragging={dragActive && dragSourceIndex === i}
          class:playing-chord={activeChordIndex === i}
          class:replace-target={replaceTargetIndex === i}
          class:selected-chord={selectedSeqIndex === i}
          onmousedown={(e) => onstartreorderdrag(e, i)}
          onclick={() => onselect(i)}
          style="width: {chord.bars * BAR_WIDTH}px"
        >
          <div class="seq-chord-name">{chord.chordType === "Custom" ? (chord.customLabel || "Custom") : `${NOTE_NAMES[chord.root]} ${chord.chordType}`}</div>
        </div>
      {/each}
    {/if}
    {#if dropTargetIndex >= 0}
      <div class="drop-indicator" style="left: {dropIndicatorLeft}px"></div>
    {/if}
  </div>
</div>

<style>
  .sequence-section {
    margin-bottom: 12px;
  }

  .sequence-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .sequence-title {
    font-size: 0.85rem;
    font-weight: 700;
    color: rgba(var(--color-3), 0.7);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .seq-info {
    font-size: 0.75rem;
    color: rgba(var(--color-3), 0.5);
  }

  .clear-seq-btn {
    margin-left: auto;
    padding: 2px 8px;
    font-size: 0.65rem;
    border-radius: 3px;
    background: rgba(255, 80, 80, 0.15);
    color: #ff6666;
    border: 1px solid rgba(255, 80, 80, 0.3);
    cursor: pointer;
  }

  .clear-seq-btn:hover {
    background: rgba(255, 80, 80, 0.3);
  }

  .sequence-track {
    position: relative;
    display: flex;
    align-items: stretch;
    gap: 4px;
    padding: 6px;
    min-height: 70px;
    border: 2px dashed rgba(var(--color-3), 0.2);
    border-radius: 6px;
    overflow-x: auto;
    transition: border-color 0.15s, background-color 0.15s;
  }

  .sequence-track.drag-active {
    border-color: rgba(var(--color-1), 0.5);
    background: rgba(var(--color-1), 0.04);
  }

  .seq-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    font-size: 0.75rem;
    color: rgba(var(--color-3), 0.35);
    pointer-events: none;
  }

  .seq-chord {
    flex-shrink: 0;
    min-width: 70px;
    background: rgba(var(--color-4), 0.12);
    border: 1px solid rgba(var(--color-4), 0.25);
    border-left: 3px solid rgb(var(--color-4));
    outline: 1px solid rgb(var(--color-1));
    border-radius: 4px;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 4px;
    position: relative;
    cursor: grab;
    user-select: none;
    transition: opacity 0.15s;
  }

  .seq-chord:active {
    cursor: grabbing;
  }

  .seq-chord.dragging {
    opacity: 0.35;
  }

  .seq-chord.playing-chord {
    background: rgba(var(--color-1), 0.25);
    border-color: rgba(var(--color-1), 0.6);
    border-left-color: rgb(var(--color-1));
    outline-color: rgb(var(--color-1));
    box-shadow: 0 0 8px rgba(var(--color-1), 0.3);
  }

  .seq-chord.replace-target {
    background: rgba(var(--color-1), 0.18);
    border-color: rgba(var(--color-1), 0.5);
    border-left-color: rgb(var(--color-1));
    outline: 2px solid rgba(var(--color-1), 0.7);
  }

  .seq-chord.selected-chord {
    outline: 2px solid rgba(var(--color-3), 0.6);
  }

  .seq-chord-name {
    font-size: 0.8rem;
    font-weight: 700;
    color: rgb(var(--color-2));
    white-space: nowrap;
  }

  .drop-indicator {
    position: absolute;
    top: 4px;
    bottom: 4px;
    width: 3px;
    background: rgb(var(--color-1));
    border-radius: 2px;
    pointer-events: none;
    z-index: 10;
  }
</style>
