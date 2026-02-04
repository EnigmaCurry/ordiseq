<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import MidiWidget from "../MidiWidget.svelte";
  import type { MidiInfo, GenerateMidiParams } from "../types";

  let selectedSequence = $state("c_major_scale");
  let midiInfo: MidiInfo | null = $state(null);
  let isLoading = $state(false);
  let error: string | null = $state(null);

  const sequences = [
    { value: "c_major_scale", label: "C Major Scale" },
    { value: "simple_melody", label: "Simple Melody" },
  ];

  async function handleGenerate() {
    isLoading = true;
    error = null;

    try {
      const params: GenerateMidiParams = {
        sequence_type: selectedSequence,
      };
      midiInfo = await invoke<MidiInfo>("generate_midi", { params });
    } catch (e) {
      error = String(e);
      midiInfo = null;
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="page">
  <h1>Test</h1>
  <p class="subtitle">MIDI Sequence Generator</p>

  <div class="form">
    <div class="field">
      <label for="sequence">Select Sequence</label>
      <select id="sequence" bind:value={selectedSequence}>
        {#each sequences as seq}
          <option value={seq.value}>{seq.label}</option>
        {/each}
      </select>
    </div>

    <button onclick={handleGenerate} disabled={isLoading}>
      {isLoading ? "Generating..." : "Generate MIDI"}
    </button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if midiInfo}
    <div class="result">
      <MidiWidget {midiInfo} />
    </div>
  {/if}
</div>

<style>
  .page {
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    color: #ff79c6;
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: #6272a4;
    margin-bottom: 2rem;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .field {
    text-align: left;
  }

  select {
    width: 100%;
  }

  .error {
    padding: 0.75rem;
    background-color: #ff5555;
    color: #f8f8f2;
    border-radius: 6px;
    margin-bottom: 1rem;
  }

  .result {
    margin-top: 1.5rem;
  }
</style>
