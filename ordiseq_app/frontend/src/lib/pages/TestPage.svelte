<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import type { MidiClip, ClipNote } from "../clientsStore";
  import Keyboard from "../components/Keyboard.svelte";
  import ChordSelector from "../components/ChordSelector.svelte";
  import SequenceTrack from "../components/SequenceTrack.svelte";

  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const NOTE_COUNT = 24;

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
    customNotes?: number[];
    customLabel?: string;
  }

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
    customNotes?: number[];
    sourceIndex: number; // -1 for new
    startX: number;
    startY: number;
    active: boolean;
  }

  let drag: DragInfo | null = $state(null);
  let dragWasActive = false;
  let pendingKeyToggle: number | null = null;
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
    const cn = chordType === "Custom" ? [...activeNotes].sort((a, b) => a - b) : undefined;
    drag = { type: 'new', root, chordType, customNotes: cn, sourceIndex: -1, startX: e.clientX, startY: e.clientY, active: false };
    window.addEventListener('mousemove', onDragMove);
    window.addEventListener('mouseup', onDragEnd);
  }

  function startReorderDrag(e: MouseEvent, i: number) {
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
    e.preventDefault();
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
        const newChord: SequenceChord = { id: genId(), root: drag.root, chordType: drag.chordType, bars: 1 };
        if (drag.customNotes) {
          newChord.customNotes = drag.customNotes;
          newChord.customLabel = chordNames[0] || undefined;
        }
        sequence = [...sequence.slice(0, idx), newChord, ...sequence.slice(idx)];
      } else if (drag.type === 'reorder' && drag.sourceIndex >= 0) {
        const chord = sequence[drag.sourceIndex];
        const filtered = sequence.filter((_, i) => i !== drag!.sourceIndex);
        const insertAt = idx > drag.sourceIndex ? idx - 1 : idx;
        sequence = [...filtered.slice(0, insertAt), chord, ...filtered.slice(insertAt)];
      }
    }

    if (!drag?.active && pendingKeyToggle !== null) {
      toggleKeyNote(pendingKeyToggle);
    }
    pendingKeyToggle = null;

    dragWasActive = drag?.active ?? false;
    drag = null;
    dropTargetIndex = -1;
  }

  function selectSequenceChord(i: number) {
    if (dragWasActive) { dragWasActive = false; return; }
    const chord = sequence[i];
    selectedRoot = chord.root;
    selectedChordType = chord.chordType;
    if (chord.chordType === "Custom" && chord.customNotes) {
      activeNotes = new Set(chord.customNotes);
    }
  }

  onDestroy(() => {
    window.removeEventListener('mousemove', onDragMove);
    window.removeEventListener('mouseup', onDragEnd);
  });

  // --- Existing state ---
  let octaveStart = $state(48);

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
    if (selectedChordType === "Custom") return;
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
    if (selectedChordType === "Custom") return;
    const rootMidi = octaveStart + selectedRoot;
    invoke<number[]>("trigger_live_chord", {
      clientId: selectedClientId,
      rootMidi,
      chordType: selectedChordType,
    }).then((notes) => {
      activeNotes = new Set(notes);
    }).catch(() => {});
  }

  function handleKeyMouseDown(e: MouseEvent, midi: number) {
    pendingKeyToggle = midi;
    startNewChordDrag(e, selectedRoot, selectedChordType);
  }

  function toggleKeyNote(midi: number) {
    stopSequenceIfPlaying();
    const newNotes = new Set(activeNotes);
    if (newNotes.has(midi)) {
      newNotes.delete(midi);
    } else {
      newNotes.add(midi);
    }
    activeNotes = newNotes;
    selectedChordType = "Custom";
    if (selectedClientId !== null && newNotes.size > 0) {
      const notes = [...newNotes].map(n => ({ note: n, channel: 0, velocity: 0.8 }));
      invoke("send_live_notes", { clientId: selectedClientId, notes, durationBeats: 0.0 });
    } else if (selectedClientId !== null) {
      invoke("send_live_notes", { clientId: selectedClientId, notes: [], durationBeats: 0.0 });
    }
  }

  function handleRootMouseDown(e: MouseEvent, i: number) {
    stopSequenceIfPlaying();
    selectedRoot = i;
    triggerChord();
    startNewChordDrag(e, i, selectedChordType);
  }

  function handleChordTypeMouseDown(e: MouseEvent, ct: string) {
    stopSequenceIfPlaying();
    selectedChordType = ct;
    triggerChord();
    startNewChordDrag(e, selectedRoot, ct);
  }

  function octaveDown() {
    if (octaveStart > 0) octaveStart -= 12;
  }

  function octaveUp() {
    if (octaveStart + NOTE_COUNT < 128) octaveStart += 12;
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

  $effect(() => {
    if (activeChordIndex >= 0 && activeChordIndex < sequence.length) {
      const chord = sequence[activeChordIndex];
      selectedRoot = chord.root;
      selectedChordType = chord.chordType;
      if (chord.chordType === "Custom" && chord.customNotes) {
        activeNotes = new Set(chord.customNotes);
      }
    }
  });

  function stopSequenceIfPlaying() {
    if (sequencePlaying && selectedClientId !== null) {
      invoke("stop_sequence", { clientId: selectedClientId });
      sequencePlaying = false;
    }
  }

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
      let chordNotes: number[];
      if (chord.chordType === "Custom" && chord.customNotes) {
        chordNotes = chord.customNotes;
      } else {
        const rootMidi = octaveStart + chord.root;
        chordNotes = await invoke("get_chord_notes", { rootMidi, chordType: chord.chordType });
      }
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

  <SequenceTrack
    {sequence}
    {activeChordIndex}
    {dropTargetIndex}
    {dropIndicatorLeft}
    dragSourceIndex={drag?.type === 'reorder' ? drag.sourceIndex : -1}
    dragActive={drag?.active ?? false}
    {sequencePlaying}
    {totalBars}
    {selectedClientId}
    onremove={removeChord}
    onadjustduration={adjustDuration}
    onstartreorderdrag={startReorderDrag}
    onselect={selectSequenceChord}
    ontoggleplay={toggleSequencePlayback}
    onclear={() => { stopSequenceIfPlaying(); sequence = []; }}
    onbindel={(el) => { sequenceEl = el; }}
  />

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

    <ChordSelector
      {selectedRoot}
      {selectedChordType}
      onrootmousedown={handleRootMouseDown}
      onchordtypemousedown={handleChordTypeMouseDown}
    />

    <Keyboard
      {activeNotes}
      {octaveStart}
      noteCount={NOTE_COUNT}
      onkeydown={handleKeyMouseDown}
    />
  </div>
</div>

{#if drag?.active}
  <div class="drag-ghost" style="left: {ghostX}px; top: {ghostY}px">
    {drag.chordType === "Custom" ? (chordNames[0] || "Custom") : `${NOTE_NAMES[drag.root]} ${drag.chordType}`}
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
</style>
