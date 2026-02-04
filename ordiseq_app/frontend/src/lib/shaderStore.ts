import { writable } from "svelte/store";

export interface ShaderConfig {
  fragmentShader: string;
  uniforms: Record<string, number | number[]>;
}

const defaultFragmentShader = `#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Subtle animated gradient
  float t = u_time * 0.1;
  vec3 color1 = vec3(0.157, 0.165, 0.212); // #282a36
  vec3 color2 = vec3(0.267, 0.278, 0.353); // #44475a

  float noise = sin(uv.x * 10.0 + t) * sin(uv.y * 10.0 + t * 0.7) * 0.5 + 0.5;
  float wave = sin(uv.x * 3.0 + uv.y * 2.0 + t) * 0.5 + 0.5;

  vec3 color = mix(color1, color2, wave * 0.3 + noise * 0.1);
  fragColor = vec4(color, 1.0);
}
`;

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

// Example shaders for reference
export const exampleShaders = {
  plasma: `#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float t = u_time * 0.5;

  float v = 0.0;
  v += sin((uv.x * 10.0 + t));
  v += sin((uv.y * 10.0 + t) / 2.0);
  v += sin((uv.x * 10.0 + uv.y * 10.0 + t) / 2.0);

  vec3 col = vec3(0.157, 0.165, 0.212);
  col += 0.1 * vec3(sin(v), sin(v + 2.094), sin(v + 4.188));

  fragColor = vec4(col, 1.0);
}
`,

  waves: `#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float t = u_time * 0.3;

  float wave1 = sin(uv.x * 8.0 + t) * 0.5 + 0.5;
  float wave2 = sin(uv.y * 6.0 + t * 1.3) * 0.5 + 0.5;
  float wave3 = sin((uv.x + uv.y) * 4.0 + t * 0.7) * 0.5 + 0.5;

  vec3 base = vec3(0.157, 0.165, 0.212);
  vec3 accent = vec3(0.267, 0.278, 0.353);

  float blend = (wave1 + wave2 + wave3) / 3.0;
  vec3 color = mix(base, accent, blend * 0.4);

  fragColor = vec4(color, 1.0);
}
`,

  noise: `#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x),
    mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x),
    f.y
  );
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float t = u_time * 0.2;

  float n = noise(uv * 5.0 + t) * 0.5;
  n += noise(uv * 10.0 - t * 0.5) * 0.25;
  n += noise(uv * 20.0 + t * 0.3) * 0.125;

  vec3 base = vec3(0.157, 0.165, 0.212);
  vec3 accent = vec3(0.267, 0.278, 0.353);

  vec3 color = mix(base, accent, n * 0.6);
  fragColor = vec4(color, 1.0);
}
`,
};
