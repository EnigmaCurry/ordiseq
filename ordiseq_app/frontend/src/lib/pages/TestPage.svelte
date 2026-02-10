<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  const NOTE_COUNT = 24;
  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

  const CHORD_TYPES = [
    "Major", "Minor", "Dim", "Aug",
    "Maj7", "7", "m7", "mMaj7",
    "dim7", "m7b5", "Aug7", "AugMaj7",
    "9", "Maj9", "m9", "add9",
    "sus2", "sus4", "6", "m6",
  ];

  interface Key {
    midi: number;
    name: string;
    octave: number;
    isBlack: boolean;
  }

  interface ClientInfo {
    id: number;
    name: string;
    connected: boolean;
  }

  let octaveStart = $state(48);

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

  // Client selection
  let clients: ClientInfo[] = $state([]);
  let selectedClientId: number | null = $state(null);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function refreshClients() {
    clients = await invoke<ClientInfo[]>("get_clients");
    // Auto-select first client if none selected or selected client disconnected
    if (selectedClientId === null || !clients.some((c) => c.id === selectedClientId)) {
      selectedClientId = clients.length > 0 ? clients[0].id : null;
    }
  }

  $effect(() => {
    refreshClients();
    pollTimer = setInterval(refreshClients, 2000);
    return () => { if (pollTimer) clearInterval(pollTimer); };
  });

  // Selection state
  let selectedRoot: number = $state(0); // pitch class 0-11, default C
  let selectedChordType: string = $state("Major");
  let activeNotes: Set<number> = $state(new Set());

  // Display-only: update keyboard visualization when root/type/octave changes
  $effect(() => {
    const rootMidi = octaveStart + selectedRoot;
    invoke<number[]>("get_chord_notes", { rootMidi, chordType: selectedChordType }).then((notes) => {
      activeNotes = new Set(notes);
    });
  });

  // Chord name detection from active notes
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

  function triggerChord() {
    if (selectedClientId === null) return;
    const rootMidi = octaveStart + selectedRoot;
    invoke<number[]>("trigger_live_chord", {
      clientId: selectedClientId,
      rootMidi,
      chordType: selectedChordType,
    }).then((notes) => {
      activeNotes = new Set(notes);
    }).catch(() => {});
  }

  function handleRootClick(i: number) {
    selectedRoot = i;
    triggerChord();
  }

  function handleChordTypeClick(ct: string) {
    selectedChordType = ct;
    triggerChord();
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
      <div class="toolbar-right">
        <select class="client-select" bind:value={selectedClientId}>
          {#if clients.length === 0}
            <option value={null}>No clients</option>
          {:else}
            {#each clients as client}
              <option value={client.id}>{client.name}</option>
            {/each}
          {/if}
        </select>
        <div class="octave-controls">
          <span class="octave-label">{octaveLabel}</span>
          <div class="octave-buttons">
            <button class="oct-btn" onclick={octaveDown} disabled={octaveStart <= 0}>-</button>
            <button class="oct-btn" onclick={octaveUp} disabled={octaveStart + NOTE_COUNT >= 128}>+</button>
          </div>
        </div>
      </div>
    </div>

    <div class="root-row">
      {#each NOTE_NAMES as name, i}
        <button
          class="root-btn"
          class:active={selectedRoot === i}
          class:black-note={[1, 3, 6, 8, 10].includes(i)}
          onmousedown={() => handleRootClick(i)}
        >{name}</button>
      {/each}
    </div>

    <div class="chord-grid">
      {#each CHORD_TYPES as ct}
        <button
          class="chord-btn"
          class:active={selectedChordType === ct}
          onmousedown={() => handleChordTypeClick(ct)}
        >{ct}</button>
      {/each}
    </div>

    <div class="keyboard">
      {#each whiteKeys as key}
        <div
          class="key white"
          class:active={activeNotes.has(key.midi)}
        >
          <span class="label">{key.name}{key.octave}</span>
        </div>
      {/each}

      {#each blackKeys as key}
        <div
          class="key black"
          class:active={activeNotes.has(key.midi)}
          style="left: {blackKeyLeft(key.midi)}%; width: {whiteW * 0.6}%"
        ></div>
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

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .client-select {
    padding: 4px 6px;
    font-size: 0.75rem;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.1);
    color: rgb(var(--color-3));
    border: 1px solid rgba(var(--color-3), 0.3);
    max-width: 150px;
  }

  .client-select:focus {
    outline: 1px solid rgb(var(--color-1));
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

  /* Root note row */
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
    color: #fff;
    border-color: rgb(var(--color-2));
  }

  /* Chord type grid */
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
    color: #fff;
    border-color: rgb(var(--color-2));
  }

  /* Keyboard */
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
