import { writable } from "svelte/store";
import defaultFragmentShader from "./shaders/default.frag.glsl?raw";
import plasmaShader from "./shaders/plasma.frag.glsl?raw";
import wavesShader from "./shaders/waves.frag.glsl?raw";
import noiseShader from "./shaders/noise.frag.glsl?raw";

export interface ShaderConfig {
  fragmentShader: string;
  uniforms: Record<string, number | number[]>;
}

const defaultConfig: ShaderConfig = {
  fragmentShader: defaultFragmentShader,
  uniforms: {},
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
};
