import { writable, get } from "svelte/store";
import plasmaShader from "./shaders/plasma.frag.glsl?raw";
import noiseShader from "./shaders/noise.frag.glsl?raw";
import retrogridShader from "./shaders/retrogrid.frag.glsl?raw";
import matrixShader from "./shaders/matrix.frag.glsl?raw";
import synthwaveShader from "./shaders/synthwave.frag.glsl?raw";
import vhsShader from "./shaders/vhs.frag.glsl?raw";
import tunnelShader from "./shaders/tunnel.frag.glsl?raw";
import platonicShader from "./shaders/platonic.frag.glsl?raw";
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
  synthwave: synthwaveShader,
  vhs: vhsShader,
  tunnel: tunnelShader,
  platonic: platonicShader,
};

const defaultConfig: ShaderConfig = {
  fragmentShader: platonicShader,
  uniforms: {
    u_pitch: 0.0,
    u_speed: 0.02,
    u_zoom: 12.0,
    u_fisheye: 0.04,
    u_overlay: 0.69,
    u_bpm: 120.0,
    u_beat: 0.0,
    u_playing: 0.0,
    u_color1: [1.0, 0.70, 0.28],   // Sunset orange (#ffb347)
    u_color2: [1.0, 0.42, 0.42],   // Sunset coral (#ff6b6b)
    u_color3: [0.78, 0.49, 0.73],  // Sunset mauve (#c77dba)
    u_color4: [0.18, 0.11, 0.24],  // Sunset plum (#2d1b3d)
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

// Irrational frequency ratios so the combined signal never repeats
const PHI = 1.6180339887;    // golden ratio
const SQRT2 = 1.4142135624;
const SQRT3 = 1.7320508076;
const SQRT5 = 2.2360679775;
const E_FRAC = 0.7182818285; // e - 2

function smoothNoise(t: number, freq: number, phase: number): number {
  // 6 sine waves with mutually irrational frequency ratios — aperiodic.
  // Heavily weighted toward slow components for smooth, drifting motion.
  // Fastest harmonic is only ~1.44x base freq (PHI * E_FRAC ≈ 1.16).
  return (
    Math.sin(t * freq + phase) * 0.40 +
    Math.sin(t * freq * E_FRAC + phase * PHI) * 0.22 +
    Math.sin(t * freq * PHI * 0.5 + phase * SQRT2) * 0.15 +
    Math.sin(t * freq * SQRT2 * 0.5 + phase * E_FRAC) * 0.10 +
    Math.sin(t * freq * SQRT3 * 0.4 + phase * SQRT3) * 0.08 +
    Math.sin(t * freq * SQRT5 * 0.3 + phase * 0.3819660113) * 0.05
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

  // Each parameter uses different freq/phase seeds for independent motion
  const pitchOffset = smoothNoise(t, 0.1, 0.0) * 180;
  const zoomOffset = smoothNoise(t, 0.07, 7.3) * 8;
  const fisheyeOffset = smoothNoise(t, 0.13, 13.7) * 0.3;

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
export const selectedShaderName = writable<string>("platonic");

// Color theme presets
export interface ColorTheme {
  label: string;
  color1: string;
  color2: string;
  color3: string;
  color4: string;
}

// Color roles:
//   color1 = primary text & accents (headings, button text, slider thumbs)
//   color2 = secondary text (values, links, info)
//   color3 = tertiary (labels, borders, section headers) - used at various opacities
//   color4 = backgrounds (buttons, selects, inputs) - color1 must read clearly on this
export const colorThemes: Record<string, ColorTheme> = {
  synthwave:  { label: "Synthwave",   color1: "#ff1493", color2: "#00ffde", color3: "#b794f6", color4: "#1a1a2e" },
  outrun:     { label: "Outrun",      color1: "#ff6ec7", color2: "#7b68ee", color3: "#b8a9c9", color4: "#0d0221" },
  vapor:      { label: "Vaporwave",   color1: "#ff71ce", color2: "#01cdfe", color3: "#b967ff", color4: "#1b1235" },
  miami:      { label: "Miami Vice",  color1: "#f0c4e0", color2: "#4ecdc4", color3: "#a78bba", color4: "#1a3a4a" },
  tron:       { label: "Tron",        color1: "#6cffff", color2: "#ff8800", color3: "#7ca8c4", color4: "#0a0a1a" },
  neon:       { label: "Neon",        color1: "#39ff14", color2: "#00d4ff", color3: "#ff5e5e", color4: "#0a0a0a" },
  sunset:     { label: "Sunset",      color1: "#ffb347", color2: "#ff6b6b", color3: "#c77dba", color4: "#2d1b3d" },
  ember:      { label: "Ember",       color1: "#ff6633", color2: "#ffcc00", color3: "#b87850", color4: "#1a0e08" },
  infrared:   { label: "Infrared",    color1: "#ff3355", color2: "#ff8844", color3: "#b86655", color4: "#1a0808" },
  cherry:     { label: "Cherry",      color1: "#ff4466", color2: "#ffaacc", color3: "#b86b8a", color4: "#1a0a10" },
  sakura:     { label: "Sakura",      color1: "#ffb7c5", color2: "#d4a5a5", color3: "#9b7e7e", color4: "#1a1218" },
  candy:      { label: "Candy",       color1: "#ff99cc", color2: "#99ddff", color3: "#c4a0d0", color4: "#1a1020" },
  amber:      { label: "Amber",       color1: "#ffbf00", color2: "#ff8c00", color3: "#cc9544", color4: "#1a1000" },
  toxic:      { label: "Toxic",       color1: "#00ff88", color2: "#bf00ff", color3: "#8a7cb8", color4: "#0d1a0d" },
  jade:       { label: "Jade",        color1: "#00d68f", color2: "#a8e6cf", color3: "#6b9e8a", color4: "#0a1a14" },
  ocean:      { label: "Ocean",       color1: "#00bfff", color2: "#48d1cc", color3: "#6a8fa8", color4: "#0a1218" },
  ice:        { label: "Ice",         color1: "#b8e4ff", color2: "#7ec8e3", color3: "#5a8fa8", color4: "#0a1520" },
  midnight:   { label: "Midnight",    color1: "#6eb5ff", color2: "#e0e0e0", color3: "#8b7ec8", color4: "#0f1729" },
  slate:      { label: "Slate",       color1: "#94b8d0", color2: "#7a99b0", color3: "#607080", color4: "#121820" },
  monochrome: { label: "Monochrome",  color1: "#a0a0a0", color2: "#e0e0e0", color3: "#707070", color4: "#1a1a1a" },
  revelation: { label: "Revelation",   color1: "#ff3b3b", color2: "#ffffff", color3: "#9a8c98", color4: "#120a0a" },
  eden:       { label: "Eden",         color1: "#caffbf", color2: "#a0c4ff", color3: "#90a955", color4: "#0b1a0f" },
  gatsby:     { label: "Gatsby",       color1: "#ffd700", color2: "#50c878", color3: "#c0c0c0", color4: "#0f1a1a" },
  dracula:    { label: "Dracula",      color1: "#ff4d6d", color2: "#c0c0c0", color3: "#8d99ae", color4: "#120a12" },
  sherlock:   { label: "Sherlock",     color1: "#7aa6c2", color2: "#e0e0e0", color3: "#6c757d", color4: "#111417" },
  narnia:     { label: "Narnia",       color1: "#ffd166", color2: "#e6f7ff", color3: "#a8dadc", color4: "#0c1420" },
  dune:       { label: "Dune",         color1: "#f4a261", color2: "#e76f51", color3: "#b08968", color4: "#1a120a" },
  mordor:     { label: "Mordor",       color1: "#ff6b35", color2: "#c1121f", color3: "#6c584c", color4: "#140b0b" },
  odyssey:    { label: "Odyssey",      color1: "#ffd166", color2: "#118ab2", color3: "#8ecae6", color4: "#0a1620" },
  frankenstein:{ label: "Frankenstein",color1: "#8aff80", color2: "#c0c0c0", color3: "#7a918d", color4: "#0e1512" },
  inferno:    { label: "Inferno",      color1: "#ff3c38", color2: "#ff9f1c", color3: "#b85c38", color4: "#1a0a0a" },
    hal9000:     { label: "HAL 9000",       color1: "#ff3b3b", color2: "#ff9e9e", color3: "#a63c3c", color4: "#0b0b0f" },
  mother:      { label: "MU/TH/UR",       color1: "#b4ff39", color2: "#39ff14", color3: "#7aa66a", color4: "#0a140a" },
  skynet:      { label: "Skynet",         color1: "#ff5e5e", color2: "#00b3ff", color3: "#6c8ea3", color4: "#0a0f14" },
  wintermute:  { label: "Wintermute",     color1: "#7df9ff", color2: "#9d4edd", color3: "#6a8fa8", color4: "#0b1020" },
  neuromancer: { label: "Neuromancer",    color1: "#00ffd5", color2: "#ff00aa", color3: "#7b6c9d", color4: "#0a0f1a" },
  trinity:     { label: "Trinity",        color1: "#39ff14", color2: "#ffffff", color3: "#6e8f6e", color4: "#0a0f0a" },
  zion:        { label: "Zion Mainframe", color1: "#ffd166", color2: "#06d6a0", color3: "#b08968", color4: "#0f1414" },
  jarvis:      { label: "JARVIS",         color1: "#7fd8ff", color2: "#ffffff", color3: "#6c9db8", color4: "#0a1620" },
  friday:      { label: "FRIDAY",         color1: "#00e5ff", color2: "#ffd6ff", color3: "#7a8fa3", color4: "#0a141a" },
  glados:      { label: "GLaDOS",         color1: "#ffae00", color2: "#ffffff", color3: "#b08968", color4: "#121212" },
  deepthought: { label: "Deep Thought",   color1: "#ffd700", color2: "#00bfff", color3: "#8a9db0", color4: "#0f0f1a" },
  colossus:    { label: "Colossus",       color1: "#ff4d4d", color2: "#c0c0c0", color3: "#8d99ae", color4: "#120c12" },
  vger:        { label: "V'Ger",          color1: "#a0a0c0", color2: "#9bf6ff", color3: "#e0e0ff", color4: "#0a0f1f" },
  redqueen:    { label: "Red Queen",      color1: "#ff2e63", color2: "#08d9d6", color3: "#b86b8a", color4: "#140a12" },
  ed209:       { label: "ED-209",         color1: "#ff3c38", color2: "#7a7a7a", color3: "#8a8a8a", color4: "#111417" },
  wopr:        { label: "WOPR",           color1: "#00ff7f", color2: "#ff5555", color3: "#7a918d", color4: "#0a120f" },
  tachikoma:   { label: "Tachikoma",      color1: "#66ccff", color2: "#ffcc00", color3: "#7a9db8", color4: "#0a1420" },
  omnius:      { label: "Omnius",         color1: "#ff8844", color2: "#ffcc00", color3: "#b08968", color4: "#140f0a" },

  eniac:       { label: "ENIAC",          color1: "#ffbf00", color2: "#ff6f00", color3: "#c89b6d", color4: "#1a1208" },
  colossus_mk1:{ label: "Colossus Mk I",  color1: "#00ffcc", color2: "#a8dadc", color3: "#6c9db8", color4: "#0a1416" },
  univac:      { label: "UNIVAC",         color1: "#ffd6a5", color2: "#caffbf", color3: "#a0c4ff", color4: "#14120f" },
  xeroxalto:   { label: "Xerox Alto",     color1: "#7fd8ff", color2: "#e0e0e0", color3: "#94b8d0", color4: "#0f1720" },
  lisp_machine:{ label: "Lisp Machine",   color1: "#ff99cc", color2: "#cc99ff", color3: "#a78bba", color4: "#1a1020" },
  symbolics:   { label: "Symbolics",      color1: "#ffd166", color2: "#9bf6ff", color3: "#b8a9c9", color4: "#120f1f" },
  nextcube:    { label: "NeXTcube",       color1: "#00e5ff", color2: "#ffffff", color3: "#8d99ae", color4: "#0a0a0a" },
  amiga:       { label: "Amiga",          color1: "#ff00aa", color2: "#00ffd5", color3: "#b967ff", color4: "#140a18" },
  commodore64: { label: "Commodore 64",   color1: "#a0c4ff", color2: "#ffd6a5", color3: "#bdb2ff", color4: "#0a1020" },
  appleii:     { label: "Apple II",       color1: "#ff6b6b", color2: "#ffd93d", color3: "#6bcBef", color4: "#141010" },
  ibm5100:     { label: "IBM 5100",       color1: "#9ec1cf", color2: "#e0fbfc", color3: "#5c6b73", color4: "#0f1419" },
  thinkpad:    { label: "ThinkPad",       color1: "#ff2e2e", color2: "#e0e0e0", color3: "#8a8a8a", color4: "#0a0a0a" },
  vt100:       { label: "VT100",          color1: "#00ff66", color2: "#33ff99", color3: "#6e8f6e", color4: "#0a120a" },
  terminal:    { label: "Green Terminal", color1: "#39ff14", color2: "#7fff00", color3: "#6c9e6c", color4: "#050a05" },
  amberterm:   { label: "Amber Terminal", color1: "#ffb000", color2: "#ffd166", color3: "#c89b3c", color4: "#140f05" },
  cyberdeck:   { label: "Cyberdeck",      color1: "#00f5ff", color2: "#ff00aa", color3: "#7b6c9d", color4: "#0a0f18" },
  mainframe:   { label: "Mainframe",      color1: "#00bfff", color2: "#e0e0e0", color3: "#8b7ec8", color4: "#0f1729" },
};

export const selectedColorTheme = writable<string>("sunset");

/**
 * Pick a random shader and apply it. Returns the chosen shader key.
 */
export function randomizeShader(): string {
  const keys = Object.keys(exampleShaders);
  const key = keys[Math.floor(Math.random() * keys.length)];
  selectedShaderName.set(key);
  setShader(exampleShaders[key]);
  persistSettings();
  return key;
}

/**
 * Pick a random color theme and apply it. Returns the chosen theme key and color1 hex.
 */
export function randomizeTheme(): { key: string; color1: string } {
  const keys = Object.keys(colorThemes);
  const key = keys[Math.floor(Math.random() * keys.length)];
  const theme = colorThemes[key];
  selectedColorTheme.set(key);
  setUniforms({
    u_color1: hexToRgb(theme.color1),
    u_color2: hexToRgb(theme.color2),
    u_color3: hexToRgb(theme.color3),
    u_color4: hexToRgb(theme.color4),
  });
  persistSettings();
  return { key, color1: theme.color1 };
}

/**
 * Initialize settings from persistent storage
 */
export async function initializeSettings(): Promise<void> {
  try {
    const settings = await loadShaderSettings();

    // Apply shader
    const shader = exampleShaders[settings.selectedShader] || platonicShader;
    selectedShaderName.set(settings.selectedShader);
    selectedColorTheme.set(settings.colorTheme || "sunset");

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
    colorTheme: get(selectedColorTheme),
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
