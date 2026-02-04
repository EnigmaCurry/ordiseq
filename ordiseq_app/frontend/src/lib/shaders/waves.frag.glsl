#version 300 es
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
