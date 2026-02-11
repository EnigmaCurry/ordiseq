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

// Pseudo-random from 2D seed
float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// Smooth stroke: 1.0 when d < w, fading to 0.0
float strokeLine(float d, float w) {
  return 1.0 - smoothstep(w - 0.02, w + 0.03, d);
}

// Character-cell glyph: returns brightness for a procedural symbol
float glyph(vec2 cellUV, float seed) {
  vec2 p = cellUV - 0.5; // centered: -0.5 to 0.5
  float w = 0.065;       // stroke half-width

  // Pick one of 32 glyph types from seed
  int t = int(floor(fract(seed * 7.31) * 32.0));
  float s = 0.0;

  if (t == 0) {
    // │ vertical bar
    s = strokeLine(abs(p.x), w);
  } else if (t == 1) {
    // ─ horizontal bar
    s = strokeLine(abs(p.y), w);
  } else if (t == 2) {
    // ┼ plus
    s = strokeLine(min(abs(p.x), abs(p.y)), w);
  } else if (t == 3) {
    // ╳ X cross
    s = strokeLine(min(abs(p.x - p.y), abs(p.x + p.y)) * 0.7071, w);
  } else if (t == 4) {
    // ○ circle
    s = strokeLine(abs(length(p) - 0.28), w);
  } else if (t == 5) {
    // ● filled dot
    s = 1.0 - smoothstep(0.15, 0.19, length(p));
  } else if (t == 6) {
    // □ square outline
    float bx = max(abs(p.x), abs(p.y));
    s = strokeLine(abs(bx - 0.28), w);
  } else if (t == 7) {
    // ≡ three horizontal lines
    float d = min(min(abs(p.y - 0.22), abs(p.y)), abs(p.y + 0.22));
    s = strokeLine(d, w * 0.55);
  } else if (t == 8) {
    // ‖ double vertical bars
    s = strokeLine(min(abs(p.x - 0.14), abs(p.x + 0.14)), w);
  } else if (t == 9) {
    // ◇ diamond outline
    float dd = abs(p.x) + abs(p.y);
    s = strokeLine(abs(dd - 0.32), w);
  } else if (t == 10) {
    // ※ asterisk (plus + X combined)
    float d1 = min(abs(p.x), abs(p.y));
    float d2 = min(abs(p.x - p.y), abs(p.x + p.y)) * 0.7071;
    s = strokeLine(min(d1, d2), w * 0.75);
  } else if (t == 11) {
    // = equals (two horizontal bars)
    float d = min(abs(p.y - 0.12), abs(p.y + 0.12));
    s = strokeLine(d, w);
  } else if (t == 12) {
    // ⌒ arc (top half of circle)
    float cd = abs(length(p) - 0.3);
    s = strokeLine(cd, w) * smoothstep(-0.05, 0.05, p.y);
  } else if (t == 13) {
    // └ L-shape (bottom-left corner)
    float vert = strokeLine(abs(p.x + 0.15), w) * step(0.0, p.y);
    float horiz = strokeLine(abs(p.y), w) * step(-0.15, p.x);
    s = max(vert, horiz);
  } else if (t == 14) {
    // ┐ reverse-L (top-right corner)
    float vert = strokeLine(abs(p.x - 0.15), w) * step(p.y, 0.0);
    float horiz = strokeLine(abs(p.y), w) * step(p.x, 0.15);
    s = max(vert, horiz);
  } else if (t == 15) {
    // ☺ smiley face
    float head = strokeLine(abs(length(p) - 0.35), w * 0.7);
    float eyeL = 1.0 - smoothstep(0.03, 0.06, length(p - vec2(-0.12, 0.1)));
    float eyeR = 1.0 - smoothstep(0.03, 0.06, length(p - vec2(0.12, 0.1)));
    float smile = strokeLine(abs(length(p - vec2(0.0, 0.04)) - 0.16), w * 0.5)
                * step(p.y, 0.04);
    s = max(head, max(max(eyeL, eyeR), smile));
  } else if (t == 16) {
    // 🐶 dog face
    float head = strokeLine(abs(length(p * vec2(1.0, 1.1)) - 0.24), w * 0.7);
    float earL = 1.0 - smoothstep(0.06, 0.09, length(p - vec2(-0.2, 0.25)));
    float earR = 1.0 - smoothstep(0.06, 0.09, length(p - vec2(0.2, 0.25)));
    float eyeL = 1.0 - smoothstep(0.02, 0.04, length(p - vec2(-0.09, 0.06)));
    float eyeR = 1.0 - smoothstep(0.02, 0.04, length(p - vec2(0.09, 0.06)));
    float nose = 1.0 - smoothstep(0.03, 0.05, length(p - vec2(0.0, -0.06)));
    float tongue = strokeLine(abs(p.x), w * 0.4) * step(-0.22, p.y) * step(p.y, -0.13);
    s = max(head, max(max(earL, earR), max(max(eyeL, eyeR), max(nose, tongue))));
  } else if (t == 17) {
    // ♥ heart (filled)
    vec2 hp = (p + vec2(0.0, 0.35)) * 1.5;
    hp.x = abs(hp.x);
    float hd;
    if (hp.y + hp.x > 1.0) {
      hd = length(hp - vec2(0.25, 0.75)) - 0.3536;
    } else {
      float a = length(hp - vec2(0.0, 1.0));
      float b = length(hp - vec2(0.5 * max(hp.x + hp.y, 0.0)));
      hd = min(a, b) * sign(hp.x - hp.y);
    }
    s = 1.0 - smoothstep(-0.02, 0.08, hd);
  } else if (t == 18) {
    // △ triangle up outline
    vec2 tp = vec2(abs(p.x), p.y + 0.05);
    float tri = max(-tp.y - 0.2, tp.x * 0.866 + tp.y * 0.5 - 0.22);
    s = strokeLine(abs(tri), w);
  } else if (t == 19) {
    // ∇ triangle down outline
    vec2 tp = vec2(abs(p.x), -p.y + 0.05);
    float tri = max(-tp.y - 0.2, tp.x * 0.866 + tp.y * 0.5 - 0.22);
    s = strokeLine(abs(tri), w);
  } else if (t == 20) {
    // ↑ arrow up
    float stem = strokeLine(abs(p.x), w) * step(p.y, 0.15);
    float chevron = strokeLine(abs(abs(p.x) + p.y - 0.35) * 0.707, w)
                  * step(0.12, p.y) * step(p.y, 0.4);
    s = max(stem, chevron);
  } else if (t == 21) {
    // → arrow right
    float stem = strokeLine(abs(p.y), w) * step(p.x, 0.15);
    float chevron = strokeLine(abs(abs(p.y) + p.x - 0.35) * 0.707, w)
                  * step(0.12, p.x) * step(p.x, 0.4);
    s = max(stem, chevron);
  } else if (t == 22) {
    // ♪ music note
    float nh = 1.0 - smoothstep(0.08, 0.12, length((p - vec2(-0.05, -0.2)) * vec2(1.0, 1.5)));
    float stem = strokeLine(abs(p.x - 0.07), w * 0.6) * step(-0.2, p.y) * step(p.y, 0.3);
    float flag = strokeLine(abs(length(p - vec2(0.18, 0.2)) - 0.14), w * 0.5)
               * step(0.07, p.x) * step(0.12, p.y);
    s = max(nh, max(stem, flag));
  } else if (t == 23) {
    // ☽ crescent moon (filled)
    float outer = length(p) - 0.3;
    float inner = length(p - vec2(0.13, 0.0)) - 0.26;
    s = 1.0 - smoothstep(-0.02, 0.04, max(outer, -inner));
  } else if (t == 24) {
    // ⊕ circle with plus
    float circ = strokeLine(abs(length(p) - 0.28), w * 0.7);
    float pl = strokeLine(min(abs(p.x), abs(p.y)), w * 0.5);
    s = max(circ, pl);
  } else if (t == 25) {
    // ⊞ square with cross
    float bx = max(abs(p.x), abs(p.y));
    float sq = strokeLine(abs(bx - 0.3), w * 0.7);
    float cr = strokeLine(min(abs(p.x), abs(p.y)), w * 0.5) * step(bx, 0.33);
    s = max(sq, cr);
  } else if (t == 26) {
    // T shape
    float vert = strokeLine(abs(p.x), w) * step(p.y, 0.25);
    float horiz = strokeLine(abs(p.y - 0.25), w);
    s = max(vert, horiz);
  } else if (t == 27) {
    // Z shape
    float topBar = strokeLine(abs(p.y - 0.25), w);
    float botBar = strokeLine(abs(p.y + 0.25), w);
    float diag = strokeLine(abs(p.x + p.y) * 0.707, w) * step(abs(p.y), 0.28);
    s = max(max(topBar, botBar), diag);
  } else if (t == 28) {
    // ∿ sine wave
    float wave = abs(p.y - 0.15 * sin(p.x * 10.0));
    s = strokeLine(wave, w);
  } else if (t == 29) {
    // ◠ bottom arc
    float cd = abs(length(p) - 0.3);
    s = strokeLine(cd, w) * smoothstep(0.05, -0.05, p.y);
  } else if (t == 30) {
    // Random 4×6 grid (denser variant)
    vec2 g = floor(cellUV * vec2(4.0, 6.0));
    float on = step(0.4, hash(g + seed * 23.7));
    vec2 f = fract(cellUV * vec2(4.0, 6.0));
    float mask = smoothstep(0.0, 0.18, f.x) * smoothstep(1.0, 0.82, f.x)
               * smoothstep(0.0, 0.18, f.y) * smoothstep(1.0, 0.82, f.y);
    s = on * mask;
  } else {
    // Random 3×5 grid (original katakana-like)
    vec2 g = floor(cellUV * vec2(3.0, 5.0));
    float on = step(0.45, hash(g + seed * 17.3));
    vec2 f = fract(cellUV * vec2(3.0, 5.0));
    float mask = smoothstep(0.0, 0.15, f.x) * smoothstep(1.0, 0.85, f.x)
               * smoothstep(0.0, 0.15, f.y) * smoothstep(1.0, 0.85, f.y);
    s = on * mask;
  }

  // Cell margin to prevent glyphs from touching edges
  float margin = smoothstep(0.0, 0.08, cellUV.x) * smoothstep(1.0, 0.92, cellUV.x)
               * smoothstep(0.0, 0.08, cellUV.y) * smoothstep(1.0, 0.92, cellUV.y);

  return s * margin;
}

