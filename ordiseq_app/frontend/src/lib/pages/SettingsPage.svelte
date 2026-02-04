<script lang="ts">
  import { setShader, exampleShaders, shaderConfig } from "../shaderStore";

  const shaderOptions = [
    { value: "default", label: "Default" },
    { value: "plasma", label: "Plasma" },
    { value: "waves", label: "Waves" },
    { value: "noise", label: "Noise" },
  ];

  let selectedShader = $state("default");

  // Sync initial state with store
  $effect(() => {
    const currentShader = $shaderConfig.fragmentShader;
    for (const [key, value] of Object.entries(exampleShaders)) {
      if (value === currentShader) {
        selectedShader = key;
        break;
      }
    }
  });

  function handleShaderChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const key = select.value as keyof typeof exampleShaders;
    selectedShader = key;
    setShader(exampleShaders[key]);
  }
</script>

<div class="page">
  <h1>Settings</h1>
  <p class="subtitle">Application Configuration</p>

  <div class="settings-section">
    <h2>Appearance</h2>

    <div class="field">
      <label for="shader">Background Shader</label>
      <select id="shader" value={selectedShader} onchange={handleShaderChange}>
        {#each shaderOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>
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

  .settings-section {
    text-align: left;
    background-color: rgba(68, 71, 90, 0.5);
    border: 1px solid #6272a4;
    border-radius: 8px;
    padding: 1.5rem;
  }

  h2 {
    font-size: 1.25rem;
    color: #bd93f9;
    margin-bottom: 1rem;
  }

  .field {
    margin-bottom: 1rem;
  }

  .field:last-child {
    margin-bottom: 0;
  }

  select {
    width: 100%;
  }
</style>
