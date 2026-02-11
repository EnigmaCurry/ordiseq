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
#define SCOPE_SAMPLES 200

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

vec3 palette(float idx) {
  idx = fract(idx) * 4.0;
  if (idx < 1.0) return mix(u_color1, u_color2, idx);
  else if (idx < 2.0) return mix(u_color2, u_color3, idx - 1.0);
  else if (idx < 3.0) return mix(u_color3, u_color4, idx - 2.0);
  else return mix(u_color4, u_color1, idx - 3.0);
}

// --- Oscilloscope shape parametric curves ---
// All shapes are time-dependent and continuously animated

// Mushroom: breathing cap, swaying stem, orbiting spots
vec2 mushroom(float s, float t) {
  float breath = 1.0 + 0.08 * sin(t * 1.5);
  float sway = sin(t * 0.8) * 0.03;
  if (s < 0.4) {
    float a = PI - s / 0.4 * PI;
    // Cap pulses wider/narrower, ripples travel along it
    float ripple = 0.015 * sin(a * 6.0 + t * 4.0);
    return vec2(cos(a) * 0.3 * breath + sway, sin(a) * 0.12 + 0.05 + ripple);
  } else if (s < 0.44) {
    float f = (s - 0.4) / 0.04;
    return mix(vec2(0.3 * breath + sway, 0.05), vec2(0.22 * breath + sway, -0.02), f);
  } else if (s < 0.52) {
    float f = (s - 0.44) / 0.08;
    return mix(vec2(0.22 * breath + sway, -0.02), vec2(0.08 + sway, -0.06), f);
  } else if (s < 0.62) {
    float f = (s - 0.52) / 0.1;
    // Stem sways with a wave
    float stemSway = sway + sin(f * PI + t * 1.2) * 0.015;
    return mix(vec2(0.08 + stemSway, -0.06), vec2(0.1 + stemSway, -0.32), f);
  } else if (s < 0.7) {
    float f = (s - 0.62) / 0.08;
    return mix(vec2(0.1 + sway, -0.32), vec2(-0.1 + sway, -0.32), f);
  } else if (s < 0.8) {
    float f = (s - 0.7) / 0.1;
    float stemSway = sway + sin((1.0 - f) * PI + t * 1.2) * 0.015;
    return mix(vec2(-0.1 + stemSway, -0.32), vec2(-0.08 + stemSway, -0.06), f);
  } else if (s < 0.88) {
    float f = (s - 0.8) / 0.08;
    return mix(vec2(-0.08 + sway, -0.06), vec2(-0.22 * breath + sway, -0.02), f);
  } else {
    float f = (s - 0.88) / 0.12;
    return mix(vec2(-0.22 * breath + sway, -0.02), vec2(-0.3 * breath + sway, 0.05), f);
  }
}

// Star: rotating, pulsing points, morphing point count
vec2 star(float s, float t) {
  float a = s * PI * 2.0 + t * 0.4; // continuous rotation
  // Point count drifts between 4 and 7
  float points = 5.0 + sin(t * 0.13) * 1.5;
  // Inner/outer radius breathes
  float outerPulse = 0.17 + 0.04 * sin(t * 1.8);
  float innerPulse = 0.15 + 0.03 * sin(t * 2.3 + 1.0);
  float r = innerPulse + outerPulse * pow(abs(cos(a * points * 0.5)), 0.7);
  // Dots orbiting the tips
  float tipWobble = sin(a * points + t * 3.0) * 0.012;
  return vec2(cos(a), sin(a)) * (r + tipWobble);
}

// Heart: beating pulse, rotating, lobes undulate
vec2 heart(float s, float t) {
  float a = s * PI * 2.0 + t * 0.15; // slow rotation
  // Heartbeat: quick squeeze then relax
  float beat = 1.0 + 0.12 * pow(abs(sin(t * 2.0)), 8.0);
  float x = 0.28 * sin(a) * sin(a) * sin(a);
  float y = 0.22 * cos(a) - 0.08 * cos(2.0 * a) - 0.04 * cos(3.0 * a) - 0.015 * cos(4.0 * a);
  vec2 p = vec2(x, y * 0.9 + 0.02) * beat;
  // Lobes ripple
  p += vec2(sin(a * 3.0 + t * 2.5), cos(a * 3.0 + t * 2.0)) * 0.01;
  return p;
}

