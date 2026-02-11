<script lang="ts">
  import {
    clients,
    renameClient,
    sendClipToClient,
    sendPlayModeToClient,
    euclideanMetaToClip,
    chordsToClip,
    defaultEuclidRowMeta,
    midiNoteName,
    computeStepPattern,
    parseMetaSequence,
    STEP_BEATS,
    loadConfigForClient,
    saveConfigForClient,
    renameStoredConfig,
    storedToRowMetas,
    type ClientInfo,
    type SequencerType,
    type SequenceChord,
    type EuclidRow,
    type EuclidRowMeta,
    type StepState,
  } from "../clientsStore";
  import { get } from "svelte/store";
  import { syncState, beatPosition, transportPlaying, transportBpm } from "../transportStore";
  import type { MidiInfo } from "../types";
  import { invoke } from "@tauri-apps/api/core";
  import Dial from "../Dial.svelte";
  import MidiWidget from "../MidiWidget.svelte";
  import ChordPanel from "../components/ChordPanel.svelte";

  let editingId = $state<number | null>(null);
  let editName = $state("");

  // Per-client sequencer state, keyed by client id
  let sequencerTypes = $state<Record<number, SequencerType>>({});
  let euclideanRowMetas = $state<Record<number, EuclidRowMeta[]>>({});
  let chordSequences = $state<Record<number, SequenceChord[]>>({});
  let octaveStarts = $state<Record<number, number>>({});
  let midiInfos = $state<Record<number, MidiInfo>>({});

  // Per-client sequence playback position (beat position, -1 = not playing)
  let seqPositions = $state<Record<number, number>>({});
  let seqPollTimers: Record<number, ReturnType<typeof setInterval>> = {};

  function startSeqPositionPoll(clientId: number) {
    stopSeqPositionPoll(clientId);
    seqPositions[clientId] = 0;
    seqPollTimers[clientId] = setInterval(async () => {
      try {
        const pos = await invoke<number>("get_sequence_position", { clientId });
        seqPositions[clientId] = pos;
        if (pos < 0) stopSeqPositionPoll(clientId);
      } catch {
        stopSeqPositionPoll(clientId);
      }
    }, 100);
  }

  function stopSeqPositionPoll(clientId: number) {
    if (seqPollTimers[clientId]) {
      clearInterval(seqPollTimers[clientId]);
      delete seqPollTimers[clientId];
    }
    seqPositions[clientId] = -1;
  }

  // Per-client edit program (1-16), controls which program slot the sequencer edits
  let editPrograms = $state<Record<number, number>>({});

  // Per-client play mode for current edit program
  let playModes = $state<Record<number, "transport" | "note_trigger">>({});

  // Per-client collapsed state
  let collapsed = $state<Record<number, boolean>>({});

  // Track which client IDs have been initialized from storage
  let initializedIds = new Set<number>();

  // Debounce timer per client
  let syncTimers: Record<number, ReturnType<typeof setTimeout>> = {};

  // Fix duplicate chord IDs in sequences loaded from storage
  let nextFixId = Date.now();
  function fixChordIds(seq: SequenceChord[]): SequenceChord[] {
    return seq.map(c => ({ ...c, id: `c${nextFixId++}` }));
  }

  // Restore saved configs when new clients appear
  $effect(() => {
    const currentClients = $clients;
    for (const client of currentClients) {
      if (!initializedIds.has(client.id)) {
        initializedIds.add(client.id);
        const pgm = editPrograms[client.id] ?? 1;
        const stored = loadConfigForClient(client.name, pgm);
        if (stored) {
          sequencerTypes[client.id] = stored.type;
          euclideanRowMetas[client.id] = storedToRowMetas(stored.rows);
          chordSequences[client.id] = fixChordIds(stored.chordSequence ?? []);
          octaveStarts[client.id] = stored.octaveStart ?? 48;
          playModes[client.id] = stored.noteTrigger ? "note_trigger" : "transport";
          sendPlayModeToClient(client.id, pgm - 1, stored.noteTrigger ?? false);
          if (stored.type === "euclidean" && stored.rows.length > 0) {
            syncToClient(client.id);
          } else if (stored.type === "chords" && (stored.chordSequence?.length ?? 0) > 0) {
            syncChordsToClient(client.id);
          }
        }
      }
    }
  });

  function startEditing(client: ClientInfo) {
    editingId = client.id;
    editName = client.name;
  }

  async function submitRename(clientId: number) {
    if (editName.trim()) {
      const oldName = clientName(clientId);
      const newName = editName.trim();
      if (oldName && oldName !== newName) {
        renameStoredConfig(oldName, newName);
      }
      await renameClient(clientId, newName);
    }
    editingId = null;
  }

  function handleKeydown(event: KeyboardEvent, clientId: number) {
    if (event.key === "Enter") {
      submitRename(clientId);
    } else if (event.key === "Escape") {
      editingId = null;
    }
  }

  function clientName(clientId: number): string {
    return $clients.find((c) => c.id === clientId)?.name ?? "";
  }

  function getEditProgram(clientId: number): number {
    return editPrograms[clientId] ?? 1;
  }

  function setEditProgram(clientId: number, pgm: number) {
    const oldPgm = getEditProgram(clientId);
    if (pgm === oldPgm) return;
    // Cancel any pending debounced sync from the old program
    if (syncTimers[clientId]) { clearTimeout(syncTimers[clientId]); delete syncTimers[clientId]; }
    // Save current config for old program
    persistConfig(clientId);
    // Switch to new program
    editPrograms[clientId] = pgm;
    // Load config for new program
    const name = clientName(clientId);
    const stored = name ? loadConfigForClient(name, pgm) : null;
    if (stored) {
      sequencerTypes[clientId] = stored.type;
      euclideanRowMetas[clientId] = storedToRowMetas(stored.rows);
      chordSequences[clientId] = fixChordIds(stored.chordSequence ?? []);
      octaveStarts[clientId] = stored.octaveStart ?? 48;
      playModes[clientId] = stored.noteTrigger ? "note_trigger" : "transport";
      midiInfos[clientId] = undefined as any;
      if (stored.type === "euclidean" && stored.rows.length > 0) {
        syncToClient(clientId);
      } else if (stored.type === "chords" && (stored.chordSequence?.length ?? 0) > 0) {
        syncChordsToClient(clientId);
      }
    } else {
      sequencerTypes[clientId] = "none";
      euclideanRowMetas[clientId] = [];
      chordSequences[clientId] = [];
      octaveStarts[clientId] = 48;
      playModes[clientId] = "transport";
      midiInfos[clientId] = undefined as any;
    }
    sendPlayModeToClient(clientId, pgm - 1, playModes[clientId] === "note_trigger");
  }

  function getSequencerType(clientId: number): SequencerType {
    return sequencerTypes[clientId] ?? "none";
  }

  function setSequencerType(clientId: number, type: SequencerType) {
    sequencerTypes[clientId] = type;
    if (type === "euclidean" && (!euclideanRowMetas[clientId] || euclideanRowMetas[clientId].length === 0)) {
      euclideanRowMetas[clientId] = [defaultEuclidRowMeta()];
    }
    if (type === "chords" && !chordSequences[clientId]) {
      chordSequences[clientId] = [];
      octaveStarts[clientId] = 48;
    }
    persistConfig(clientId);
    if (type === "euclidean") {
      syncToClient(clientId);
    } else if (type === "chords") {
      syncChordsToClient(clientId);
    } else if (type === "none") {
      const pgm = getEditProgram(clientId) - 1;
      sendClipToClient(clientId, { name: "Empty", length_beats: 1, notes: [] }, pgm);
    }
  }

  function getRowMetas(clientId: number): EuclidRowMeta[] {
    return euclideanRowMetas[clientId] ?? [];
  }

  /** Get the active slot's EuclidRow for a row meta. */
  function activeRow(meta: EuclidRowMeta): EuclidRow {
    return meta.slots[meta.activeSlot] ?? meta.slots["A"];
  }

  function addRow(clientId: number) {
    const metas = euclideanRowMetas[clientId] ?? [];
    const lastNote = metas.length > 0 ? activeRow(metas[metas.length - 1]).note : 35;
    euclideanRowMetas[clientId] = [...metas, defaultEuclidRowMeta(lastNote + 1)];
    syncToClient(clientId);
  }

  function removeRow(clientId: number, index: number) {
    const metas = euclideanRowMetas[clientId] ?? [];
    if (metas.length <= 1) return;
    euclideanRowMetas[clientId] = metas.filter((_, i) => i !== index);
    syncToClient(clientId);
  }

  function updateRow(clientId: number, index: number, field: keyof EuclidRow, value: number) {
    const metas = euclideanRowMetas[clientId];
    if (!metas) return;
    const meta = metas[index];
    const slot = meta.activeSlot;
    const row = { ...meta.slots[slot] };

    if (field === "note") {
      row.note = Math.max(0, Math.min(127, value));
    } else if (field === "length") {
      row.length = Math.max(1, Math.min(32, value));
      row.hits = Math.min(row.hits, row.length);
      row.rotation = Math.min(row.rotation, row.length - 1);
      row.accents = Math.min(row.accents, row.hits);
    } else if (field === "hits") {
      row.hits = Math.max(0, Math.min(row.length, value));
      row.accents = Math.min(row.accents, row.hits);
    } else if (field === "rotation") {
      row.rotation = Math.max(0, Math.min(row.length - 1, value));
    } else if (field === "accents") {
      row.accents = Math.max(0, Math.min(row.hits, value));
    } else if (field === "velocity") {
      row.velocity = Math.max(0, Math.min(127, value));
    } else if (field === "accentVelocity") {
      row.accentVelocity = Math.max(0, Math.min(127, value));
    }

    const newMetas = [...metas];
    newMetas[index] = { ...meta, slots: { ...meta.slots, [slot]: row } };
    euclideanRowMetas[clientId] = newMetas;
    debouncedSync(clientId);
  }

  function setActiveSlot(clientId: number, rowIndex: number, letter: string) {
    const metas = euclideanRowMetas[clientId];
    if (!metas) return;
    const newMetas = [...metas];
    newMetas[rowIndex] = { ...newMetas[rowIndex], activeSlot: letter };
    euclideanRowMetas[clientId] = newMetas;
  }

  function updateMetaSequence(clientId: number, rowIndex: number, text: string) {
    const metas = euclideanRowMetas[clientId];
    if (!metas) return;
    const newMetas = [...metas];
    newMetas[rowIndex] = { ...newMetas[rowIndex], metaSequence: text };
    euclideanRowMetas[clientId] = newMetas;
    debouncedSync(clientId);
  }

  function debouncedSync(clientId: number) {
    if (syncTimers[clientId]) clearTimeout(syncTimers[clientId]);
    syncTimers[clientId] = setTimeout(() => {
      const type = getSequencerType(clientId);
      if (type === "euclidean") syncToClient(clientId);
      else if (type === "chords") syncChordsToClient(clientId);
    }, 150);
  }

  function persistConfig(clientId: number) {
    const name = clientName(clientId);
    if (name) {
      const pgm = getEditProgram(clientId);
      const noteTrigger = (playModes[clientId] ?? "transport") === "note_trigger";
      saveConfigForClient(
        name, sequencerTypes[clientId] ?? "none", euclideanRowMetas[clientId] ?? [], pgm, noteTrigger,
        chordSequences[clientId], octaveStarts[clientId],
      );
    }
  }

  async function syncToClient(clientId: number) {
    const metas = euclideanRowMetas[clientId];
    if (!metas || metas.length === 0) return;
    const pgm = getEditProgram(clientId) - 1; // capture before any await
    persistConfig(clientId);
    try {
      const clip = euclideanMetaToClip(metas);
      await sendClipToClient(clientId, clip, pgm);
      const result = await invoke<MidiInfo>("clip_to_midi_file", { clip, repeats: 1 });
      midiInfos[clientId] = result;
    } catch (e) {
      console.error("Failed to sync clip:", e);
    }
  }

  async function syncChordsToClient(clientId: number) {
    const seq = chordSequences[clientId];
    const pgm = getEditProgram(clientId) - 1; // capture before any await
    if (!seq || seq.length === 0) {
      sendClipToClient(clientId, { name: "Empty", length_beats: 1, notes: [] }, pgm);
      midiInfos[clientId] = undefined as any;
      persistConfig(clientId);
      return;
    }
    persistConfig(clientId);
    try {
      const oct = octaveStarts[clientId] ?? 48;
      const clip = await chordsToClip(seq, oct);
      await sendClipToClient(clientId, clip, pgm);
      const result = await invoke<MidiInfo>("clip_to_midi_file", { clip, repeats: 1 });
      midiInfos[clientId] = result;
    } catch (e) {
      console.error("Failed to sync chord clip:", e);
    }
  }

  function handleChordSequenceChange(clientId: number, seq: SequenceChord[]) {
    chordSequences[clientId] = seq;
    debouncedSync(clientId);
  }

  function handleOctaveChange(clientId: number, oct: number) {
    octaveStarts[clientId] = oct;
    debouncedSync(clientId);
  }

  async function playSequenceForClient(clientId: number): Promise<number> {
    const seq = chordSequences[clientId];
    const oct = octaveStarts[clientId] ?? 48;
    if (!seq || seq.length === 0) return 0;
    const clip = await chordsToClip(seq, oct);
    await invoke("play_sequence", { clientId, clip });
    startSeqPositionPoll(clientId);
    const bpm = get(transportBpm) || 120;
    const totalBars = seq.reduce((sum, c) => sum + c.bars, 0);
    return totalBars * 4 * (60000 / bpm);
  }

  async function stopSequenceForClient(clientId: number): Promise<void> {
    await invoke("stop_sequence", { clientId });
    stopSeqPositionPoll(clientId);
  }

  function getPlayMode(clientId: number): "transport" | "note_trigger" {
    return playModes[clientId] ?? "transport";
  }

  function setPlayMode(clientId: number, mode: "transport" | "note_trigger") {
    playModes[clientId] = mode;
    persistConfig(clientId);
    const pgm = getEditProgram(clientId) - 1;
    sendPlayModeToClient(clientId, pgm, mode === "note_trigger");
  }

  function isManualMode(row: EuclidRow): boolean {
    return row.manualPattern != null;
  }

  function cycleStep(state: StepState): StepState {
    if (state === "off") return "hit";
    if (state === "hit") return "accent";
    return "off";
  }

  function toggleStep(clientId: number, rowIndex: number, stepIdx: number) {
    const metas = euclideanRowMetas[clientId];
    if (!metas) return;
    const meta = metas[rowIndex];
    const slot = meta.activeSlot;
    const row = { ...meta.slots[slot] };

    if (!row.manualPattern) {
      row.manualPattern = [...computeStepPattern(row)];
    }

    row.manualPattern = [...row.manualPattern];
    row.manualPattern[stepIdx] = cycleStep(row.manualPattern[stepIdx]);

    const newMetas = [...metas];
    newMetas[rowIndex] = { ...meta, slots: { ...meta.slots, [slot]: row } };
    euclideanRowMetas[clientId] = newMetas;
    debouncedSync(clientId);
  }

  function resetToEuclidean(clientId: number, rowIndex: number) {
    if (!confirm("Reset to Euclidean mode? Your manual edits will be lost.")) return;
    const metas = euclideanRowMetas[clientId];
    if (!metas) return;
    const meta = metas[rowIndex];
    const slot = meta.activeSlot;
    const row = { ...meta.slots[slot] };
    delete row.manualPattern;
    const newMetas = [...metas];
    newMetas[rowIndex] = { ...meta, slots: { ...meta.slots, [slot]: row } };
    euclideanRowMetas[clientId] = newMetas;
    debouncedSync(clientId);
  }

  // --- Meta-sequencer playback position ---

  const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

  /** Determine which slot letter is currently playing for a row, given beat position. */
  function getPlayingSlot(meta: EuclidRowMeta, beat: number): string | null {
    const letters = parseMetaSequence(meta.metaSequence);
    if (!letters || letters.length === 0) return null;

    let totalSteps = 0;
    const segments: { letter: string; startStep: number; length: number }[] = [];
    for (const letter of letters) {
      const row = meta.slots[letter] ?? meta.slots["A"];
      segments.push({ letter, startStep: totalSteps, length: row.length });
      totalSteps += row.length;
    }

    if (totalSteps === 0) return null;

    const globalStep = Math.floor(beat / STEP_BEATS);
    const cycleStep = ((globalStep % totalSteps) + totalSteps) % totalSteps;

    for (const seg of segments) {
      if (cycleStep >= seg.startStep && cycleStep < seg.startStep + seg.length) {
        return seg.letter;
      }
    }
    return letters[0];
  }

  /** Current step index within the active slot, or -1 if a different slot is playing. */
  function getCurrentMetaStep(meta: EuclidRowMeta, beat: number): number {
    const letters = parseMetaSequence(meta.metaSequence) ?? ["A"];

    let totalSteps = 0;
    const segments: { letter: string; startStep: number; length: number }[] = [];
    for (const letter of letters) {
      const row = meta.slots[letter] ?? meta.slots["A"];
      segments.push({ letter, startStep: totalSteps, length: row.length });
      totalSteps += row.length;
    }

    if (totalSteps === 0) return -1;

    const globalStep = Math.floor(beat / STEP_BEATS);
    const cycleStepPos = ((globalStep % totalSteps) + totalSteps) % totalSteps;

    for (const seg of segments) {
      if (cycleStepPos >= seg.startStep && cycleStepPos < seg.startStep + seg.length) {
        if (seg.letter === meta.activeSlot) {
          return cycleStepPos - seg.startStep;
        }
        return -1;
      }
    }
    return -1;
  }
