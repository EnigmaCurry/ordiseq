#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform float u_pitch;    // 0.0 to 360.0 - camera pitch angle in degrees
uniform float u_speed;    // 0.0 to 2.0 - movement speed
uniform float u_zoom;     // 0.5 to 2.0 - zoom level (affects central area size)
uniform float u_fisheye;  // 0.0 to 1.0 - blend between perspective (0) and fisheye (1)
uniform float u_bpm;      // beats per minute - drives movement speed
uniform float u_beat;     // continuous beat position from DAW
uniform float u_playing;  // 1.0 = playing, 0.0 = stopped
uniform vec3 u_color1;    // Grid color 1
uniform vec3 u_color2;    // Grid color 2
uniform vec3 u_color3;    // Grid color 3
uniform vec3 u_color4;    // Grid color 4
out vec4 fragColor;

const float PI = 3.14159265;
const float TAU = 6.28318530;

// --- Hash functions for pseudo-randomness ---
float hash(float p) {
  p = fract(p * 0.1031);
  p *= p + 33.33;
  p *= p + p;
  return fract(p);
}

float hash2(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

// --- Epicycle tip position at parametric time t ---
// 9 nested rotating circles with hash-derived periods
vec2 epicyclePos(float t, float seed) {
  vec2 pos = vec2(0.0);
  for (int i = 0; i < 9; i++) {
    float fi = float(i);
    float radius = 0.12 / (1.0 + fi * 0.55);
    float period = 1.0 + floor(hash(seed + fi * 7.13) * 7.0);
    float phase = hash(seed + fi * 3.71 + 10.0) * TAU;
    float dir = hash(seed + fi * 5.37 + 20.0) > 0.5 ? 1.0 : -1.0;
    pos += vec2(cos(dir * t * period + phase),
                sin(dir * t * period + phase)) * radius;
  }
  return pos;
}

// --- Phosphor bloom: hot white core + colored glow + wide halo ---
vec3 phosphor(float d, vec3 color) {
  // Layer 1: tight bright core (white-hot)
  float core = smoothstep(0.005, 0.001, d);
  // Layer 2: inner colored glow
  float inner = exp(-d * 220.0);
  // Layer 3: wide soft halo (phosphor bleed)
  float outer = exp(-d * 60.0);
  return vec3(1.0) * core * 1.5
       + color * inner * 1.8
       + color * outer * 0.5;
}

// --- Draw one epicycle system with phosphor rendering ---
vec3 drawEpicycle(vec2 uv, float time, vec2 center, float seed, vec3 color) {
  vec2 local = uv - center;
  vec3 result = vec3(0.0);

  float speed = 0.06 + hash(seed + 50.0) * 0.04;
  float t0 = time * speed;

  // Orbit circles showing the mechanism (phosphor rings)
  vec2 pivot = vec2(0.0);
  for (int i = 0; i < 9; i++) {
    float fi = float(i);
    float radius = 0.12 / (1.0 + fi * 0.55);
    float period = 1.0 + floor(hash(seed + fi * 7.13) * 7.0);
    float phase = hash(seed + fi * 3.71 + 10.0) * TAU;
    float dir = hash(seed + fi * 5.37 + 20.0) > 0.5 ? 1.0 : -1.0;
    float angle = dir * t0 * period + phase;

    // Node at pivot (center of this orbit circle)
    float nodeDist = length(local - pivot);
    float dimmer = 1.0 / (1.0 + fi * 0.3);
    result += phosphor(nodeDist, color) * 0.3 * dimmer;

    // Orbit ring with phosphor glow, dimmer for inner orbits
    float ringDist = abs(length(local - pivot) - radius);
    float ringLine = smoothstep(0.003, 0.0005, ringDist);
    float ringBloom = exp(-ringDist * 120.0);
    result += color * ringLine * 0.2 * dimmer + color * ringBloom * 0.06 * dimmer;

    pivot += vec2(cos(angle), sin(angle)) * radius;
  }

  // Trail with phosphor afterglow
  float trailSpan = TAU * 4.0;
  for (int i = 0; i < 100; i++) {
    float fi = float(i);
    float t = t0 - fi / 100.0 * trailSpan;
    vec2 pos = epicyclePos(t, seed);
    float d = length(local - pos);

    // Phosphor persistence: slow exponential decay
    float fade = exp(-fi * 0.025);
    // Tight core + soft glow per trail point
    float core = smoothstep(0.004, 0.0008, d);
    float glow = exp(-d * 180.0);
    result += color * (core * 0.6 + glow * 0.3) * fade;
  }

  // Tip: full phosphor bloom
  vec2 tipPos = epicyclePos(t0, seed);
  float d = length(local - tipPos);
  result += phosphor(d, color);

  return result;
}

// --- Oort Cloud: sparse shell of distant icy bodies ---
vec3 drawOortCloud(vec2 uv, float time) {
  vec3 result = vec3(0.0);

  // 3 concentric rings of particles
  for (int layer = 0; layer < 3; layer++) {
    float fl = float(layer);
    float ringRadius = 0.33 + fl * 0.055;

    for (int i = 0; i < 24; i++) {
      float fi = float(i);
      float seed = fi + fl * 100.0;

      // Angular position with slow orbital drift
      float baseAngle = fi / 24.0 * TAU + hash(seed) * 0.4;
      float orbitSpeed = 0.012 + hash(seed + 1.0) * 0.02;
      float dir = hash(seed + 2.0) > 0.5 ? 1.0 : -1.0;
      float a = baseAngle + time * orbitSpeed * dir;

      // Radial scatter within the ring
      float r = ringRadius + (hash(seed + 3.0) - 0.5) * 0.045;
      vec2 pos = vec2(cos(a), sin(a)) * r;
      float d = length(uv - pos);

      // Size and twinkle variation
      float size = 0.0008 + hash(seed + 4.0) * 0.0014;
      float twinkle = 0.35 + 0.65 * sin(time * (0.3 + hash(seed + 5.0) * 1.5) + hash(seed + 6.0) * TAU);

      // Ice-blue with slight color variation per body
      vec3 iceColor = vec3(0.55, 0.72, 0.92) + vec3(
        (hash(seed + 7.0) - 0.5) * 0.12,
        (hash(seed + 8.0) - 0.5) * 0.08,
        hash(seed + 9.0) * 0.08
      );

      // Phosphor-style particle: bright core + wide halo
      float core = smoothstep(size * 3.0, size * 0.2, d) * twinkle;
      float halo = exp(-d * 300.0) * twinkle;
      float bloom = exp(-d * 80.0) * twinkle;
      result += iceColor * core * 0.9 + iceColor * halo * 0.4 + iceColor * bloom * 0.12;
    }
  }

  // Diffuse haze band at Oort cloud distance
  float dist = length(uv);
  float haze = exp(-pow((dist - 0.36) / 0.09, 2.0)) * 0.06;
  result += vec3(0.4, 0.55, 0.8) * haze;

  return result;
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  vec2 centered = uv - 0.5;

  // Aspect ratio correction
  float aspect = u_resolution.x / u_resolution.y;
  centered.x *= aspect;

  // Apply zoom
  centered *= u_zoom;

  float r = length(centered);
  float maxR = 0.8 * u_zoom;

  vec3 baseColor = vec3(0.0);

  if (r < maxR) {
    // Fisheye / perspective projection
    float phi_fisheye = r * PI * 0.8;
    float phi_persp = atan(r, 1.0);
    float phi = mix(phi_persp, phi_fisheye, u_fisheye);
    float theta = atan(centered.y, centered.x);

    vec3 rayDir;
    rayDir.x = sin(phi) * cos(theta);
    rayDir.y = cos(phi);
    rayDir.z = sin(phi) * sin(theta);

    // Tilt camera based on pitch
    float pitchAngle = u_pitch * PI / 180.0;
    float cy = cos(pitchAngle);
    float sy = sin(pitchAngle);
    vec3 tilted = vec3(
      rayDir.x,
      rayDir.y * cy - rayDir.z * sy,
      rayDir.y * sy + rayDir.z * cy
    );

    // Ground plane
    if (tilted.y < -0.001) {
      float t = -1.0 / tilted.y;
      float x = tilted.x * t;
      float z = tilted.z * t;

      // Beat-synced movement
      float tempo = u_bpm / 120.0;
      float movement = mix(u_time * u_speed * 5.0 * tempo, u_beat * u_speed * 5.0, u_playing);
      z += movement;

      // Grid
      float gridSize = 1.0;
      float lineWidth = 0.03;

      float xGrid = mod(x, gridSize);
      float zGrid = mod(z, gridSize);
      float xAA = fwidth(x) * 1.5;
      float zAA = fwidth(z) * 1.5;

      float hLine = smoothstep(lineWidth + zAA, lineWidth, zGrid) +
                    smoothstep(gridSize - lineWidth - zAA, gridSize - lineWidth, zGrid);
      float vLine = smoothstep(lineWidth + xAA, lineWidth, xGrid) +
                    smoothstep(gridSize - lineWidth - xAA, gridSize - lineWidth, xGrid);

      float cellX = floor(x);
      float cellZ = floor(z);
      int colorIdx = int(mod(cellX + cellZ, 4.0));

      vec3 lineColor;
      if (colorIdx == 0) lineColor = u_color1;
      else if (colorIdx == 1) lineColor = u_color2;
      else if (colorIdx == 2) lineColor = u_color3;
      else lineColor = u_color4;

      float grid = min(1.0, hLine + vLine);
      float dist = length(vec2(x, z - movement));
      float fade = 1.0 - smoothstep(5.0, 30.0, dist);
      grid *= fade;

      baseColor = lineColor * grid * 0.3;
    }
  }

  // --- Overlays in screen space (aspect-corrected, zoom-independent) ---
  vec2 overlayUV = (uv - 0.5) * vec2(aspect, 1.0);

  // Epicycle: single system with drifting center
  float dt = u_time * 0.04;
  vec2 epiCenter = vec2(sin(dt * 0.7) * 0.05, cos(dt * 0.5) * 0.04);
  vec3 epicycles = drawEpicycle(overlayUV, u_time, epiCenter, 1.0, u_color1);

  // Oort cloud shell
  vec3 oort = drawOortCloud(overlayUV, u_time);

  fragColor = vec4(baseColor + epicycles + oort, 1.0);
}