// Butterfly: flapping wings, body sways, wing veins animate
vec2 butterfly(float s, float t) {
  float a = s * PI * 2.0;
  float r = exp(sin(a)) - 2.0 * cos(4.0 * a) + pow(sin((2.0 * a - PI) / 24.0), 5.0);
  vec2 p = vec2(sin(a), cos(a)) * r * 0.08;
  // Flap wings
  float flap = 0.5 + 0.5 * sin(t * 2.5);
  p.x *= 0.4 + flap * 0.6;
  // Vertical bob
  p.y += flap * 0.02;
  // Wing vein detail that crawls along the shape
  p += vec2(sin(a * 8.0 + t * 3.0), cos(a * 6.0 - t * 2.0)) * 0.008;
  // Whole body sways in a figure-8
  p.x += sin(t * 0.6) * 0.03;
  p.y += sin(t * 1.2) * 0.015;
  return p;
}

// Spirograph / Lissajous: evolving frequencies
vec2 spirograph(float s, float t) {
  float a = s * PI * 2.0 * 3.0;
  float fx = 3.0 + sin(t * 0.05);
  float fy = 2.0 + cos(t * 0.07);
  return vec2(sin(a * fx + t * 0.2), cos(a * fy)) * 0.25;
}

// Atom: electrons orbiting a nucleus in 3 planes
vec2 atom(float s, float t) {
  float orbit = s * PI * 2.0;
  // 3 electron orbits tilted at different angles, particle dots race around
  float plane = floor(s * 3.0);
  float localS = fract(s * 3.0);
  float localA = localS * PI * 2.0;
  float tilt = plane * PI / 3.0 + t * 0.1;
  // Orbit ellipse
  float rx = 0.25;
  float ry = 0.08;
  vec2 p = vec2(cos(localA + t * (1.5 + plane * 0.4)) * rx,
                sin(localA + t * (1.5 + plane * 0.4)) * ry);
  // Rotate by tilt
  float ct = cos(tilt), st = sin(tilt);
  p = vec2(p.x * ct - p.y * st, p.x * st + p.y * ct);
  // Nucleus wobble in center (small cluster for first few samples)
  if (s < 0.04) {
    float na = s / 0.04 * PI * 2.0;
    p = vec2(cos(na + t * 5.0), sin(na + t * 5.0)) * 0.02;
  }
  return p;
}

// Clock: face with tick marks, rotating hour and minute hands
vec2 clockShape(float s, float t) {
  if (s < 0.5) {
    // Clock face circle with tick bumps at hour positions
    float a = s / 0.5 * PI * 2.0;
    float baseR = 0.25;
    // 12 tick marks as small radius bumps
    float tickAngle = mod(a, PI / 6.0) - PI / 12.0;
    float tick = smoothstep(0.06, 0.0, abs(tickAngle)) * 0.02;
    float r = baseR + tick;
    // Subtle breathing
    r *= 1.0 + 0.015 * sin(t * 1.0);
    return vec2(cos(a), sin(a)) * r;
  } else if (s < 0.75) {
    // Minute hand: full rotation every ~30 seconds of shader time
    float f = (s - 0.5) / 0.25;
    float minuteAngle = PI * 0.5 - t * 0.2;
    vec2 tip = vec2(cos(minuteAngle), sin(minuteAngle)) * 0.2;
    // Slight wobble at the tip
    tip += vec2(sin(t * 3.0), cos(t * 2.7)) * 0.003;
    return mix(vec2(0.0), tip, f);
  } else {
    // Hour hand: slower rotation, shorter
    float f = (s - 0.75) / 0.25;
    float hourAngle = PI * 0.5 - t * 0.017;
    vec2 tip = vec2(cos(hourAngle), sin(hourAngle)) * 0.14;
    tip += vec2(sin(t * 2.5 + 1.0), cos(t * 2.0 + 1.0)) * 0.002;
    return mix(vec2(0.0), tip, f);
  }
}

