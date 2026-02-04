import { writable, get } from "svelte/store";
import defaultFragmentShader from "./shaders/default.frag.glsl?raw";
import plasmaShader from "./shaders/plasma.frag.glsl?raw";
import wavesShader from "./shaders/waves.frag.glsl?raw";
import noiseShader from "./shaders/noise.frag.glsl?raw";
import retrogridShader from "./shaders/retrogrid.frag.glsl?raw";

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

const defaultConfig: ShaderConfig = {
  fragmentShader: retrogridShader,
  uniforms: {
    u_pitch: 0.0,
    u_speed: 0.02,
    u_zoom: 12.0,
    u_fisheye: 0.04,
    u_overlay: 0.9,
    u_color1: [1.0, 0.08, 0.58],   // Hot pink
    u_color2: [0.0, 1.0, 0.87],    // Cyan
    u_color3: [0.74, 0.58, 0.98],  // Purple
    u_color4: [0.31, 0.98, 0.48],  // Green
  },
};

export const shaderConfig = writable<ShaderConfig>(defaultConfig);

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

// Example shaders loaded from .glsl files
export const exampleShaders = {
  default: defaultFragmentShader,
  plasma: plasmaShader,
  waves: wavesShader,
  noise: noiseShader,
  retrogrid: retrogridShader,
};

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
