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
  let pitch = $state(0.0);
  let speed = $state(0.02);
  let zoom = $state(12.0);
  let fisheye = $state(0.04);
  let overlay = $state(0.4);
  let color1 = $state("#ff1493");
  let color2 = $state("#00ffde");
  let color3 = $state("#bd93f9");
  let color4 = $state("#50fa7b");

  // Convert hex to RGB array (0-1 range)
  function hexToRgb(hex: string): number[] {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result
      ? [
          parseInt(result[1], 16) / 255,
          parseInt(result[2], 16) / 255,
          parseInt(result[3], 16) / 255,
        ]
      : [1, 1, 1];
  }

  // Convert RGB array (0-1 range) to hex
  function rgbToHex(rgb: number[]): string {
    const r = Math.round(rgb[0] * 255).toString(16).padStart(2, "0");
    const g = Math.round(rgb[1] * 255).toString(16).padStart(2, "0");
    const b = Math.round(rgb[2] * 255).toString(16).padStart(2, "0");
    return `#${r}${g}${b}`;
  }

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
    if (config.uniforms.u_zoom !== undefined) zoom = config.uniforms.u_zoom as number;
    if (config.uniforms.u_fisheye !== undefined) fisheye = config.uniforms.u_fisheye as number;
    if (config.uniforms.u_overlay !== undefined) overlay = config.uniforms.u_overlay as number;
    if (config.uniforms.u_color1 !== undefined) color1 = rgbToHex(config.uniforms.u_color1 as number[]);
    if (config.uniforms.u_color2 !== undefined) color2 = rgbToHex(config.uniforms.u_color2 as number[]);
    if (config.uniforms.u_color3 !== undefined) color3 = rgbToHex(config.uniforms.u_color3 as number[]);
    if (config.uniforms.u_color4 !== undefined) color4 = rgbToHex(config.uniforms.u_color4 as number[]);
  });

  function handleShaderChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const key = select.value as keyof typeof exampleShaders;
    selectedShader = key;
    setShader(exampleShaders[key]);

    // Set default uniforms for retrogrid
    if (key === "retrogrid") {
      setUniforms({
        u_pitch: pitch,
        u_speed: speed,
        u_zoom: zoom,
        u_fisheye: fisheye,
        u_overlay: overlay,
        u_color1: hexToRgb(color1),
        u_color2: hexToRgb(color2),
        u_color3: hexToRgb(color3),
        u_color4: hexToRgb(color4),
      });
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

  function updateZoom(event: Event) {
    const input = event.target as HTMLInputElement;
    zoom = parseFloat(input.value);
    setUniforms({ u_zoom: zoom });
  }

  function updateFisheye(event: Event) {
    const input = event.target as HTMLInputElement;
    fisheye = parseFloat(input.value);
    setUniforms({ u_fisheye: fisheye });
  }

  function updateOverlay(event: Event) {
    const input = event.target as HTMLInputElement;
    overlay = parseFloat(input.value);
    setUniforms({ u_overlay: overlay });
  }

  function updateColor(colorNum: number, event: Event) {
    const input = event.target as HTMLInputElement;
    const hex = input.value;
    const rgb = hexToRgb(hex);
    if (colorNum === 1) { color1 = hex; setUniforms({ u_color1: rgb }); }
    else if (colorNum === 2) { color2 = hex; setUniforms({ u_color2: rgb }); }
    else if (colorNum === 3) { color3 = hex; setUniforms({ u_color3: rgb }); }
    else { color4 = hex; setUniforms({ u_color4: rgb }); }
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
            <span class="value">{pitch.toFixed(0)}°</span>
          </label>
          <input
            type="range"
            id="pitch"
            min="0"
            max="360"
            step="1"
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

        <div class="field">
          <label for="zoom">
            Zoom
            <span class="value">{zoom.toFixed(2)}</span>
          </label>
          <input
            type="range"
            id="zoom"
            min="0.5"
            max="32"
            step="0.01"
            value={zoom}
            oninput={updateZoom}
          />
        </div>

        <div class="field">
          <label for="fisheye">
            Fisheye
            <span class="value">{fisheye.toFixed(2)}</span>
          </label>
          <input
            type="range"
            id="fisheye"
            min="0"
            max="1"
            step="0.01"
            value={fisheye}
            oninput={updateFisheye}
          />
        </div>

        <div class="field">
          <label for="overlay">
            Overlay
            <span class="value">{overlay.toFixed(2)}</span>
          </label>
          <input
            type="range"
            id="overlay"
            min="0"
            max="1"
            step="0.01"
            value={overlay}
            oninput={updateOverlay}
          />
        </div>

        <div class="color-grid">
          <div class="color-field">
            <label for="color1">Color 1</label>
            <input
              type="color"
              id="color1"
              value={color1}
              oninput={(e) => updateColor(1, e)}
            />
          </div>
          <div class="color-field">
            <label for="color2">Color 2</label>
            <input
              type="color"
              id="color2"
              value={color2}
              oninput={(e) => updateColor(2, e)}
            />
          </div>
          <div class="color-field">
            <label for="color3">Color 3</label>
            <input
              type="color"
              id="color3"
              value={color3}
              oninput={(e) => updateColor(3, e)}
            />
          </div>
          <div class="color-field">
            <label for="color4">Color 4</label>
            <input
              type="color"
              id="color4"
              value={color4}
              oninput={(e) => updateColor(4, e)}
            />
          </div>
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
    color: rgb(var(--color-1));
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: rgba(var(--color-3), 0.7);
    margin-bottom: 2rem;
  }

  .settings-section {
    text-align: left;
    background-color: rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 8px;
    padding: 1.5rem;
  }

  h2 {
    font-size: 1.25rem;
    color: rgb(var(--color-3));
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
    border-top: 1px solid rgba(var(--color-3), 0.5);
  }

  label {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .value {
    font-family: monospace;
    color: rgb(var(--color-2));
    font-size: 0.9rem;
  }

  input[type="range"] {
    width: 100%;
    height: 6px;
    margin-top: 0.5rem;
    background: rgba(var(--color-3), 0.3);
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
    background: rgb(var(--color-1));
    border-radius: 50%;
    cursor: pointer;
  }

  input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    background: rgb(var(--color-1));
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .color-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    margin-top: 1rem;
  }

  .color-field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .color-field label {
    font-size: 0.9rem;
  }

  input[type="color"] {
    width: 100%;
    height: 40px;
    border: 1px solid rgba(var(--color-3), 0.5);
    border-radius: 6px;
    cursor: pointer;
    background: none;
    padding: 2px;
  }

  input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius: 4px;
  }

  input[type="color"]::-moz-color-swatch {
    border: none;
    border-radius: 4px;
  }
</style>