// Smiley face: blinking eyes, morphing grin
vec2 smiley(float s, float t) {
  // Blink: eyes squash to a line periodically
  float blinkCycle = mod(t, 4.0); // blink every ~4 seconds
  float blink = 1.0 - 0.9 * pow(max(1.0 - abs(blinkCycle - 0.15) * 10.0, 0.0), 2.0);
  // Grin width and curve morph
  float grinWidth = 0.14 + 0.04 * sin(t * 0.7);
  float grinCurve = 0.08 + 0.06 * sin(t * 0.5);

  if (s < 0.4) {
    // Head outline, gentle wobble
    float a = s / 0.4 * PI * 2.0;
    float r = 0.25 + 0.008 * sin(a * 5.0 + t * 2.0);
    return vec2(cos(a), sin(a)) * r;
  } else if (s < 0.52) {
    // Left eye (ellipse that squashes on blink)
    float a = (s - 0.4) / 0.12 * PI * 2.0;
    float ex = 0.055;
    float ey = 0.06 * blink;
    vec2 p = vec2(-0.09 + cos(a) * ex, 0.06 + sin(a) * ey);
    // Pupil jitter
    p += vec2(sin(t * 1.5), cos(t * 1.2)) * 0.005;
    return p;
  } else if (s < 0.64) {
    // Right eye
    float a = (s - 0.52) / 0.12 * PI * 2.0;
    float ex = 0.055;
    float ey = 0.06 * blink;
    vec2 p = vec2(0.09 + cos(a) * ex, 0.06 + sin(a) * ey);
    p += vec2(sin(t * 1.5), cos(t * 1.2)) * 0.005;
    return p;
  } else {
    // Mouth: curved grin arc
    float f = (s - 0.64) / 0.36;
    float a = PI + f * PI; // bottom half arc
    float mx = cos(a) * grinWidth;
    float my = sin(a) * grinCurve - 0.08;
    // Animate corners up/down for expression changes
    float corner = sin(t * 0.3) * 0.02 * (1.0 - abs(f - 0.5) * 2.0);
    return vec2(mx, my + corner);
  }
}

