#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform float u_bpm;
uniform float u_beat;
uniform float u_playing;
uniform vec3 u_color1;
uniform vec3 u_color2;
uniform vec3 u_color3;
uniform vec3 u_color4;
out vec4 fragColor;

float hash(float n) {
  return fract(sin(n) * 43758.5453);
}

float hash2(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// Smooth noise
float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash2(i);
  float b = hash2(i + vec2(1.0, 0.0));
  float c = hash2(i + vec2(0.0, 1.0));
  float d = hash2(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float tempo = u_bpm / 120.0;
  float t = mix(u_time * tempo, u_beat, u_playing);

  // --- Tracking wobble: horizontal offset that glitches periodically ---
  float trackingPhase = t * 0.7;
  float trackingBand = smoothstep(0.0, 0.05, abs(uv.y - fract(trackingPhase))) *
                       smoothstep(0.0, 0.05, abs(uv.y - fract(trackingPhase + 0.5)));
  // Occasional heavy tracking distortion
  float glitchTrigger = step(0.92, hash(floor(t * 2.0)));
  float glitchY = hash(floor(t * 4.0));
  float glitchBand = smoothstep(0.0, 0.03, abs(uv.y - glitchY)) *
                     smoothstep(0.0, 0.08, abs(uv.y - glitchY - 0.05));
  float wobble = (1.0 - trackingBand) * 0.01 + (1.0 - glitchBand) * glitchTrigger * 0.06;

  // Apply wobble to UV
  vec2 distUV = uv;
  distUV.x += wobble * sin(uv.y * 100.0 + t * 10.0);

  // --- Chromatic aberration ---
  float aberr = 0.003 + glitchTrigger * 0.01;
  float r = noise(vec2(distUV.x + aberr, distUV.y) * 4.0 + t * 0.5);
  float g = noise(vec2(distUV.x, distUV.y) * 4.0 + t * 0.5);
  float b = noise(vec2(distUV.x - aberr, distUV.y) * 4.0 + t * 0.5);

  // --- Colorize: cycle noise through user colors ---
  float n = noise(distUV * 3.0 + t * 0.3);
  float idx = n * 4.0;
  vec3 baseCol;
  if (idx < 1.0) baseCol = mix(u_color1, u_color2, fract(idx));
  else if (idx < 2.0) baseCol = mix(u_color2, u_color3, fract(idx));
  else if (idx < 3.0) baseCol = mix(u_color3, u_color4, fract(idx));
  else baseCol = mix(u_color4, u_color1, fract(idx));

  // Mix chromatic aberration with base color
  vec3 col = baseCol * vec3(0.6 + r * 0.8, 0.6 + g * 0.8, 0.6 + b * 0.8);

  // --- Scanlines ---
  float scanline = 0.85 + 0.15 * sin(gl_FragCoord.y * 2.5);
  col *= scanline;

  // --- Horizontal noise bars (static bands) ---
  float staticBar = noise(vec2(0.0, uv.y * 80.0 + t * 20.0));
  col += vec3(staticBar * 0.03);

  // --- Bottom tracking bar (VHS head switch) ---
  float headSwitch = smoothstep(0.0, 0.02, uv.y) * smoothstep(0.06, 0.03, uv.y);
  float headNoise = hash(floor(t * 30.0) + uv.x * 10.0);
  col = mix(col, vec3(headNoise) * baseCol, headSwitch * 0.8);

  // --- Rolling bar (brightness wave) ---
  float roll = sin(uv.y * 6.28 - t * 1.5) * 0.5 + 0.5;
  col *= 0.85 + 0.15 * roll;

  // --- Vignette ---
  vec2 vig = uv - 0.5;
  col *= 1.0 - dot(vig, vig) * 0.8;

  // --- CRT curvature (subtle) ---
  // Darken edges to simulate curved screen
  float edgeDist = max(abs(uv.x - 0.5), abs(uv.y - 0.5));
  col *= smoothstep(0.55, 0.45, edgeDist);

  fragColor = vec4(col * 0.45, 1.0);
}
