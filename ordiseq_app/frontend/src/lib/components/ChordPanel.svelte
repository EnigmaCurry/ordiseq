<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import type { SequenceChord } from "../clientsStore";
  import { beatPosition, transportPlaying } from "../transportStore";
  import Keyboard from "./Keyboard.svelte";
  import ChordSelector from "./ChordSelector.svelte";
  import SequenceTrack from "./SequenceTrack.svelte";

  const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const NOTE_COUNT = 24;

  interface Props {
    sequence: SequenceChord[];
    octaveStart: number;
    clientId: number;
    onsequencechange: (sequence: SequenceChord[]) => void;
    onoctavechange: (octaveStart: number) => void;
  }

  let { sequence, octaveStart, clientId, onsequencechange, onoctavechange }: Props = $props();

  const DRAG_THRESHOLD = 25;

  let nextChordId = 0;
  let dropTargetIndex: number = $state(-1);
  let dropIndicatorLeft: number = $state(0);
  let sequenceEl: HTMLElement | undefined = $state(undefined);

  let totalBars: number = $derived(sequence.reduce((sum, c) => sum + c.bars, 0));

  function getActiveChordIndex(beat: number): number {
    if (sequence.length === 0 || totalBars === 0) return -1;
    const totalBeats = totalBars * 4;
    const pos = ((beat % totalBeats) + totalBeats) % totalBeats;
    let cumulative = 0;
    for (let i = 0; i < sequence.length; i++) {
      cumulative += sequence[i].bars * 4;
      if (pos < cumulative) return i;
    }
    return sequence.length - 1;
  }

  // --- Custom mouse drag state ---
  interface DragInfo {
    type: 'new' | 'reorder';
    root: number;
    chordType: string;
    customNotes?: number[];
    sourceIndex: number;
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

  function emitSequence(seq: SequenceChord[]) {
    onsequencechange(seq);
  }

  function removeChord(i: number) {
    emitSequence(sequence.filter((_, idx) => idx !== i));
  }

  function adjustDuration(i: number, delta: number) {
    const newBars = Math.max(0.5, sequence[i].bars + delta);
    emitSequence(sequence.map((c, idx) => idx === i ? { ...c, bars: newBars } : c));
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

  function cleanupDragListeners() {
    window.removeEventListener('mousemove', onDragMove);
    window.removeEventListener('mouseup', onDragEnd);
    window.removeEventListener('blur', abortDrag);
    window.removeEventListener('pointercancel', abortDrag);
  }

  function abortDrag() {
    cleanupDragListeners();
    pendingKeyToggle = null;
    drag = null;
    dropTargetIndex = -1;
  }

  function cancelDrag() {
    cleanupDragListeners();
    if (!drag?.active && pendingKeyToggle !== null) {
      toggleKeyNote(pendingKeyToggle);
    }
    pendingKeyToggle = null;
    dragWasActive = drag?.active ?? false;
    drag = null;
    dropTargetIndex = -1;
  }

  function addDragListeners() {
    window.addEventListener('mousemove', onDragMove);
    window.addEventListener('mouseup', onDragEnd);
    window.addEventListener('blur', abortDrag);
    window.addEventListener('pointercancel', abortDrag);
  }

  function startNewChordDrag(e: MouseEvent, root: number, chordType: string) {
    if (e.button !== 0) return;
    cleanupDragListeners();
    const cn = chordType === "Custom" ? [...activeNotes].sort((a, b) => a - b) : undefined;
    drag = { type: 'new', root, chordType, customNotes: cn, sourceIndex: -1, startX: e.clientX, startY: e.clientY, active: false };
    addDragListeners();
  }

  function startReorderDrag(e: MouseEvent, i: number) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest('button')) return;
    cleanupDragListeners();
    const chord = sequence[i];
    drag = { type: 'reorder', root: chord.root, chordType: chord.chordType, sourceIndex: i, startX: e.clientX, startY: e.clientY, active: false };
    addDragListeners();
  }

  function onDragMove(e: MouseEvent) {
    if (!drag) return;
    // Detect missed mouseup (e.g. mouse left window)
    if (e.buttons === 0) { cancelDrag(); return; }
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
    cleanupDragListeners();

    if (drag?.active && isOverSequence(e.clientX, e.clientY)) {
      const idx = calcDropIndex(e.clientX);

      if (drag.type === 'new') {
        const newChord: SequenceChord = { id: genId(), root: drag.root, chordType: drag.chordType, bars: 1 };
        if (drag.customNotes) {
          newChord.customNotes = drag.customNotes;
          newChord.customLabel = chordNames[0] || undefined;
        }
        emitSequence([...sequence.slice(0, idx), newChord, ...sequence.slice(idx)]);
      } else if (drag.type === 'reorder' && drag.sourceIndex >= 0) {
        const chord = sequence[drag.sourceIndex];
        const filtered = sequence.filter((_, i) => i !== drag!.sourceIndex);
        const insertAt = idx > drag.sourceIndex ? idx - 1 : idx;
        emitSequence([...filtered.slice(0, insertAt), chord, ...filtered.slice(insertAt)]);
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

  onDestroy(cleanupDragListeners);

  // --- Selection state ---
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

  function handleKeyMouseDown(e: MouseEvent, midi: number) {
    pendingKeyToggle = midi;
    startNewChordDrag(e, selectedRoot, selectedChordType);
  }

  function triggerChord() {
    if (selectedChordType === "Custom") return;
    const rootMidi = octaveStart + selectedRoot;
    invoke<number[]>("trigger_live_chord", {
      clientId,
      rootMidi,
      chordType: selectedChordType,
    }).then((notes) => {
      activeNotes = new Set(notes);
    }).catch(() => {});
  }

  function toggleKeyNote(midi: number) {
    const newNotes = new Set(activeNotes);
    if (newNotes.has(midi)) {
      newNotes.delete(midi);
    } else {
      newNotes.add(midi);
    }
    activeNotes = newNotes;
    selectedChordType = "Custom";
    if (newNotes.size > 0) {
      const notes = [...newNotes].map(n => ({ note: n, channel: 0, velocity: 0.8 }));
      invoke("send_live_notes", { clientId, notes, durationBeats: 0.0 });
    } else {
      invoke("send_live_notes", { clientId, notes: [], durationBeats: 0.0 });
    }
  }

  function handleRootMouseDown(e: MouseEvent, i: number) {
    selectedRoot = i;
    triggerChord();
    startNewChordDrag(e, i, selectedChordType);
  }

  function handleChordTypeMouseDown(e: MouseEvent, ct: string) {
    selectedChordType = ct;
    triggerChord();
    startNewChordDrag(e, selectedRoot, ct);
  }

  function doOctaveDown() {
    if (octaveStart > 0) onoctavechange(octaveStart - 12);
  }

  function doOctaveUp() {
    if (octaveStart + NOTE_COUNT < 128) onoctavechange(octaveStart + 12);
  }

  let octaveLabel: string = $derived(`C${Math.floor(octaveStart / 12) - 1}`);

  // Sync chord chooser to the currently playing chord
  let lastPlaybackChordIndex = -1;
  $effect(() => {
    if (!$transportPlaying || sequence.length === 0) {
      lastPlaybackChordIndex = -1;
      return;
    }
    const idx = getActiveChordIndex($beatPosition);
    if (idx < 0 || idx === lastPlaybackChordIndex) return;
    lastPlaybackChordIndex = idx;
    const chord = sequence[idx];
    selectedRoot = chord.root;
    selectedChordType = chord.chordType;
    if (chord.chordType === "Custom" && chord.customNotes) {
      activeNotes = new Set(chord.customNotes);
    }
  });
</script>

<div class="chord-panel">
  <SequenceTrack
    {sequence}
    activeChordIndex={$transportPlaying ? getActiveChordIndex($beatPosition) : -1}
    {dropTargetIndex}
    {dropIndicatorLeft}
    dragSourceIndex={drag?.type === 'reorder' ? drag.sourceIndex : -1}
    dragActive={drag?.active ?? false}
    {totalBars}
    onremove={removeChord}
    onadjustduration={adjustDuration}
    onstartreorderdrag={startReorderDrag}
    onselect={selectSequenceChord}
    onclear={() => emitSequence([])}
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
      <div class="octave-controls">
        <span class="octave-label">{octaveLabel}</span>
        <div class="octave-buttons">
          <button class="oct-btn" onclick={doOctaveDown} disabled={octaveStart <= 0}>-</button>
          <button class="oct-btn" onclick={doOctaveUp} disabled={octaveStart + NOTE_COUNT >= 128}>+</button>
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

<!-- Drag ghost floating element -->
{#if drag?.active}
  <div class="drag-ghost" style="left: {ghostX}px; top: {ghostY}px">
    {drag.chordType === "Custom" ? (chordNames[0] || "Custom") : `${NOTE_NAMES[drag.root]} ${drag.chordType}`}
  </div>
{/if}

<style>
  .chord-panel {
    text-align: center;
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

  /* ===== Panel ===== */
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
</style>
