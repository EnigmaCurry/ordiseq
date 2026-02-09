#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform float u_bpm;
uniform vec3 u_color1;
uniform vec3 u_color2;
uniform vec3 u_color3;
uniform vec3 u_color4;
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
  float tempo = u_bpm / 120.0;
  float t = u_time * 0.2 * tempo;

  float n = noise(uv * 5.0 + t) * 0.5;
  n += noise(uv * 10.0 - t * 0.5) * 0.25;
  n += noise(uv * 20.0 + t * 0.3) * 0.125;

  // Cycle through 4 colors based on noise value
  float idx = n * 4.0;
  vec3 col;
  if (idx < 1.0) col = mix(u_color1, u_color2, idx);
  else if (idx < 2.0) col = mix(u_color2, u_color3, idx - 1.0);
  else if (idx < 3.0) col = mix(u_color3, u_color4, idx - 2.0);
  else col = mix(u_color4, u_color1, idx - 3.0);

  fragColor = vec4(col * 0.4, 1.0);
}
