<script lang="ts">
  import { onDestroy } from "svelte";
  import {
    clients,
    startPolling,
    stopPolling,
    renameClient,
    sendClipToClient,
    euclideanToClip,
    defaultEuclidRow,
    midiNoteName,
    bjorklund,
    loadConfigForClient,
    saveConfigForClient,
    renameStoredConfig,
    type ClientInfo,
    type SequencerType,
    type EuclidRow,
  } from "../clientsStore";
  import Dial from "../Dial.svelte";

  startPolling();
  onDestroy(stopPolling);

  let editingId = $state<number | null>(null);
  let editName = $state("");

  // Per-client sequencer state, keyed by client id
  let sequencerTypes = $state<Record<number, SequencerType>>({});
  let euclideanRows = $state<Record<number, EuclidRow[]>>({});

  // Track which client IDs have been initialized from storage
  let initializedIds = new Set<number>();

  // Debounce timer per client
  let syncTimers: Record<number, ReturnType<typeof setTimeout>> = {};

  // Restore saved configs when new clients appear
  $effect(() => {
    for (const client of $clients) {
      if (!initializedIds.has(client.id)) {
        initializedIds.add(client.id);
        const stored = loadConfigForClient(client.name);
        if (stored) {
          sequencerTypes[client.id] = stored.type;
          euclideanRows[client.id] = stored.rows;
          if (stored.type === "euclidean" && stored.rows.length > 0) {
            syncToClient(client.id);
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

  function getSequencerType(clientId: number): SequencerType {
    return sequencerTypes[clientId] ?? "none";
  }

  function setSequencerType(clientId: number, type: SequencerType) {
    sequencerTypes[clientId] = type;
    if (type === "euclidean" && !euclideanRows[clientId]) {
      euclideanRows[clientId] = [defaultEuclidRow()];
    }
    persistConfig(clientId);
    if (type === "euclidean") {
      syncToClient(clientId);
    } else if (type === "none") {
      sendClipToClient(clientId, { name: "Empty", length_beats: 1, notes: [] });
    }
  }

  function getRows(clientId: number): EuclidRow[] {
    return euclideanRows[clientId] ?? [];
  }

  function addRow(clientId: number) {
    const rows = euclideanRows[clientId] ?? [];
    const lastNote = rows.length > 0 ? rows[rows.length - 1].note : 35;
    euclideanRows[clientId] = [...rows, defaultEuclidRow(lastNote + 1)];
    syncToClient(clientId);
  }

  function removeRow(clientId: number, index: number) {
    const rows = euclideanRows[clientId] ?? [];
    if (rows.length <= 1) return;
    euclideanRows[clientId] = rows.filter((_, i) => i !== index);
    syncToClient(clientId);
  }

  function updateRow(clientId: number, index: number, field: keyof EuclidRow, value: number) {
    const rows = euclideanRows[clientId];
    if (!rows) return;
    const row = { ...rows[index] };

    if (field === "note") {
      row.note = Math.max(0, Math.min(127, value));
    } else if (field === "length") {
      row.length = Math.max(1, Math.min(32, value));
      row.hits = Math.min(row.hits, row.length);
      row.rotation = Math.min(row.rotation, row.length - 1);
    } else if (field === "hits") {
      row.hits = Math.max(0, Math.min(row.length, value));
    } else if (field === "rotation") {
      row.rotation = Math.max(0, Math.min(row.length - 1, value));
    }

    const newRows = [...rows];
    newRows[index] = row;
    euclideanRows[clientId] = newRows;
    debouncedSync(clientId);
  }

  function debouncedSync(clientId: number) {
    if (syncTimers[clientId]) clearTimeout(syncTimers[clientId]);
    syncTimers[clientId] = setTimeout(() => syncToClient(clientId), 150);
  }

  function persistConfig(clientId: number) {
    const name = clientName(clientId);
    if (name) {
      saveConfigForClient(name, sequencerTypes[clientId] ?? "none", euclideanRows[clientId] ?? []);
    }
  }

  async function syncToClient(clientId: number) {
    const rows = euclideanRows[clientId];
    if (!rows || rows.length === 0) return;
    persistConfig(clientId);
    try {
      const clip = euclideanToClip(rows);
      await sendClipToClient(clientId, clip);
    } catch (e) {
      console.error("Failed to sync clip:", e);
    }
  }

  function getPattern(row: EuclidRow): boolean[] {
    const pat = bjorklund(row.length, row.hits);
    if (row.rotation === 0) return pat;
    const r = ((row.rotation % pat.length) + pat.length) % pat.length;
    return [...pat.slice(r), ...pat.slice(0, r)];
  }
</script>

<div class="page">
  <h2>Connected Plugins</h2>

  {#if $clients.length === 0}
    <p class="empty">No plugins connected. Load an Ordiseq plugin in your DAW to get started.</p>
  {:else}
    <div class="client-list">
      {#each $clients as client (client.id)}
        <div class="client-card">
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
          </div>

          <div class="client-info">
            <div class="info-row">
              <span class="label">BPM:</span>
              <span class="value">{client.bpm > 0 ? client.bpm.toFixed(1) : "—"}</span>
            </div>
            <div class="info-row">
              <span class="label">Transport:</span>
              <span class="value transport" class:playing={client.playing}>
                {client.playing ? "Playing" : "Stopped"}
              </span>
            </div>
          </div>

          <div class="sequencer-section">
            <div class="seq-select-row">
              <span class="label">Sequencer:</span>
              <select
                class="seq-select"
                value={getSequencerType(client.id)}
                onchange={(e) => setSequencerType(client.id, (e.target as HTMLSelectElement).value as SequencerType)}
              >
                <option value="none">None</option>
                <option value="euclidean">Euclidean</option>
              </select>
            </div>

            {#if getSequencerType(client.id) === "euclidean"}
              <div class="euclid-panel">
                <div class="euclid-header-row">
                  <span class="euclid-col note-col">Note</span>
                  <span class="euclid-col">Length</span>
                  <span class="euclid-col">Hits</span>
                  <span class="euclid-col">Rotation</span>
                  <span class="euclid-col pattern-col">Pattern</span>
                  <span class="euclid-col btn-col"></span>
                </div>

                {#each getRows(client.id) as row, i}
                  <div class="euclid-row">
                    <span class="euclid-col note-col dial-col">
                      <Dial value={row.note} min={0} max={127}
                        formatValue={midiNoteName}
                        onchange={(v) => updateRow(client.id, i, "note", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.length} min={1} max={32}
                        onchange={(v) => updateRow(client.id, i, "length", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.hits} min={0} max={row.length}
                        onchange={(v) => updateRow(client.id, i, "hits", v)} />
                    </span>
                    <span class="euclid-col dial-col">
                      <Dial value={row.rotation} min={0} max={Math.max(0, row.length - 1)}
                        onchange={(v) => updateRow(client.id, i, "rotation", v)} />
                    </span>
                    <span class="euclid-col pattern-col pattern-display">
                      {#each getPattern(row) as active}
                        <span class="step-dot" class:active></span>
                      {/each}
                    </span>
                    <span class="euclid-col btn-col">
                      {#if getRows(client.id).length > 1}
                        <button class="remove-btn" onclick={() => removeRow(client.id, i)}>×</button>
                      {/if}
                    </span>
                  </div>
                {/each}

                <button class="add-row-btn" onclick={() => addRow(client.id)}
                  disabled={getRows(client.id).length >= 12}>
                  + Add Row
                </button>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    padding: 2rem;
    max-width: 700px;
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

  .client-info {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 0.75rem;
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

  /* Sequencer section */
  .sequencer-section {
    margin-top: 0.5rem;
    border-top: 1px solid rgba(var(--color-3), 0.2);
    padding-top: 0.75rem;
  }

  .seq-select-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .seq-select {
    background-color: rgba(0, 0, 0, 0.4);
    color: #f8f8f2;
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 4px;
    padding: 0.3rem 0.5rem;
    font-size: 0.85rem;
    outline: none;
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
    width: 60px;
    text-align: center;
    flex-shrink: 0;
  }

  .note-col {
    width: 48px;
    text-align: center;
  }

  .pattern-col {
    width: 78px;
    text-align: left;
  }

  .btn-col {
    width: 28px;
  }

  .dial-col {
    display: flex;
    justify-content: center;
  }

  .pattern-display {
    display: flex;
    gap: 2px;
    align-items: center;
    flex-wrap: wrap;
    /* 8 dots per row: 8 * 8px + 7 * 2px = 78px */
    width: 78px;
    flex-shrink: 0;
  }

  .step-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
  }

  .step-dot.active {
    background-color: rgb(var(--color-2));
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
</style>
