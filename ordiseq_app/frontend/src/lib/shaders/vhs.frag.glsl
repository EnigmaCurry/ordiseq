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

#define PI 3.14159265359

float hash(float n) {
  return fract(sin(n) * 43758.5453);
}

float hash2(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

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

float fbm(vec2 p) {
  float v = 0.0;
  float a = 0.5;
  mat2 rot = mat2(0.8, 0.6, -0.6, 0.8);
  for (int i = 0; i < 4; i++) {
    v += a * noise(p);
    p = rot * p * 2.0;
    a *= 0.5;
  }
  return v;
}

// Palette lookup helper
vec3 palette(float idx) {
  idx = fract(idx) * 4.0;
  if (idx < 1.0) return mix(u_color1, u_color2, idx);
  else if (idx < 2.0) return mix(u_color2, u_color3, idx - 1.0);
  else if (idx < 3.0) return mix(u_color3, u_color4, idx - 2.0);
  else return mix(u_color4, u_color1, idx - 3.0);
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float tempo = u_bpm / 120.0;
  float t = mix(u_time * tempo, u_beat, u_playing);

  // --- CRT barrel distortion ---
  vec2 crtUV = uv * 2.0 - 1.0;
  float barrel = 0.12;
  float r2 = dot(crtUV, crtUV);
  crtUV *= 1.0 + barrel * r2;
  vec2 curvedUV = crtUV * 0.5 + 0.5;
  // Outside the curved screen = black
  float offScreen = step(0.0, curvedUV.x) * step(curvedUV.x, 1.0)
                  * step(0.0, curvedUV.y) * step(curvedUV.y, 1.0);

  // --- Multi-layer glitch triggers ---
  float glitch1 = step(0.90, hash(floor(t * 2.0)));
  float glitch2 = step(0.95, hash(floor(t * 3.0 + 7.0)));
  float glitch3 = step(0.97, hash(floor(t * 1.5 + 13.0)));

  // --- Vertical hold drift: image slowly rolls then snaps back ---
  float holdPhase = t * 0.04;
  float holdDrift = fract(sin(holdPhase * 0.7) * 0.5 + holdPhase * 0.02);
  float vHold = holdDrift * 0.06 * sin(t * 0.3);
  // Heavy glitch can cause a big vertical jump
  vHold += glitch3 * (hash(floor(t * 2.0 + 5.0)) - 0.5) * 0.15;

  // --- Tracking wobble ---
  float trackingPhase = t * 0.7;
  float trackBand1 = 1.0 - smoothstep(0.0, 0.05, abs(curvedUV.y - fract(trackingPhase)));
  float trackBand2 = 1.0 - smoothstep(0.0, 0.04, abs(curvedUV.y - fract(trackingPhase * 1.3 + 0.33)));

  // Glitch bands
  float glitchY1 = hash(floor(t * 4.0));
  float glitchY2 = hash(floor(t * 6.0 + 3.0));
  float glitchY3 = hash(floor(t * 2.5 + 9.0));
  float glitchBand1 = (1.0 - smoothstep(0.0, 0.03, abs(curvedUV.y - glitchY1))) *
                      smoothstep(0.0, 0.08, abs(curvedUV.y - glitchY1 - 0.05));
  float glitchBand2 = (1.0 - smoothstep(0.0, 0.05, abs(curvedUV.y - glitchY2))) *
                      smoothstep(0.0, 0.12, abs(curvedUV.y - glitchY2 - 0.08));
  float glitchBand3 = (1.0 - smoothstep(0.0, 0.02, abs(curvedUV.y - glitchY3))) *
                      smoothstep(0.0, 0.15, abs(curvedUV.y - glitchY3 - 0.1));

  float wobble = trackBand1 * 0.008 + trackBand2 * 0.006
               + glitchBand1 * glitch1 * 0.04
               + glitchBand2 * glitch2 * 0.07
               + glitchBand3 * glitch3 * 0.12;

  // --- Horizontal tear ---
  float tearOffset = 0.0;
  if (glitch3 > 0.0) {
    float tearY = hash(floor(t * 2.0 + 5.0));
    float tearBand = step(tearY, curvedUV.y) * step(curvedUV.y, tearY + 0.08);
    tearOffset = tearBand * (hash(floor(t * 8.0)) - 0.5) * 0.2;
  }

  // Apply distortions
  vec2 distUV = curvedUV;
  distUV.y = fract(distUV.y + vHold); // vertical hold
  distUV.x += wobble * sin(curvedUV.y * 100.0 + t * 10.0) + tearOffset;

  // --- Wandering drift direction (magnetic shift) ---
  // Layered sines at incommensurate frequencies create a slowly wandering angle
  float driftAngle = sin(t * 0.011) * 2.5
                   + sin(t * 0.0047 + 1.0) * 1.8
                   + sin(t * 0.0023 + 4.0) * 1.0;
  // Varying drift speed
  float driftSpeed = 0.35 + 0.15 * sin(t * 0.007 + 2.0);
  vec2 drift = vec2(cos(driftAngle), sin(driftAngle)) * driftSpeed * t;

  // --- Composite video color bleed: horizontal smear ---
  float bleedAmt = 0.008 + glitch1 * 0.015;
  vec2 bleedUV1 = distUV + vec2(bleedAmt, 0.0);
  vec2 bleedUV2 = distUV - vec2(bleedAmt, 0.0);

  // --- Chromatic aberration ---
  float aberr = 0.003 + glitch1 * 0.008 + glitch2 * 0.015 + glitch3 * 0.03;

  // Sample FBM plasma at three offset positions with wandering drift
  float lumC = fbm(distUV * 4.0 + drift * 1.2);
  float lumL = fbm(bleedUV2 * 4.0 + drift * 1.2);
  float lumR = fbm(bleedUV1 * 4.0 + drift * 1.2);

  // --- Colorize through user palette ---
  float n = fbm(distUV * 3.0 + drift);
  vec3 baseCol = palette(n);

  // Palette-safe aberration: brightness + color bleed from neighboring palette regions
  float aberrLum = (lumL + lumC + lumR) / 3.0;
  vec3 bleedColL = palette(n - aberr * 8.0);
  vec3 bleedColR = palette(n + aberr * 8.0);
  vec3 col = mix(baseCol, bleedColL, 0.15) * (0.4 + lumL * 0.4)
           + mix(baseCol, baseCol, 0.7) * (0.4 + lumC * 0.4)
           + mix(baseCol, bleedColR, 0.15) * (0.4 + lumR * 0.4);
  col /= 1.8; // normalize

  // --- Color posterization (reduced bit depth like old video) ---
  float posterLevels = 12.0 - glitch2 * 6.0; // drops to 6 levels during glitch
  col = floor(col * posterLevels + 0.5) / posterLevels;

  // --- Palette color shift during glitch ---
  if (glitch2 > 0.0) {
    float dropChoice = hash(floor(t * 5.0 + 2.0));
    vec3 shiftColor = palette(dropChoice);
    col = mix(col, shiftColor * (0.5 + aberrLum * 0.5), 0.6);
  }

  // --- Soft scanlines (wide, not per-pixel) ---
  float scanline = 0.90 + 0.10 * sin(curvedUV.y * u_resolution.y * 0.5);
  col *= scanline;

  // --- RF interference: broad diagonal waves ---
  float rf = sin((curvedUV.x + curvedUV.y) * 20.0 + t * 5.0) * 0.5 + 0.5;
  rf *= sin((curvedUV.x - curvedUV.y * 0.5) * 30.0 - t * 3.0) * 0.5 + 0.5;
  col += palette(curvedUV.y + t * 0.1) * rf * 0.03;

  // --- Static bursts during glitches (thick bands) ---
  if (glitch1 > 0.0) {
    float burstY = hash(floor(t * 3.0 + 11.0));
    float burstBand = smoothstep(burstY - 0.01, burstY, curvedUV.y)
                    * smoothstep(burstY + 0.05, burstY + 0.04, curvedUV.y);
    float burstNoise = hash2(vec2(gl_FragCoord.x * 0.3, floor(t * 25.0)));
    vec3 burstTint = palette(hash(floor(t * 3.0 + 17.0)));
    col = mix(col, burstTint * burstNoise, burstBand * 0.8);
  }
  // Second burst band at a different position
  if (glitch2 > 0.0) {
    float burst2Y = hash(floor(t * 2.0 + 19.0));
    float burst2Band = smoothstep(burst2Y - 0.01, burst2Y, curvedUV.y)
                     * smoothstep(burst2Y + 0.07, burst2Y + 0.06, curvedUV.y);
    float burst2Noise = hash2(vec2(gl_FragCoord.x * 0.2, floor(t * 20.0) + 5.0));
    vec3 burst2Tint = palette(hash(floor(t * 2.0 + 23.0)));
    col = mix(col, burst2Tint * burst2Noise, burst2Band * 0.7);
  }

  // --- Snow: per-pixel white noise like no-signal TV ---
  float snow = hash2(gl_FragCoord.xy * 0.1 + floor(t * 60.0) * 7.0);
  float snowAmt = 0.03 + glitch3 * 0.25; // heavy snow during big glitches
  col = mix(col, palette(snow) * snow, snowAmt);

  // --- Horizontal noise bars ---
  float staticBar = noise(vec2(0.0, curvedUV.y * 80.0 + t * 20.0));
  float staticBar2 = noise(vec2(3.0, curvedUV.y * 120.0 - t * 15.0));
  col += baseCol * (staticBar * 0.03 + staticBar2 * 0.02);

  // --- VHS head switch noise at bottom ---
  float headSwitch = smoothstep(0.0, 0.02, curvedUV.y) * smoothstep(0.06, 0.03, curvedUV.y);
  float headNoise = hash(floor(t * 30.0) + curvedUV.x * 10.0);
  col = mix(col, baseCol * headNoise, headSwitch * 0.8);

  // --- Rolling brightness bar ---
  float roll = sin(curvedUV.y * 6.28 - t * 1.5) * 0.5 + 0.5;
  col *= 0.85 + 0.15 * roll;

  // --- CRT bloom/glow: bright areas bleed light ---
  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  float bloom = smoothstep(0.35, 0.7, lum);
  col += baseCol * bloom * 0.12;

  // --- Frame jitter during heavy glitch ---
  if (glitch3 > 0.0) {
    float jitter = (hash(floor(t * 10.0)) - 0.5) * 0.03;
    col *= 0.85 + 0.3 * noise(curvedUV * 50.0 + jitter);
  }

  // --- Ghosting: faint afterimage shifted right ---
  float ghostN = fbm((distUV - vec2(0.02, 0.005)) * 3.0 + t * 0.3);
  vec3 ghostCol = palette(ghostN) * 0.08;
  col += ghostCol;

  // --- Vignette ---
  vec2 vig = curvedUV - 0.5;
  col *= 1.0 - dot(vig, vig) * 1.2;

  // --- CRT edge shadow + off-screen black ---
  col *= offScreen;

  fragColor = vec4(col * 0.45, 1.0);
}
