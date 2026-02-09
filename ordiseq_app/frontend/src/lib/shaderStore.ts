import { writable, get } from "svelte/store";
import plasmaShader from "./shaders/plasma.frag.glsl?raw";
import noiseShader from "./shaders/noise.frag.glsl?raw";
import retrogridShader from "./shaders/retrogrid.frag.glsl?raw";
import matrixShader from "./shaders/matrix.frag.glsl?raw";
import { loadShaderSettings, saveShaderSettings, type ShaderSettings } from "./settingsStore";

export interface ShaderConfig {
  fragmentShader: string;
  uniforms: Record<string, number | number[]>;
}

export interface AnimationConfig {
  enabled: boolean;
  basePitch: number;
  baseZoom: number;
  baseFisheye: number;
}

// Example shaders loaded from .glsl files
export const exampleShaders: Record<string, string> = {
  plasma: plasmaShader,
  noise: noiseShader,
  retrogrid: retrogridShader,
  matrix: matrixShader,
};

const defaultConfig: ShaderConfig = {
  fragmentShader: plasmaShader,
  uniforms: {
    u_pitch: 0.0,
    u_speed: 0.02,
    u_zoom: 12.0,
    u_fisheye: 0.04,
    u_overlay: 0.9,
    u_bpm: 120.0,
    u_beat: 0.0,
    u_playing: 0.0,
    u_color1: [1.0, 0.08, 0.58],   // Hot pink
    u_color2: [0.0, 1.0, 0.87],    // Cyan
    u_color3: [0.74, 0.58, 0.98],  // Purple
    u_color4: [0.31, 0.98, 0.48],  // Green
  },
};

export const shaderConfig = writable<ShaderConfig>(defaultConfig);
export const settingsLoaded = writable<boolean>(false);

/**
 * Set a new fragment shader. Must be a valid GLSL ES 3.0 shader.
 * Available built-in uniforms: u_time (float), u_resolution (vec2)
 */
export function setShader(fragmentShader: string): void {
  shaderConfig.update((config) => ({
    ...config,
    fragmentShader,
  }));
}

/**
 * Set uniform values. Supports float, vec2, vec3, vec4.
 */
export function setUniforms(uniforms: Record<string, number | number[]>): void {
  shaderConfig.update((config) => ({
    ...config,
    uniforms: { ...config.uniforms, ...uniforms },
  }));
}

/**
 * Reset to default shader
 */
export function resetShader(): void {
  shaderConfig.set(defaultConfig);
}

// Animation state
export const animationConfig = writable<AnimationConfig>({
  enabled: true,
  basePitch: 0.0,
  baseZoom: 12.0,
  baseFisheye: 0.04,
});

let animationFrame: number | null = null;
let startTime = performance.now();

function smoothNoise(t: number, freq: number, phase: number): number {
  // Combine multiple sine waves for organic movement
  return (
    Math.sin(t * freq + phase) * 0.5 +
    Math.sin(t * freq * 0.7 + phase * 1.3) * 0.3 +
    Math.sin(t * freq * 1.3 + phase * 0.7) * 0.2
  );
}

function animationLoop() {
  const config = get(shaderConfig);
  const anim = get(animationConfig);

  if (!anim.enabled) {
    animationFrame = requestAnimationFrame(animationLoop);
    return;
  }

  const speed = (config.uniforms.u_speed as number) || 0.02;
  const elapsed = (performance.now() - startTime) / 1000;
  const t = elapsed * speed * 10;

  // Calculate animated values with different frequencies and phases
  const pitchOffset = smoothNoise(t, 0.1, 0) * 180; // +/- 180 degrees
  const zoomOffset = smoothNoise(t, 0.07, 2.5) * 8; // +/- 8 zoom
  const fisheyeOffset = smoothNoise(t, 0.13, 5.0) * 0.3; // +/- 0.3

  const newPitch = ((anim.basePitch + pitchOffset) % 360 + 360) % 360;
  const newZoom = Math.max(0.5, Math.min(32, anim.baseZoom + zoomOffset));
  const newFisheye = Math.max(0, Math.min(1, anim.baseFisheye + fisheyeOffset));

  shaderConfig.update((c) => ({
    ...c,
    uniforms: {
      ...c.uniforms,
      u_pitch: newPitch,
      u_zoom: newZoom,
      u_fisheye: newFisheye,
    },
  }));

  animationFrame = requestAnimationFrame(animationLoop);
}

