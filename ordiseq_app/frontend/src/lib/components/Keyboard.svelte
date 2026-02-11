<script lang="ts">
  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

  interface Props {
    activeNotes: Set<number>;
    octaveStart: number;
    noteCount?: number;
    onkeydown: (e: MouseEvent, midi: number) => void;
  }

  let { activeNotes, octaveStart, noteCount = 24, onkeydown }: Props = $props();

  interface Key {
    midi: number;
    name: string;
    octave: number;
    isBlack: boolean;
  }

  let keys: Key[] = $derived(
    Array.from({ length: noteCount }, (_, i) => {
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
</script>

<div class="keyboard">
  {#each whiteKeys as key}
    <div
      class="key white"
      class:active={activeNotes.has(key.midi)}
      onmousedown={(e) => onkeydown(e, key.midi)}
    >
      <span class="label">{key.name}{key.octave}</span>
    </div>
  {/each}

  {#each blackKeys as key}
    <div
      class="key black"
      class:active={activeNotes.has(key.midi)}
      style="left: {blackKeyLeft(key.midi)}%; width: {whiteW * 0.6}%"
      onmousedown={(e) => onkeydown(e, key.midi)}
    ></div>
  {/each}
</div>

<style>
  .keyboard {
    position: relative;
    display: flex;
    height: 140px;
    overflow: visible;
  }

  .key {
    border: none;
    padding: 0;
    margin: 0;
    font-size: 0.6rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.1s ease;
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

  .key.white:last-child {
    border-right: none;
  }

  .key.white.active {
    background: rgb(var(--color-1));
    border: 2px solid rgb(var(--color-2));
    border-top: none;
  }

  .label {
    color: #666;
    padding-bottom: 6px;
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

  .key.black.active {
    background: rgb(var(--color-1));
    border: 2px solid rgb(var(--color-2));
    border-top: none;
  }
</style>
