#version 300 es
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