export function startAnimation() {
  if (animationFrame === null) {
    startTime = performance.now();
    animationFrame = requestAnimationFrame(animationLoop);
  }
}

export function stopAnimation() {
  if (animationFrame !== null) {
    cancelAnimationFrame(animationFrame);
    animationFrame = null;
  }
}

export function setAnimationEnabled(enabled: boolean) {
  animationConfig.update((c) => ({ ...c, enabled }));
}

export function setAnimationBase(pitch: number, zoom: number, fisheye: number) {
  animationConfig.update((c) => ({
    ...c,
    basePitch: pitch,
    baseZoom: zoom,
    baseFisheye: fisheye,
  }));
}

// Start animation by default
startAnimation();

// Helper functions for color conversion
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

function rgbToHex(rgb: number[]): string {
  const r = Math.round(rgb[0] * 255).toString(16).padStart(2, "0");
  const g = Math.round(rgb[1] * 255).toString(16).padStart(2, "0");
  const b = Math.round(rgb[2] * 255).toString(16).padStart(2, "0");
  return `#${r}${g}${b}`;
}

// Track which shader is currently selected by name
export const selectedShaderName = writable<string>("plasma");

/**
 * Initialize settings from persistent storage
 */
export async function initializeSettings(): Promise<void> {
  try {
    const settings = await loadShaderSettings();

    // Apply shader
    const shader = exampleShaders[settings.selectedShader] || plasmaShader;
    selectedShaderName.set(settings.selectedShader);

    // Apply uniforms with color conversion
    shaderConfig.set({
      fragmentShader: shader,
      uniforms: {
        u_pitch: settings.uniforms.pitch,
        u_speed: settings.uniforms.speed,
        u_zoom: settings.uniforms.zoom,
        u_fisheye: settings.uniforms.fisheye,
        u_overlay: settings.uniforms.overlay,
        u_bpm: 120.0,
        u_beat: 0.0,
        u_playing: 0.0,
        u_color1: hexToRgb(settings.uniforms.color1),
        u_color2: hexToRgb(settings.uniforms.color2),
        u_color3: hexToRgb(settings.uniforms.color3),
        u_color4: hexToRgb(settings.uniforms.color4),
      },
    });

    // Apply animation config
    animationConfig.set({
      enabled: settings.animationEnabled,
      basePitch: settings.uniforms.pitch,
      baseZoom: settings.uniforms.zoom,
      baseFisheye: settings.uniforms.fisheye,
    });

    settingsLoaded.set(true);
  } catch (e) {
    console.warn("Failed to initialize settings:", e);
    settingsLoaded.set(true);
  }
}

/**
 * Save current settings to persistent storage
 */
export async function persistSettings(): Promise<void> {
  const config = get(shaderConfig);
  const anim = get(animationConfig);
  const shaderName = get(selectedShaderName);

  const settings: ShaderSettings = {
    selectedShader: shaderName,
    uniforms: {
      pitch: anim.basePitch,
      speed: config.uniforms.u_speed as number,
      zoom: anim.baseZoom,
      fisheye: anim.baseFisheye,
      overlay: config.uniforms.u_overlay as number,
      color1: rgbToHex(config.uniforms.u_color1 as number[]),
      color2: rgbToHex(config.uniforms.u_color2 as number[]),
      color3: rgbToHex(config.uniforms.u_color3 as number[]),
      color4: rgbToHex(config.uniforms.u_color4 as number[]),
    },
    animationEnabled: anim.enabled,
  };

  await saveShaderSettings(settings);
}