// Mushroom man: stick figure holding a big mushroom
vec2 mushroomMan(float s, float t) {
  float walk = sin(t * 2.0); // leg/arm swing
  float bob = abs(sin(t * 2.0)) * 0.01; // body bounces with steps

  if (s < 0.08) {
    // Head circle
    float a = s / 0.08 * PI * 2.0;
    return vec2(-0.12 + cos(a) * 0.04, 0.02 + sin(a) * 0.04 + bob);
  } else if (s < 0.16) {
    // Body (torso line)
    float f = (s - 0.08) / 0.08;
    return mix(vec2(-0.12, -0.02 + bob), vec2(-0.12, -0.18 + bob), f);
  } else if (s < 0.22) {
    // Left leg (swings with walk)
    float f = (s - 0.16) / 0.06;
    float swing = walk * 0.04;
    return mix(vec2(-0.12, -0.18 + bob), vec2(-0.16 - swing, -0.32 + bob), f);
  } else if (s < 0.24) {
    // Back to hip
    float f = (s - 0.22) / 0.02;
    float swing = walk * 0.04;
    return mix(vec2(-0.16 - swing, -0.32 + bob), vec2(-0.12, -0.18 + bob), f);
  } else if (s < 0.30) {
    // Right leg
    float f = (s - 0.24) / 0.06;
    float swing = -walk * 0.04;
    return mix(vec2(-0.12, -0.18 + bob), vec2(-0.08 - swing, -0.32 + bob), f);
  } else if (s < 0.32) {
    // Back to shoulder
    float f = (s - 0.30) / 0.02;
    float swing = -walk * 0.04;
    return mix(vec2(-0.08 - swing, -0.32 + bob), vec2(-0.12, -0.06 + bob), f);
  } else if (s < 0.38) {
    // Left arm (swings opposite legs)
    float f = (s - 0.32) / 0.06;
    float swing = -walk * 0.03;
    return mix(vec2(-0.12, -0.06 + bob), vec2(-0.2 + swing, -0.14 + bob), f);
  } else if (s < 0.40) {
    // Back to shoulder
    float f = (s - 0.38) / 0.02;
    float swing = -walk * 0.03;
    return mix(vec2(-0.2 + swing, -0.14 + bob), vec2(-0.12, -0.06 + bob), f);
  } else if (s < 0.46) {
    // Right arm reaching up to hold mushroom
    float f = (s - 0.40) / 0.06;
    float reach = sin(t * 1.5) * 0.01;
    return mix(vec2(-0.12, -0.06 + bob), vec2(-0.02 + reach, 0.06 + bob), f);
  } else if (s < 0.48) {
    // Transition to mushroom cap start
    float f = (s - 0.46) / 0.02;
    return mix(vec2(-0.02, 0.06 + bob), vec2(0.18, 0.12 + bob), f);
  } else if (s < 0.72) {
    // Mushroom cap dome
    float a = PI - (s - 0.48) / 0.24 * PI;
    float breath = 1.0 + 0.05 * sin(t * 1.8);
    float ripple = 0.01 * sin(a * 5.0 + t * 3.0);
    return vec2(cos(a) * 0.2 * breath + 0.1, sin(a) * 0.1 + 0.12 + bob + ripple);
  } else if (s < 0.78) {
    // Cap right edge to underside
    float f = (s - 0.72) / 0.06;
    return mix(vec2(0.3, 0.12 + bob), vec2(0.22, 0.06 + bob), f);
  } else if (s < 0.84) {
    // Mushroom stem right
    float f = (s - 0.78) / 0.06;
    return mix(vec2(0.22, 0.06 + bob), vec2(0.15, -0.12 + bob), f);
  } else if (s < 0.90) {
    // Stem bottom
    float f = (s - 0.84) / 0.06;
    return mix(vec2(0.15, -0.12 + bob), vec2(0.05, -0.12 + bob), f);
  } else if (s < 0.96) {
    // Stem left
    float f = (s - 0.90) / 0.06;
    return mix(vec2(0.05, -0.12 + bob), vec2(-0.02, 0.06 + bob), f);
  } else {
    // Cap left edge back up
    float f = (s - 0.96) / 0.04;
    return mix(vec2(-0.02, 0.06 + bob), vec2(-0.1, 0.12 + bob), f);
  }
}

// Get shape by index
vec2 getShape(int idx, float s, float t) {
  if (idx == 0) return mushroom(s, t);
  else if (idx == 1) return star(s, t);
  else if (idx == 2) return heart(s, t);
  else if (idx == 3) return butterfly(s, t);
  else if (idx == 4) return spirograph(s, t);
  else if (idx == 5) return atom(s, t);
  else if (idx == 6) return clockShape(s, t);
  else if (idx == 7) return smiley(s, t);
  else return mushroomMan(s, t);
}

// Morph continuously between shapes, slowing near each target
vec2 morphShape(float s, float t) {
  // Continuous phase that accelerates between shapes and decelerates near them
  float rawPhase = t * 0.04;
  float f = fract(rawPhase);
  // Ease: fast in middle, slow near 0 and 1 (smooth-stop/start)
  float eased = f * f * (3.0 - 2.0 * f); // smoothstep-style
  eased = eased * eased * (3.0 - 2.0 * eased); // double-smooth for more hang time at endpoints

  float base = floor(rawPhase);
  int shapeIdx = int(mod(base, 9.0));
  int nextIdx = int(mod(base + 1.0, 9.0));

  return mix(getShape(shapeIdx, s, t), getShape(nextIdx, s, t), eased);
}

