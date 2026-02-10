<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  const NOTE_COUNT = 24;
  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

  interface Key {
    midi: number;
    name: string;
    octave: number;
    isBlack: boolean;
  }

  let octaveStart = $state(48); // C3 default

  let keys: Key[] = $derived(
    Array.from({ length: NOTE_COUNT }, (_, i) => {
      const midi = octaveStart + i;
      const noteIndex = midi % 12;
      return {
        midi,
        name: NOTE_NAMES[noteIndex],
        octave: Math.floor(midi / 12) - 1,
        isBlack: [1, 3, 6, 8, 10].includes(noteIndex),
      };
    })
  );

  let whiteKeys: Key[] = $derived(keys.filter((k) => !k.isBlack));
  let blackKeys: Key[] = $derived(keys.filter((k) => k.isBlack));

  let activeNotes: Set<number> = $state(new Set());

  function toggleNote(midi: number) {
    const next = new Set(activeNotes);
    if (next.has(midi)) {
      next.delete(midi);
    } else {
      next.add(midi);
    }
    activeNotes = next;
  }

  function octaveDown() {
    if (octaveStart > 0) octaveStart -= 12;
  }

  function octaveUp() {
    if (octaveStart + NOTE_COUNT < 128) octaveStart += 12;
  }

  let whiteW: number = $derived(100 / whiteKeys.length);

  function blackKeyLeft(midi: number): number {
    const noteInOctave = midi % 12;
    const octaveOffset = Math.floor((midi - octaveStart) / 12);
    const blackToWhiteIndex: Record<number, number> = {
      1: 0, 3: 1, 6: 3, 8: 4, 10: 5,
    };
    const whiteIndexInOctave = blackToWhiteIndex[noteInOctave];
    const whiteIndex = octaveOffset * 7 + whiteIndexInOctave;
    return (whiteIndex + 1) * whiteW - whiteW * 0.3;
  }

  let octaveLabel: string = $derived(`C${Math.floor(octaveStart / 12) - 1}`);

  const MAX_CHORDS = 6;
  let chordNames: string[] = $state([]);

  $effect(() => {
    const notes = [...activeNotes];
    if (notes.length < 3) {
      chordNames = [];
      return;
    }
    invoke<string[]>("detect_chord", { midiNotes: notes }).then((names) => {
      chordNames = names;
    });
  });
</script>

<div class="page">
  <h1>Test</h1>
  <p class="subtitle">Chord Selector</p>

  <div class="panel">
    <div class="toolbar">
      <div class="chord-names">
        {#if chordNames.length > 0}
          <span class="chord-primary">{chordNames[0]}</span>
          {#if chordNames.length > 1}
            <span class="chord-alts">{chordNames.slice(1, MAX_CHORDS).join(", ")}{chordNames.length > MAX_CHORDS ? ", ..." : ""}</span>
          {/if}
        {/if}
      </div>
      <div class="octave-controls">
        <span class="octave-label">{octaveLabel}</span>
        <div class="octave-buttons">
          <button class="oct-btn" onclick={octaveDown} disabled={octaveStart <= 0}>-</button>
          <button class="oct-btn" onclick={octaveUp} disabled={octaveStart + NOTE_COUNT >= 128}>+</button>
        </div>
      </div>
    </div>

    <div class="keyboard">
      {#each whiteKeys as key}
        <button
          class="key white"
          class:active={activeNotes.has(key.midi)}
          onclick={() => toggleNote(key.midi)}
        >
          <span class="label">{key.name}{key.octave}</span>
        </button>
      {/each}

      {#each blackKeys as key}
        <button
          class="key black"
          class:active={activeNotes.has(key.midi)}
          style="left: {blackKeyLeft(key.midi)}%; width: {whiteW * 0.6}%"
          onclick={() => toggleNote(key.midi)}
        >
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  .page {
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    color: rgb(var(--color-1));
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: rgba(var(--color-3), 0.7);
    margin-bottom: 1.5rem;
  }

  .panel {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(var(--color-3), 0.25);
    border-radius: 8px;
    padding: 12px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  .chord-names {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-height: 1.5em;
    overflow: hidden;
  }

  .chord-primary {
    font-size: 1.1rem;
    font-weight: 700;
    color: rgb(var(--color-1));
    flex-shrink: 0;
  }

  .chord-alts {
    font-size: 0.75rem;
    color: rgba(var(--color-3), 0.6);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .octave-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .octave-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: rgba(var(--color-3), 0.7);
  }

  .octave-buttons {
    display: flex;
    gap: 4px;
  }

  .oct-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    font-size: 1rem;
    font-weight: 700;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    background: rgba(var(--color-3), 0.15);
    color: rgb(var(--color-3));
    border: 1px solid rgba(var(--color-3), 0.3);
  }

  .oct-btn:hover:not(:disabled) {
    background: rgba(var(--color-3), 0.3);
  }

  .oct-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    filter: none;
  }

  .keyboard {
    position: relative;
    display: flex;
    height: 180px;
    overflow: visible;
  }

  .key {
    border: none;
    cursor: pointer;
    transition: background-color 0.1s ease;
    padding: 0;
    margin: 0;
    font-size: 0.65rem;
    font-weight: 600;
  }

  .key.white {
    flex: 1;
    height: 100%;
    background: #e8e8e8;
    border-right: 1px solid #bbb;
    border-radius: 0 0 4px 4px;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    z-index: 1;
  }

  .key.white:first-child {
    border-radius: 0 0 4px 4px;
  }

  .key.white:last-child {
    border-right: none;
  }

  .key.white:hover {
    background: #d0d0d0;
  }

  .key.white.active {
    background: rgb(var(--color-1));
    border: 2px solid rgb(var(--color-2));
    border-top: none;
  }

  .key.white.active:hover {
    background: rgb(var(--color-1));
    border: 2px solid rgb(var(--color-2));
    border-top: none;
    filter: brightness(1.15);
  }

  .label {
    color: #666;
    padding-bottom: 8px;
    pointer-events: none;
  }

  .key.white.active .label {
    color: #fff;
  }

  .key.black {
    position: absolute;
    top: 0;
    height: 58%;
    background: #222;
    border-radius: 0 0 3px 3px;
    z-index: 2;
  }

  .key.black:hover {
    background: #444;
  }

  .key.black.active {
    background: rgb(var(--color-1));
    border: 2px solid rgb(var(--color-2));
    border-top: none;
  }

  .key.black.active:hover {
    background: rgb(var(--color-1));
    border: 2px solid rgb(var(--color-2));
    border-top: none;
    filter: brightness(1.15);
  }
</style>
