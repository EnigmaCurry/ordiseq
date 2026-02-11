<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import type { MidiClip, ClipNote } from "../clientsStore";

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

  interface SequenceChord {
    id: string;
    root: number;
    chordType: string;
    bars: number;
  }

  const BAR_WIDTH = 100;
  const DRAG_THRESHOLD = 25; // px² (5px movement)

  // --- Sequence state ---
  let sequence: SequenceChord[] = $state([]);
  let nextChordId = 0;
  let dropTargetIndex: number = $state(-1);
  let dropIndicatorLeft: number = $state(0);
  let sequenceEl: HTMLElement | undefined = $state(undefined);

  let totalBars: number = $derived(sequence.reduce((sum, c) => sum + c.bars, 0));

  // --- Custom mouse drag state ---
  interface DragInfo {
    type: 'new' | 'reorder';
    root: number;
    chordType: string;
    sourceIndex: number; // -1 for new
    startX: number;
    startY: number;
    active: boolean;
  }

  let drag: DragInfo | null = $state(null);
  let dragWasActive = false;
  let ghostX: number = $state(0);
  let ghostY: number = $state(0);

  function genId(): string { return `c${nextChordId++}`; }

  function removeChord(i: number) {
    stopSequenceIfPlaying();
    sequence = sequence.filter((_, idx) => idx !== i);
  }

  function adjustDuration(i: number, delta: number) {
    stopSequenceIfPlaying();
    const newBars = Math.max(0.5, sequence[i].bars + delta);
    sequence = sequence.map((c, idx) => idx === i ? { ...c, bars: newBars } : c);
  }

  function isOverSequence(cx: number, cy: number): boolean {
    if (!sequenceEl) return false;
    const r = sequenceEl.getBoundingClientRect();
    return cx >= r.left && cx <= r.right && cy >= r.top && cy <= r.bottom;
  }

  function calcDropIndex(cx: number): number {
    if (!sequenceEl || sequence.length === 0) return 0;
    const chordEls = sequenceEl.querySelectorAll('.seq-chord');
    for (let i = 0; i < chordEls.length; i++) {
      const rect = chordEls[i].getBoundingClientRect();
      if (cx < rect.left + rect.width / 2) return i;
    }
    return sequence.length;
  }

  function updateIndicatorPosition(idx: number) {
    if (!sequenceEl) return;
    const containerRect = sequenceEl.getBoundingClientRect();
    const chordEls = sequenceEl.querySelectorAll('.seq-chord');
    if (chordEls.length === 0) {
      dropIndicatorLeft = 6;
    } else if (idx <= 0) {
      const r = chordEls[0].getBoundingClientRect();
      dropIndicatorLeft = r.left - containerRect.left + sequenceEl.scrollLeft;
    } else if (idx >= chordEls.length) {
      const r = chordEls[chordEls.length - 1].getBoundingClientRect();
      dropIndicatorLeft = r.right - containerRect.left + sequenceEl.scrollLeft;
    } else {
      const prev = chordEls[idx - 1].getBoundingClientRect();
      const next = chordEls[idx].getBoundingClientRect();
      dropIndicatorLeft = (prev.right + next.left) / 2 - containerRect.left + sequenceEl.scrollLeft;
    }
  }

  function startNewChordDrag(e: MouseEvent, root: number, chordType: string) {
    drag = { type: 'new', root, chordType, sourceIndex: -1, startX: e.clientX, startY: e.clientY, active: false };
    window.addEventListener('mousemove', onDragMove);
    window.addEventListener('mouseup', onDragEnd);
  }

  function startReorderDrag(e: MouseEvent, i: number) {
    // Don't start drag when clicking buttons inside the chord block
    if ((e.target as HTMLElement).closest('button')) return;
    const chord = sequence[i];
    drag = { type: 'reorder', root: chord.root, chordType: chord.chordType, sourceIndex: i, startX: e.clientX, startY: e.clientY, active: false };
    window.addEventListener('mousemove', onDragMove);
    window.addEventListener('mouseup', onDragEnd);
  }

  function onDragMove(e: MouseEvent) {
    if (!drag) return;
    if (!drag.active) {
      const dx = e.clientX - drag.startX;
      const dy = e.clientY - drag.startY;
      if (dx * dx + dy * dy < DRAG_THRESHOLD) return;
      drag = { ...drag, active: true };
    }
    e.preventDefault(); // prevent text selection while dragging
    ghostX = e.clientX;
    ghostY = e.clientY;

    if (isOverSequence(e.clientX, e.clientY)) {
      const idx = calcDropIndex(e.clientX);
      dropTargetIndex = idx;
      updateIndicatorPosition(idx);
    } else {
      dropTargetIndex = -1;
    }
  }

  function onDragEnd(e: MouseEvent) {
    window.removeEventListener('mousemove', onDragMove);
    window.removeEventListener('mouseup', onDragEnd);

    if (drag?.active && isOverSequence(e.clientX, e.clientY)) {
      stopSequenceIfPlaying();
      const idx = calcDropIndex(e.clientX);

      if (drag.type === 'new') {
        sequence = [...sequence.slice(0, idx), { id: genId(), root: drag.root, chordType: drag.chordType, bars: 1 }, ...sequence.slice(idx)];
      } else if (drag.type === 'reorder' && drag.sourceIndex >= 0) {
        const chord = sequence[drag.sourceIndex];
        const filtered = sequence.filter((_, i) => i !== drag!.sourceIndex);
        const insertAt = idx > drag.sourceIndex ? idx - 1 : idx;
        sequence = [...filtered.slice(0, insertAt), chord, ...filtered.slice(insertAt)];
      }
    }

    dragWasActive = drag?.active ?? false;
    drag = null;
    dropTargetIndex = -1;
  }

  function selectSequenceChord(i: number) {
    if (dragWasActive) { dragWasActive = false; return; }
    const chord = sequence[i];
    selectedRoot = chord.root;
    selectedChordType = chord.chordType;
  }

  onDestroy(() => {
    window.removeEventListener('mousemove', onDragMove);
    window.removeEventListener('mouseup', onDragEnd);
  });

  // --- Existing state ---
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
  let selectedRoot: number = $state(0);
  let selectedChordType: string = $state("Major");
  let activeNotes: Set<number> = $state(new Set());

  $effect(() => {
    const rootMidi = octaveStart + selectedRoot;
    invoke<number[]>("get_chord_notes", { rootMidi, chordType: selectedChordType }).then((notes) => {
      activeNotes = new Set(notes);
    });
  });

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
    stopSequenceIfPlaying();
    selectedRoot = i;
    triggerChord();
  }

  function handleChordTypeClick(ct: string) {
    stopSequenceIfPlaying();
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

  // --- Sequence playback ---
  let sequencePlaying: boolean = $state(false);
  let activeChordIndex: number = $state(-1);
  let positionTimer: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    if (positionTimer) { clearInterval(positionTimer); positionTimer = null; }
    if (sequencePlaying && selectedClientId !== null) {
      const clientId = selectedClientId;
      positionTimer = setInterval(async () => {
        const beatPos = await invoke<number>("get_sequence_position", { clientId });
        if (beatPos < 0) { activeChordIndex = -1; return; }
        let cumBeats = 0;
        for (let i = 0; i < sequence.length; i++) {
          cumBeats += sequence[i].bars * 4;
          if (beatPos < cumBeats) { activeChordIndex = i; return; }
        }
        activeChordIndex = 0;
      }, 150);
    } else {
      activeChordIndex = -1;
    }
    return () => { if (positionTimer) { clearInterval(positionTimer); positionTimer = null; } };
  });

  // Auto-update chord selector to match the currently playing chord
  $effect(() => {
    if (activeChordIndex >= 0 && activeChordIndex < sequence.length) {
      const chord = sequence[activeChordIndex];
      selectedRoot = chord.root;
      selectedChordType = chord.chordType;
    }
  });

  function stopSequenceIfPlaying() {
    if (sequencePlaying && selectedClientId !== null) {
      invoke("stop_sequence", { clientId: selectedClientId });
      sequencePlaying = false;
    }
  }

  // Reset playback state when client changes
  let prevClientId: number | null = null;
  $effect(() => {
    if (selectedClientId !== prevClientId) {
      prevClientId = selectedClientId;
      sequencePlaying = false;
    }
  });

  async function sequenceToClip(): Promise<MidiClip> {
    const notes: ClipNote[] = [];
    let beatPos = 0;
    for (const chord of sequence) {
      const rootMidi = octaveStart + chord.root;
      const chordNotes: number[] = await invoke("get_chord_notes", { rootMidi, chordType: chord.chordType });
      const fullBeats = chord.bars * 4;
      const durationBeats = fullBeats - 0.25;
      for (const n of chordNotes) {
        notes.push({ note: n, channel: 0, velocity: 0.8, start_beats: beatPos, duration_beats: durationBeats });
      }
      beatPos += fullBeats;
    }
    return { name: "Chord Sequence", length_beats: beatPos, notes };
  }

  async function toggleSequencePlayback() {
    if (selectedClientId === null) return;
    if (sequencePlaying) {
      await invoke("stop_sequence", { clientId: selectedClientId });
      sequencePlaying = false;
    } else {
      const clip = await sequenceToClip();
      await invoke("play_sequence", { clientId: selectedClientId, clip });
      sequencePlaying = true;
    }
  }
