import { writable } from "svelte/store";
import defaultFragmentShader from "./shaders/default.frag.glsl?raw";
import plasmaShader from "./shaders/plasma.frag.glsl?raw";
import wavesShader from "./shaders/waves.frag.glsl?raw";
import noiseShader from "./shaders/noise.frag.glsl?raw";
import retrogridShader from "./shaders/retrogrid.frag.glsl?raw";

export interface ShaderConfig {
  fragmentShader: string;
  uniforms: Record<string, number | number[]>;
}

const defaultConfig: ShaderConfig = {
  fragmentShader: retrogridShader,
  uniforms: {
    u_pitch: 285.0,
    u_speed: 0.02,
    u_zoom: 4.0,
    u_fisheye: 1.0,
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