// Compute oscilloscope glow for a pixel
float scopeGlow(vec2 p, float t) {
  float aspect = u_resolution.x / u_resolution.y;
  vec2 centered = (p - 0.5) * vec2(aspect, 1.0);

  float minDist = 1.0;
  for (int i = 0; i < SCOPE_SAMPLES; i++) {
    float s = float(i) / float(SCOPE_SAMPLES);
    vec2 shapeP = morphShape(s, t);
    // Gentle wobble like real analog scope
    shapeP += vec2(sin(s * 40.0 + t * 3.0), cos(s * 35.0 + t * 2.5)) * 0.003;
    float d = length(centered - shapeP);
    minDist = min(minDist, d);
  }
  // Tight phosphor glow: bright core + softer halo
  float core = 0.0015 / (minDist * minDist + 0.0015);
  float halo = 0.008 / (minDist + 0.02);
  return min(core + halo, 5.0);
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
  float offScreen = step(0.0, curvedUV.x) * step(curvedUV.x, 1.0)
                  * step(0.0, curvedUV.y) * step(curvedUV.y, 1.0);

  // --- Multi-layer glitch triggers ---
  float glitch1 = step(0.90, hash(floor(t * 2.0)));
  float glitch2 = step(0.95, hash(floor(t * 3.0 + 7.0)));
  float glitch3 = step(0.97, hash(floor(t * 1.5 + 13.0)));

  // --- Vertical hold drift ---
  float holdPhase = t * 0.04;
  float holdDrift = fract(sin(holdPhase * 0.7) * 0.5 + holdPhase * 0.02);
  float vHold = holdDrift * 0.06 * sin(t * 0.3);
  vHold += glitch3 * (hash(floor(t * 2.0 + 5.0)) - 0.5) * 0.15;

  // --- Tracking wobble ---
  float trackingPhase = t * 0.7;
  float trackBand1 = 1.0 - smoothstep(0.0, 0.05, abs(curvedUV.y - fract(trackingPhase)));
  float trackBand2 = 1.0 - smoothstep(0.0, 0.04, abs(curvedUV.y - fract(trackingPhase * 1.3 + 0.33)));

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
  distUV.y = fract(distUV.y + vHold);
  distUV.x += wobble * sin(curvedUV.y * 100.0 + t * 10.0) + tearOffset;

  // --- Wandering drift direction ---
  float driftAngle = sin(t * 0.011) * 2.5
                   + sin(t * 0.0047 + 1.0) * 1.8
                   + sin(t * 0.0023 + 4.0) * 1.0;
  float driftSpeed = 0.35 + 0.15 * sin(t * 0.007 + 2.0);
  vec2 drift = vec2(cos(driftAngle), sin(driftAngle)) * driftSpeed * t;

  // --- Composite video color bleed ---
  float bleedAmt = 0.008 + glitch1 * 0.015;
  vec2 bleedUV1 = distUV + vec2(bleedAmt, 0.0);
  vec2 bleedUV2 = distUV - vec2(bleedAmt, 0.0);

  // --- Chromatic aberration ---
  float aberr = 0.003 + glitch1 * 0.008 + glitch2 * 0.015 + glitch3 * 0.03;

  float lumC = fbm(distUV * 4.0 + drift * 1.2);
  float lumL = fbm(bleedUV2 * 4.0 + drift * 1.2);
  float lumR = fbm(bleedUV1 * 4.0 + drift * 1.2);

  // --- Colorize through user palette ---
  float n = fbm(distUV * 3.0 + drift);
  vec3 baseCol = palette(n);

  float aberrLum = (lumL + lumC + lumR) / 3.0;
  vec3 bleedColL = palette(n - aberr * 8.0);
  vec3 bleedColR = palette(n + aberr * 8.0);
  vec3 col = mix(baseCol, bleedColL, 0.15) * (0.4 + lumL * 0.4)
           + mix(baseCol, baseCol, 0.7) * (0.4 + lumC * 0.4)
           + mix(baseCol, bleedColR, 0.15) * (0.4 + lumR * 0.4);
  col /= 1.8;

  // --- Color posterization ---
  float posterLevels = 12.0 - glitch2 * 6.0;
  col = floor(col * posterLevels + 0.5) / posterLevels;

  // --- Palette color shift during glitch ---
  if (glitch2 > 0.0) {
    float dropChoice = hash(floor(t * 5.0 + 2.0));
    vec3 shiftColor = palette(dropChoice);
    col = mix(col, shiftColor * (0.5 + aberrLum * 0.5), 0.6);
  }

  // --- Soft scanlines ---
  float scanline = 0.90 + 0.10 * sin(curvedUV.y * u_resolution.y * 0.5);
  col *= scanline;

  // --- RF interference ---
  float rf = sin((curvedUV.x + curvedUV.y) * 20.0 + t * 5.0) * 0.5 + 0.5;
  rf *= sin((curvedUV.x - curvedUV.y * 0.5) * 30.0 - t * 3.0) * 0.5 + 0.5;
  col += palette(curvedUV.y + t * 0.1) * rf * 0.03;

  // --- Static bursts during glitches ---
  if (glitch1 > 0.0) {
    float burstY = hash(floor(t * 3.0 + 11.0));
    float burstBand = smoothstep(burstY - 0.01, burstY, curvedUV.y)
                    * smoothstep(burstY + 0.05, burstY + 0.04, curvedUV.y);
    float burstNoise = hash2(vec2(gl_FragCoord.x * 0.3, floor(t * 25.0)));
    vec3 burstTint = palette(hash(floor(t * 3.0 + 17.0)));
    col = mix(col, burstTint * burstNoise, burstBand * 0.8);
  }
  if (glitch2 > 0.0) {
    float burst2Y = hash(floor(t * 2.0 + 19.0));
    float burst2Band = smoothstep(burst2Y - 0.01, burst2Y, curvedUV.y)
                     * smoothstep(burst2Y + 0.07, burst2Y + 0.06, curvedUV.y);
    float burst2Noise = hash2(vec2(gl_FragCoord.x * 0.2, floor(t * 20.0) + 5.0));
    vec3 burst2Tint = palette(hash(floor(t * 2.0 + 23.0)));
    col = mix(col, burst2Tint * burst2Noise, burst2Band * 0.7);
  }

  // --- Snow ---
  float snow = hash2(gl_FragCoord.xy * 0.1 + floor(t * 60.0) * 7.0);
  float snowAmt = 0.03 + glitch3 * 0.25;
  col = mix(col, palette(snow) * snow, snowAmt);

  // --- Horizontal noise bars ---
  float staticBar = noise(vec2(0.0, curvedUV.y * 80.0 + t * 20.0));
  float staticBar2 = noise(vec2(3.0, curvedUV.y * 120.0 - t * 15.0));
  col += baseCol * (staticBar * 0.03 + staticBar2 * 0.02);

  // --- VHS head switch ---
  float headSwitch = smoothstep(0.0, 0.02, curvedUV.y) * smoothstep(0.06, 0.03, curvedUV.y);
  float headNoise = hash(floor(t * 30.0) + curvedUV.x * 10.0);
  col = mix(col, baseCol * headNoise, headSwitch * 0.8);

  // --- Rolling brightness bar ---
  float roll = sin(curvedUV.y * 6.28 - t * 1.5) * 0.5 + 0.5;
  col *= 0.85 + 0.15 * roll;

  // --- CRT bloom ---
  float lum = dot(col, vec3(0.299, 0.587, 0.114));
  float bloom = smoothstep(0.35, 0.7, lum);
  col += baseCol * bloom * 0.12;

  // --- Frame jitter ---
  if (glitch3 > 0.0) {
    float jitter = (hash(floor(t * 10.0)) - 0.5) * 0.03;
    col *= 0.85 + 0.3 * noise(curvedUV * 50.0 + jitter);
  }

  // --- Ghosting ---
  float ghostN = fbm((distUV - vec2(0.02, 0.005)) * 3.0 + t * 0.3);
  vec3 ghostCol = palette(ghostN) * 0.08;
  col += ghostCol;

  // --- Oscilloscope art overlay ---
  float glow = scopeGlow(curvedUV, t);
  // Scope beam color: bright palette color with phosphor green tint
  float scopeColorIdx = t * 0.02;
  vec3 scopeColor = palette(scopeColorIdx);
  // Brighten toward white at high intensity for phosphor bloom
  vec3 scopeBeam = mix(scopeColor * 2.0, vec3(1.0), smoothstep(1.0, 4.0, glow));
  col += scopeBeam * glow * 0.7;

  // --- Vignette ---
  vec2 vig = curvedUV - 0.5;
  col *= 1.0 - dot(vig, vig) * 1.2;

  // --- Off-screen black ---
  col *= offScreen;

  fragColor = vec4(col * 0.45, 1.0);
}