</script>

<div class="page">
  <h1>Test</h1>

  <!-- Sequence Builder -->
  <div class="sequence-section">
    <div class="sequence-header">
      <span class="sequence-title">Sequence</span>
      {#if sequence.length > 0}
        <span class="seq-info">{totalBars} bar{totalBars !== 1 ? 's' : ''}</span>
        <button
          class="seq-play-btn"
          class:playing={sequencePlaying}
          disabled={selectedClientId === null}
          onclick={toggleSequencePlayback}
        >{sequencePlaying ? "Stop" : "Play"}</button>
        <button class="clear-seq-btn" onclick={() => { stopSequenceIfPlaying(); sequence = []; }}>Clear</button>
      {/if}
    </div>
    <div
      class="sequence-track"
      class:drag-active={dropTargetIndex >= 0}
      bind:this={sequenceEl}
    >
      {#if sequence.length === 0 && dropTargetIndex < 0}
        <div class="seq-empty">Drag chords here to build a sequence</div>
      {:else}
        {#each sequence as chord, i (chord.id)}
          <div
            class="seq-chord"
            class:dragging={drag?.active === true && drag.type === 'reorder' && drag.sourceIndex === i}
            class:playing-chord={activeChordIndex === i}
            onmousedown={(e) => startReorderDrag(e, i)}
            onclick={() => selectSequenceChord(i)}
            style="width: {chord.bars * BAR_WIDTH}px"
          >
            <button class="seq-remove" onclick={() => removeChord(i)}>×</button>
            <div class="seq-chord-name">{NOTE_NAMES[chord.root]} {chord.chordType}</div>
            <div class="seq-duration">
              <button class="dur-btn" onclick={() => adjustDuration(i, -0.5)} disabled={chord.bars <= 0.5}>-</button>
              <span class="dur-label">{chord.bars}</span>
              <button class="dur-btn" onclick={() => adjustDuration(i, 0.5)}>+</button>
            </div>
          </div>
        {/each}
      {/if}
      {#if dropTargetIndex >= 0}
        <div class="drop-indicator" style="left: {dropIndicatorLeft}px"></div>
      {/if}
    </div>
  </div>

  <!-- Chord Selector -->
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
          onmousedown={(e) => { handleRootClick(i); startNewChordDrag(e, i, selectedChordType); }}
        >{name}</button>
      {/each}
    </div>

    <div class="chord-grid">
      {#each CHORD_TYPES as ct}
        <button
          class="chord-btn"
          class:active={selectedChordType === ct}
          onmousedown={(e) => { handleChordTypeClick(ct); startNewChordDrag(e, selectedRoot, ct); }}
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

<!-- Drag ghost floating element -->
{#if drag?.active}
  <div class="drag-ghost" style="left: {ghostX}px; top: {ghostY}px">
    {NOTE_NAMES[drag.root]} {drag.chordType}
  </div>
{/if}

<style>
  .page {
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    color: rgb(var(--color-1));
    margin-bottom: 0.25rem;
  }

  /* ===== Sequence Builder ===== */
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

  .seq-play-btn {
    margin-left: auto;
    padding: 2px 10px;
    font-size: 0.65rem;
    font-weight: 700;
    border-radius: 3px;
    background: rgba(var(--color-1), 0.2);
    color: rgb(var(--color-1));
    border: 1px solid rgba(var(--color-1), 0.4);
    cursor: pointer;
  }

  .seq-play-btn:hover:not(:disabled) {
    background: rgba(var(--color-1), 0.35);
  }

  .seq-play-btn.playing {
    background: rgba(var(--color-1), 0.6);
    color: #fff;
  }

  .seq-play-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .clear-seq-btn {
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

  .seq-chord-name {
    font-size: 0.8rem;
    font-weight: 700;
    color: rgb(var(--color-2));
    white-space: nowrap;
  }

  .seq-duration {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .dur-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    font-size: 0.75rem;
    font-weight: 700;
    line-height: 1;
    border-radius: 3px;
    background: rgba(var(--color-3), 0.15);
    color: rgb(var(--color-3));
    border: 1px solid rgba(var(--color-3), 0.2);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dur-btn:hover:not(:disabled) {
    background: rgba(var(--color-3), 0.3);
  }

  .dur-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .dur-label {
    font-size: 0.65rem;
    color: rgba(var(--color-3), 0.6);
    min-width: 16px;
    text-align: center;
  }

  .seq-remove {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 16px;
    height: 16px;
    padding: 0;
    font-size: 0.7rem;
    line-height: 1;
    background: transparent;
    color: rgba(var(--color-3), 0.3);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
  }

  .seq-remove:hover {
    background: rgba(255, 0, 0, 0.2);
    color: #ff6666;
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

  /* ===== Drag Ghost ===== */
  :global(.drag-ghost) {
    position: fixed;
    pointer-events: none;
    z-index: 1000;
    transform: translate(-50%, -110%);
    padding: 4px 10px;
    font-size: 0.75rem;
    font-weight: 700;
    color: rgb(var(--color-1));
    background: rgba(var(--color-1), 0.15);
    border: 1px solid rgba(var(--color-1), 0.4);
    border-radius: 4px;
    white-space: nowrap;
    backdrop-filter: blur(4px);
  }

  /* ===== Chord Selector Panel ===== */
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