void main() {
  float tempo = u_bpm / 120.0;
  float syncTime = mix(u_time, u_beat / tempo, u_playing);
  float beatPulse = exp(-fract(syncTime * tempo) * 4.0);

  // Camera path — smooth cinematic drift through the infinite field
  float camX = syncTime * 15.0 + sin(syncTime * 0.2) * 60.0;
  float camY = sin(syncTime * 0.15) * 30.0 + cos(syncTime * 0.1) * 20.0;
  float angle = sin(syncTime * 0.09) * 0.04; // subtle rotation

  vec3 totalColor = vec3(0.0);

  const int NUM_LAYERS = 7;
  for (int i = 0; i < NUM_LAYERS; i++) {
    float fi = float(i);
    float depth = 0.4 + fi * 0.45;
    float invDepth = 1.0 / depth;

    // Depth fog — closer layers brighter
    float fog = 1.0 / (1.0 + fi * 0.5);

    // Column density varies by depth (closer = bigger glyphs)
    float cols = 14.0 + fi * 4.0;
    float cellW = u_resolution.x / cols;
    float cellH = cellW * 1.6;

    // Parallax offset — deeper layers drift slower
    vec2 offset = vec2(camX, camY) * invDepth;

    // Per-layer rotation around screen center
    float a = angle * invDepth;
    vec2 centered = gl_FragCoord.xy - u_resolution * 0.5;
    vec2 rotated = vec2(
      centered.x * cos(a) - centered.y * sin(a),
      centered.x * sin(a) + centered.y * cos(a)
    );
    vec2 fragPos = rotated + u_resolution * 0.5 + offset;

    // Cell coordinates (work with any sign thanks to floor/fract)
    vec2 cell = vec2(
      floor(fragPos.x / cellW),
      floor(fragPos.y / cellH)
    );
    vec2 cellUV = vec2(
      fract(fragPos.x / cellW),
      fract(fragPos.y / cellH)
    );

    // Per-column stream properties
    float colSeed = hash(vec2(cell.x, fi * 73.13));
    float speed = (1.5 + colSeed * 2.5) * tempo;
    float phase = colSeed * 100.0;
    float streamLen = 4.0 + colSeed * 8.0;
    float gapLen = 14.0 + colSeed * 20.0;
    float period = streamLen + gapLen;

    // Periodic stream cycle — tiles seamlessly over infinite scrolling
    float cyclePos = mod(cell.y + syncTime * speed + phase, period);

    if (cyclePos >= streamLen) {
      continue;
    }

    float dist = cyclePos;

    // Glyph character (flickers over time)
    float charSeed = hash(cell + vec2(fi * 37.0, floor(syncTime * (2.0 + colSeed * 3.0) * tempo)));
    float g = glyph(cellUV, charSeed);

    // Brightness: head is white-hot, fades along tail
    float brightness;
    if (dist < 1.0) {
      brightness = 1.0;
    } else {
      brightness = max(0.05, 1.0 - (dist - 1.0) / (streamLen - 1.0));
      brightness *= brightness;
    }

    // Color from user palette, offset per layer
    int colorIdx = int(mod(cell.x + fi, 4.0));
    vec3 baseColor;
    if (colorIdx == 0) baseColor = u_color1;
    else if (colorIdx == 1) baseColor = u_color2;
    else if (colorIdx == 2) baseColor = u_color3;
    else baseColor = u_color4;

    vec3 neon = baseColor * 1.5;
    vec3 col;
    if (dist < 1.0) {
      col = mix(neon, vec3(1.0), 0.7);
    } else {
      col = neon * brightness;
    }

    // Additive blend with depth fade
    totalColor += col * g * fog;
  }

  // Beat-synced brightness pulse + filmic tone mapping
  totalColor *= 1.0 + beatPulse * 0.15;
  totalColor = 1.0 - exp(-totalColor * 1.5);

  fragColor = vec4(totalColor, 1.0);
}
