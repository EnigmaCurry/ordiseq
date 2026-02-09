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

// Pseudo-random from 2D seed
float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

// Character-cell glyph: returns brightness for a fake symbol
float glyph(vec2 cellUV, float seed) {
  // Grid of dots within each cell to simulate a character
  vec2 g = floor(cellUV * vec2(3.0, 5.0));
  float on = step(0.45, hash(g + seed * 17.3));
  // Soften edges
  vec2 f = fract(cellUV * vec2(3.0, 5.0));
  float mask = smoothstep(0.0, 0.15, f.x) * smoothstep(1.0, 0.85, f.x)
             * smoothstep(0.0, 0.15, f.y) * smoothstep(1.0, 0.85, f.y);
  return on * mask;
}

void main() {
  float cols = 40.0;
  float cellW = u_resolution.x / cols;
  float cellH = cellW * 1.6;
  float rows = u_resolution.y / cellH;

  vec2 cell = vec2(
    floor(gl_FragCoord.x / cellW),
    floor(gl_FragCoord.y / cellH)
  );
  vec2 cellUV = vec2(
    fract(gl_FragCoord.x / cellW),
    fract(gl_FragCoord.y / cellH)
  );

  // Each column has its own speed and phase, scaled by BPM
  float colSeed = hash(vec2(cell.x, 0.0));
  float tempo = u_bpm / 120.0;
  float speed = (3.0 + colSeed * 5.0) * tempo;
  float phase = colSeed * 100.0;

  // Stream head position (in row units, moving downward)
  // Flip y: row 0 is bottom in GL, so invert
  float invRow = rows - cell.y;
  float headPos = mod(u_time * speed + phase, rows + 20.0);
  float dist = headPos - invRow;

  // Stream length varies per column
  float streamLen = 8.0 + colSeed * 14.0;

  // Only render if within the stream trail
  if (dist < 0.0 || dist > streamLen) {
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
    return;
  }

  // Character changes over time, flicker rate scales with BPM
  float charSeed = hash(cell + floor(u_time * (2.0 + colSeed * 3.0) * tempo));

  float g = glyph(cellUV, charSeed);

  // Brightness: head is brightest, fades along tail
  float brightness;
  if (dist < 1.0) {
    // Head: white-hot
    brightness = 1.0;
  } else {
    brightness = max(0.05, 1.0 - (dist - 1.0) / (streamLen - 1.0));
    brightness *= brightness; // quadratic falloff
  }

  // Pick color from the 4 user colors based on column
  int colorIdx = int(mod(cell.x, 4.0));
  vec3 baseColor;
  if (colorIdx == 0) baseColor = u_color1;
  else if (colorIdx == 1) baseColor = u_color2;
  else if (colorIdx == 2) baseColor = u_color3;
  else baseColor = u_color4;

  // Neon boost: saturate and brighten
  vec3 neon = baseColor * 1.5;

  vec3 col;
  if (dist < 1.0) {
    // Head glows white-ish with color tint
    col = mix(neon, vec3(1.0), 0.7);
  } else {
    col = neon * brightness;
  }

  fragColor = vec4(col * g, 1.0);
}
