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

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float tempo = u_bpm / 120.0;
  float t = u_time * 0.5 * tempo;

  float v = 0.0;
  v += sin((uv.x * 10.0 + t));
  v += sin((uv.y * 10.0 + t) / 2.0);
  v += sin((uv.x * 10.0 + uv.y * 10.0 + t) / 2.0);

  // Map plasma value (-3..3) to 0..1, then cycle through 4 colors
  float n = (v + 3.0) / 6.0;
  float idx = n * 4.0;
  vec3 col;
  if (idx < 1.0) col = mix(u_color1, u_color2, idx);
  else if (idx < 2.0) col = mix(u_color2, u_color3, idx - 1.0);
  else if (idx < 3.0) col = mix(u_color3, u_color4, idx - 2.0);
  else col = mix(u_color4, u_color1, idx - 3.0);

  fragColor = vec4(col * 0.4, 1.0);
}