</script>

<div class="page">
  <h2>Connected Plugins</h2>

  {#if $clients.length === 0}
    <p class="empty">No plugins connected. Load an Ordiseq plugin in your DAW to get started.</p>
  {:else}
    <div class="client-list">
      {#each $clients as client (client.id)}
        {@const seqType = sequencerTypes[client.id] ?? "none"}
        <div class="client-card" onwheel={(e) => e.preventDefault()}>
          <div class="client-header">
            {#if editingId === client.id}
              <input
                class="name-input"
                type="text"
                bind:value={editName}
                onkeydown={(e) => handleKeydown(e, client.id)}
                onblur={() => submitRename(client.id)}
              />
            {:else}
              <span class="client-name" onclick={() => startEditing(client)}>
                {client.name}
              </span>
            {/if}
            <span class="client-version">{client.version}</span>
            <button class="disconnect-btn" onclick={() => invoke("disconnect_client", { clientId: client.id })} title="Disconnect">×</button>
          </div>

          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="client-info" onclick={() => collapsed[client.id] = !collapsed[client.id]}>
            <span class="collapse-chevron" class:open={!collapsed[client.id]}>&#9654;</span>
            {#if $syncState.source_client_id === client.id}
              <div class="info-row">
                <span class="beat-dot" style="opacity: {0.15 + 0.85 * Math.max(0, Math.cos(($beatPosition % 1.0) * 2 * Math.PI))}"></span>
                <span class="label">BPM:</span>
                <span class="value">{client.bpm > 0 ? client.bpm.toFixed(1) : "—"}</span>
              </div>
            {/if}
            <div class="info-row">
              <span class="label">Transport:</span>
              <span class="value transport" class:playing={client.playing}>
                {client.playing ? "Playing" : "Stopped"}
              </span>
            </div>
            <div class="info-row">
              <span class="label">Program:</span>
              <span class="value">{client.program + 1}</span>
            </div>
          </div>

          {#if !collapsed[client.id]}
          <div class="sequencer-section">
            <div class="seq-select-row">
              <Dial value={getEditProgram(client.id)} min={1} max={16}
                label="Program"
                onchange={(v) => setEditProgram(client.id, v)} />
              <div class="seq-select-wrapper">
                <select
                  class="seq-select"
                  value={seqType}
                  onchange={(e) => setSequencerType(client.id, (e.target as HTMLSelectElement).value as SequencerType)}
                >
                  <option value="none">None</option>
                  <option value="euclidean">Euclidean</option>
                  <option value="chords">Chords</option>
                </select>
                <span class="seq-select-spacer"></span>
                <span class="seq-select-label">Sequencer</span>
              </div>
              <div class="seq-select-wrapper">
                <select
                  class="seq-select"
                  value={getPlayMode(client.id)}
                  onchange={(e) => setPlayMode(client.id, (e.target as HTMLSelectElement).value as "transport" | "note_trigger")}
                >
                  <option value="transport">Transport</option>
                  <option value="note_trigger">Note Trigger</option>
                </select>
                <span class="seq-select-spacer"></span>
                <span class="seq-select-label">Play Mode</span>
              </div>
              {#if midiInfos[client.id]}
                <div class="midi-drag-inline">
                  <MidiWidget midiInfo={midiInfos[client.id]} compact
                    transportPlaying={client.playing}
                    onplay={seqType === "chords" ? () => playSequenceForClient(client.id) : undefined}
                    onstop={seqType === "chords" ? () => stopSequenceForClient(client.id) : undefined}
                  />
                  <span class="seq-select-spacer"></span>
                  <span class="midi-drag-label">{seqType === "chords" ? "Drag clip to export" : "Drag to export"}</span>
                </div>
              {/if}
            </div>

            {#if seqType === "euclidean"}
              <div class="euclid-panel">
                <div class="euclid-header-row">
                  <span class="euclid-col note-col">Note</span>
                  <span class="euclid-col">Steps</span>
                  <span class="euclid-col">Hits</span>
                  <span class="euclid-col">Accents</span>
                  <span class="euclid-col">Vel Hit</span>
                  <span class="euclid-col">Vel Acc</span>
                  <span class="euclid-col">Rotate</span>
                  <span class="euclid-col btn-col"></span>
                </div>

                {#each getRowMetas(client.id) as rowMeta, i}
                  {@const row = activeRow(rowMeta)}
                  {@const manual = isManualMode(row)}
                  {@const pattern = computeStepPattern(row)}
                  {@const currentStep = $transportPlaying ? getCurrentMetaStep(rowMeta, $beatPosition) : -1}
                  {@const playingSlot = $transportPlaying ? getPlayingSlot(rowMeta, $beatPosition) : null}
                  <div class="euclid-row">
                    <span class="euclid-col note-col dial-col">
                      <Dial value={row.note} min={0} max={127}
                        formatValue={midiNoteName}
                        onchange={(v) => updateRow(client.id, i, "note", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.length} min={1} max={32}
                        disabled={manual}
                        ondisabledinteract={() => resetToEuclidean(client.id, i)}
                        onchange={(v) => updateRow(client.id, i, "length", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.hits} min={0} max={row.length}
                        disabled={manual}
                        ondisabledinteract={() => resetToEuclidean(client.id, i)}
                        onchange={(v) => updateRow(client.id, i, "hits", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.accents} min={0} max={row.hits}
                        disabled={manual}
                        ondisabledinteract={() => resetToEuclidean(client.id, i)}
                        onchange={(v) => updateRow(client.id, i, "accents", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.velocity} min={0} max={127}
                        onchange={(v) => updateRow(client.id, i, "velocity", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.accentVelocity} min={0} max={127}
                        onchange={(v) => updateRow(client.id, i, "accentVelocity", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.rotation} min={0} max={Math.max(0, row.length - 1)}
                        disabled={manual}
                        ondisabledinteract={() => resetToEuclidean(client.id, i)}
                        onchange={(v) => updateRow(client.id, i, "rotation", v)} />
                    </span>
                    <span class="euclid-col btn-col">
                      {#if getRowMetas(client.id).length > 1}
                        <button class="remove-btn" onclick={() => removeRow(client.id, i)}>×</button>
                      {/if}
                    </span>
                  </div>
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="pattern-row" class:single-step={pattern.length === 1} class:manual-mode={manual}>
                    {#each pattern as step, stepIdx}
                      <!-- svelte-ignore a11y_click_events_have_key_events -->
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <span class="step-dot clickable" class:active={step !== "off"} class:accent={step === "accent"} class:current={currentStep === stepIdx}
                        onclick={() => toggleStep(client.id, i, stepIdx)}></span>
                    {/each}
                  </div>
                  <!-- Meta-sequencer: slot selector + sequence input -->
                  <div class="meta-row">
                    <div class="slot-selector">
                      {#each ALPHABET as letter}
                        {@const isEditing = rowMeta.activeSlot === letter}
                        {@const isPlaying = playingSlot === letter}
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <span class="slot-btn" class:editing={isEditing} class:playing={isPlaying && !isEditing} class:editing-playing={isEditing && isPlaying}
                          onclick={() => setActiveSlot(client.id, i, letter)}>{letter}</span>
                      {/each}
                    </div>
                    <div class="meta-seq-row">
                      <span class="meta-seq-label">Sequence</span>
                      <input
                        type="text"
                        class="meta-input"
                        class:invalid={parseMetaSequence(rowMeta.metaSequence) === null}
                        value={rowMeta.metaSequence}
                        oninput={(e) => updateMetaSequence(client.id, i, (e.target as HTMLInputElement).value)}
                        placeholder="AAAA"
                      />
                    </div>
                  </div>
                {/each}

                <button class="add-row-btn" onclick={() => addRow(client.id)}
                  disabled={getRowMetas(client.id).length >= 12}>
                  + Add Row
                </button>

              </div>
            {/if}

            {#if seqType === "chords"}
              <ChordPanel
                sequence={chordSequences[client.id] ?? []}
                octaveStart={octaveStarts[client.id] ?? 48}
                clientId={client.id}
                seqBeatPosition={seqPositions[client.id] ?? -1}
                playbackActive={client.program === getEditProgram(client.id) - 1}
                onsequencechange={(seq) => handleChordSequenceChange(client.id, seq)}
                onoctavechange={(oct) => handleOctaveChange(client.id, oct)}
              />
            {/if}

          </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    padding: 2rem;
    max-width: 900px;
    margin: 0 auto;
  }

  h2 {
    color: rgb(var(--color-1));
    margin-bottom: 1.5rem;
  }

  .empty {
    color: #888;
    font-style: italic;
    text-align: center;
    padding: 2rem;
  }

  .client-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .client-card {
    background-color: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(var(--color-3), 0.4);
    border-radius: 8px;
    padding: 1rem;
  }

  .client-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .client-name {
    font-size: 1.1rem;
    font-weight: bold;
    color: rgb(var(--color-2));
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
  }

  .client-name:hover {
    background-color: rgba(var(--color-2), 0.15);
  }

  .name-input {
    font-size: 1.1rem;
    font-weight: bold;
    color: rgb(var(--color-2));
    background-color: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(var(--color-3), 0.6);
    border-radius: 4px;
    padding: 2px 6px;
    outline: none;
    width: 200px;
  }

  .client-version {
    font-size: 0.8rem;
    color: #666;
  }

  .disconnect-btn {
    margin-left: auto;
    padding: 0 6px;
    font-size: 1.1rem;
    line-height: 1;
    background: none;
    border: 1px solid rgba(255, 80, 80, 0.3);
    border-radius: 4px;
    color: #888;
    cursor: pointer;
  }

  .disconnect-btn:hover {
    color: #ff5555;
    background: rgba(255, 80, 80, 0.15);
  }

  .client-info {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    margin-bottom: 0.75rem;
    cursor: pointer;
    padding: 4px 0;
    border-radius: 4px;
  }

  .client-info:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .collapse-chevron {
    font-size: 0.6rem;
    color: #666;
    transition: transform 0.15s;
    flex-shrink: 0;
  }

  .collapse-chevron.open {
    transform: rotate(90deg);
  }

  .info-row {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .label {
    color: #888;
    font-size: 0.9rem;
  }

  .value {
    color: #f8f8f2;
    font-size: 0.9rem;
  }

  .transport.playing {
    color: rgb(80, 200, 80);
  }

  .beat-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgb(80, 200, 80);
    margin-right: 4px;
    flex-shrink: 0;
  }

  /* Sequencer section */
  .sequencer-section {
    margin-top: 0.5rem;
    border-top: 1px solid rgba(var(--color-3), 0.2);
    padding-top: 0.75rem;
  }

  .seq-select-row {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .seq-select-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .seq-select {
    background-color: rgba(0, 0, 0, 0.4);
    color: #f8f8f2;
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 4px;
    padding: 0.3rem 0.5rem;
    font-size: 0.85rem;
    outline: none;
    height: 40px;
  }

  .seq-select-spacer {
    height: 0.75rem;
    margin-top: -4px;
  }

  .seq-select-label {
    font-size: 0.6rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* Euclidean panel */
  .euclid-panel {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .euclid-header-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.2rem 0;
    color: #888;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .euclid-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.2rem 0;
  }

  .euclid-col {
    flex: 1;
    text-align: center;
    min-width: 40px;
  }

  .note-col {
    flex: 0 0 48px;
    text-align: center;
  }

  .btn-col {
    flex: 0 0 28px;
  }

  .dial-col {
    display: flex;
    justify-content: center;
  }

  .pattern-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: calc(100% - 28px - 0.4rem);
    padding-bottom: 0.3rem;
  }

  .pattern-row.single-step {
    justify-content: center;
  }

  .pattern-row.manual-mode {
    border-left: 2px solid rgba(255, 220, 80, 0.5);
    padding-left: 6px;
  }

  .step-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
    transition: transform 0.1s;
  }

  .step-dot.clickable {
    cursor: pointer;
  }

  .step-dot.clickable:hover {
    transform: scale(1.5);
  }

  .step-dot.active {
    background-color: rgb(var(--color-2));
  }

  .step-dot.accent {
    background-color: rgb(255, 220, 80);
  }

  .step-dot.current {
    background-color: rgb(var(--color-3));
    box-shadow: 0 0 6px rgb(var(--color-3));
  }

  .remove-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 1.1rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .remove-btn:hover {
    color: #f44;
  }

  .add-row-btn {
    align-self: flex-start;
    margin-top: 0.4rem;
    padding: 0.3rem 0.7rem;
    font-size: 0.8rem;
    color: #f8f8f2;
    background-color: rgba(var(--color-3), 0.25);
    border: 1px solid rgba(var(--color-3), 0.4);
    border-radius: 4px;
    cursor: pointer;
  }

  .add-row-btn:hover:not(:disabled) {
    background-color: rgba(var(--color-3), 0.4);
  }

  .add-row-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .midi-drag-inline {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .midi-drag-label {
    font-size: 0.6rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* Meta-sequencer row */
  .meta-row {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.2rem 0 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 0.2rem;
  }

  .slot-selector {
    display: flex;
    flex-wrap: nowrap;
    gap: 0px;
  }

  .slot-btn {
    width: 18px;
    height: 18px;
    padding: 0;
    font-size: 0.5rem;
    font-family: monospace;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    color: #666;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .slot-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #aaa;
  }

  .slot-btn.editing {
    background: rgba(var(--color-2), 0.3);
    border-color: rgb(var(--color-2));
    color: rgb(var(--color-2));
  }

  .slot-btn.playing {
    background: rgba(80, 200, 80, 0.3);
    border-color: rgb(80, 200, 80);
    color: rgb(80, 200, 80);
  }

  .slot-btn.editing-playing {
    background: rgba(var(--color-2), 0.3);
    border-color: rgb(80, 200, 80);
    color: rgb(var(--color-2));
    box-shadow: 0 0 4px rgba(80, 200, 80, 0.5);
  }

  .meta-seq-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .meta-seq-label {
    font-size: 0.6rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    flex-shrink: 0;
  }

  .meta-input {
    width: 160px;
    height: 22px;
    font-size: 0.75rem;
    font-family: monospace;
    text-transform: uppercase;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 4px;
    color: #f8f8f2;
    padding: 0 6px;
    outline: none;
  }

  .meta-input:focus {
    border-color: rgb(var(--color-2));
  }

  .meta-input.invalid {
    border-color: rgba(255, 80, 80, 0.7);
  }

</style>
