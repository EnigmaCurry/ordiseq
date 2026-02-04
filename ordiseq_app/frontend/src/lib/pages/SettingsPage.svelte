<script lang="ts">
  import { setShader, setUniforms, exampleShaders, shaderConfig } from "../shaderStore";

  const shaderOptions = [
    { value: "retrogrid", label: "Retro Grid" },
    { value: "default", label: "Default" },
    { value: "plasma", label: "Plasma" },
    { value: "waves", label: "Waves" },
    { value: "noise", label: "Noise" },
  ];

  let selectedShader = $state("retrogrid");

  // Retrogrid settings
  let pitch = $state(0.20);
  let speed = $state(0.16);

  // Sync initial state with store
  $effect(() => {
    const config = $shaderConfig;
    for (const [key, value] of Object.entries(exampleShaders)) {
      if (value === config.fragmentShader) {
        selectedShader = key;
        break;
      }
    }
    // Sync retrogrid uniforms
    if (config.uniforms.u_pitch !== undefined) pitch = config.uniforms.u_pitch as number;
    if (config.uniforms.u_speed !== undefined) speed = config.uniforms.u_speed as number;
  });

  function handleShaderChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const key = select.value as keyof typeof exampleShaders;
    selectedShader = key;
    setShader(exampleShaders[key]);

    // Set default uniforms for retrogrid
    if (key === "retrogrid") {
      setUniforms({ u_pitch: pitch, u_speed: speed });
    }
  }

  function updatePitch(event: Event) {
    const input = event.target as HTMLInputElement;
    pitch = parseFloat(input.value);
    setUniforms({ u_pitch: pitch });
  }

  function updateSpeed(event: Event) {
    const input = event.target as HTMLInputElement;
    speed = parseFloat(input.value);
    setUniforms({ u_speed: speed });
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

    {#if selectedShader === "retrogrid"}
      <div class="shader-settings">
        <div class="field">
          <label for="pitch">
            Camera Pitch
            <span class="value">{pitch.toFixed(2)}</span>
          </label>
          <input
            type="range"
            id="pitch"
            min="0.1"
            max="0.9"
            step="0.01"
            value={pitch}
            oninput={updatePitch}
          />
        </div>

        <div class="field">
          <label for="speed">
            Speed
            <span class="value">{speed.toFixed(2)}</span>
          </label>
          <input
            type="range"
            id="speed"
            min="0"
            max="2"
            step="0.01"
            value={speed}
            oninput={updateSpeed}
          />
        </div>
      </div>
    {/if}
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

  .shader-settings {
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid #6272a4;
  }

  label {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .value {
    font-family: monospace;
    color: #8be9fd;
    font-size: 0.9rem;
  }

  input[type="range"] {
    width: 100%;
    height: 6px;
    margin-top: 0.5rem;
    background: #44475a;
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
    appearance: none;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    background: #ff79c6;
    border-radius: 50%;
    cursor: pointer;
  }

  input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    background: #ff79c6;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }
</style>
