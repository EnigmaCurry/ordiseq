<script lang="ts">
  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

  const CHORD_TYPES = [
    "Major", "Minor", "Dim", "Aug",
    "Maj7", "7", "m7", "mMaj7",
    "dim7", "m7b5", "Aug7", "AugMaj7",
    "9", "Maj9", "m9", "add9",
    "sus2", "sus4", "6", "m6",
  ];

  interface Props {
    selectedRoot: number;
    selectedChordType: string;
    onrootmousedown: (e: MouseEvent, index: number) => void;
    onchordtypemousedown: (e: MouseEvent, chordType: string) => void;
  }

  let { selectedRoot, selectedChordType, onrootmousedown, onchordtypemousedown }: Props = $props();
</script>

<div class="root-row">
  {#each NOTE_NAMES as name, i}
    <button
      class="root-btn"
      class:active={selectedRoot === i}
      class:black-note={[1, 3, 6, 8, 10].includes(i)}
      onmousedown={(e) => onrootmousedown(e, i)}
    >{name}</button>
  {/each}
</div>

<div class="chord-grid">
  {#each CHORD_TYPES as ct}
    <button
      class="chord-btn"
      class:active={selectedChordType === ct}
      onmousedown={(e) => onchordtypemousedown(e, ct)}
    >{ct}</button>
  {/each}
  {#if selectedChordType === "Custom"}
    <button
      class="chord-btn active"
      onmousedown={(e) => onchordtypemousedown(e, "Custom")}
    >Custom</button>
  {/if}
</div>

<style>
  .root-row {
    display: flex;
    gap: 3px;
    margin-bottom: 8px;
  }

  .root-btn {
    flex: 1;
    padding: 6px 0;
    font-size: 0.75rem;
    font-weight: 600;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.1);
    color: #ccc;
    border: 1px solid rgba(var(--color-3), 0.2);
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .root-btn.black-note {
    background: rgba(0, 0, 0, 0.3);
    color: #999;
  }

  .root-btn:hover {
    background: rgba(var(--color-3), 0.2);
  }

  .root-btn.active {
    background: rgb(var(--color-1));
    color: rgb(var(--text-on-1));
    border-color: rgb(var(--color-2));
  }

  .chord-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 3px;
    margin-bottom: 10px;
  }

  .chord-btn {
    padding: 5px 0;
    font-size: 0.7rem;
    font-weight: 600;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    color: #aaa;
    border: 1px solid rgba(var(--color-3), 0.15);
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .chord-btn:hover {
    background: rgba(var(--color-3), 0.15);
  }

  .chord-btn.active {
    background: rgb(var(--color-1));
    color: rgb(var(--text-on-1));
    border-color: rgb(var(--color-2));
  }
</style>
