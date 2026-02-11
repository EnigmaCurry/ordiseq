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

vec3 rgb2hsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
  vec2 rawUV = gl_FragCoord.xy / u_resolution;
  float aspect = u_resolution.x / u_resolution.y;
  float tempo = u_bpm / 120.0;
  float t = mix(u_time * 0.3 * tempo, u_beat * 0.3, u_playing);

  // --- Continuous morph: fisheye breathes in and out slowly ---
  float bpm = max(u_bpm, 1.0);
  float rawTime = mix(u_time, u_beat * 60.0 / bpm, u_playing);
  float mt = rawTime * 0.02;
  float fishW = 0.5 + 0.5 * (
    sin(mt * 1.6180339) * 0.5 +
    sin(mt * 1.4142135 + 2.1) * 0.3 +
    sin(mt * 1.7320508 + 4.7) * 0.2
  );
  fishW = smoothstep(0.15, 0.85, fishW);

  // --- Fisheye with slowly evolving shape ---
  // Irrational ratios so the distortion never repeats the same way twice
  float slowT = rawTime * 0.031;

  // --- Slow color drift: gentle hue sway within the theme ---
  float hueShift = (
    sin(slowT * 0.87) * 0.5 +
    sin(slowT * 1.31 + 1.9) * 0.3 +
    sin(slowT * 0.53 + 4.1) * 0.2
  ) * 0.06;
  vec3 c1 = hsv2rgb(rgb2hsv(u_color1) + vec3(hueShift, 0.0, 0.0));
  vec3 c2 = hsv2rgb(rgb2hsv(u_color2) + vec3(hueShift, 0.0, 0.0));
  vec3 c3 = hsv2rgb(rgb2hsv(u_color3) + vec3(hueShift, 0.0, 0.0));
  vec3 c4 = hsv2rgb(rgb2hsv(u_color4) + vec3(hueShift, 0.0, 0.0));

  // Fisheye center — multi-component so it wanders unpredictably
  vec2 fishCenter = vec2(
    sin(slowT * 1.6180339) * 0.07 + sin(slowT * 0.73 + 3.7) * 0.06,
    sin(slowT * 1.4142135) * 0.05 + sin(slowT * 0.91 + 1.3) * 0.05
  );

  // Barrel strength — can dip negative (pincushion)
  float fishStrength = 1.0 + (
    sin(slowT * 1.7320508) * 0.5 +
    sin(slowT * 0.67 + 2.1) * 0.3 +
    sin(slowT * 1.13 + 4.5) * 0.2
  ) * 1.5;

  // Asymmetric squeeze — x and y vary independently
  float sqX = 1.0 + sin(slowT * 2.2360679) * 0.25;
  float sqY = 1.0 + sin(slowT * 1.8392867) * 0.20;

  // Swirl — multi-component so direction truly alternates
  float swirlAmt =
    sin(slowT * 0.618) * 0.20 +
    sin(slowT * 1.17 + 3.3) * 0.15 +
    sin(slowT * 0.41 + 1.7) * 0.10;

  // Wave distortion — ripple that fades in and out
  float waveAmt = sin(slowT * 0.53 + 2.9) * 0.5 + 0.5;
  waveAmt *= waveAmt;
  float waveAngle = slowT * 0.31;

  vec2 delta = (rawUV - 0.5 - fishCenter) * vec2(aspect * sqX, sqY);
  float r2 = dot(delta, delta);

  // Swirl: rotate proportional to distance from center
  float ang = swirlAmt * r2;
  float cs = cos(ang), sn = sin(ang);
  delta = vec2(cs * delta.x - sn * delta.y, sn * delta.x + cs * delta.y);

  // Barrel/pincushion distortion
  vec2 distortedUV = delta * (1.0 + fishStrength * r2);

  // Wave overlay — directional ripple through the image
  vec2 waveDir = vec2(cos(waveAngle), sin(waveAngle));
  float waveDot = dot(delta, waveDir);
  distortedUV += waveDir * sin(waveDot * 12.0) * 0.03 * waveAmt;

  distortedUV /= vec2(aspect * sqX, sqY);
  vec2 uv = mix(rawUV, distortedUV + 0.5 + fishCenter, fishW);

  // --- Camera: slowly drifting pitch and flight height ---
  float camPitch =
    sin(slowT * 0.61) * 0.5 +
    sin(slowT * 0.89 + 3.1) * 0.3 +
    sin(slowT * 0.37 + 1.4) * 0.2;
  float horizon = 0.40 + camPitch * 0.12;
  float camH =
    sin(slowT * 0.47) * 0.5 +
    sin(slowT * 0.79 + 2.7) * 0.3 +
    sin(slowT * 0.31 + 5.3) * 0.2;
  float heightOffset = 0.015 + (camH * 0.5 + 0.5) * 0.025;

  // --- Sun position (computed early for mirage haze) ---
  float sunPhase =
    sin(slowT * 0.73) * 0.5 +
    sin(slowT * 1.07 + 2.3) * 0.3 +
    sin(slowT * 0.41 + 5.1) * 0.2;
  float sunY = horizon + 0.10 + sunPhase * 0.30;
  float sunProximity = 1.0 - smoothstep(0.0, 0.25, sunY - horizon);
  vec2 sunCenter = vec2(0.5, sunY);
  float sunRadius = 0.15 + sunProximity * 0.12;

  // --- Heat mirage: horizon + sun circumference ---
  float mirageIntensity = 0.5 + 0.5 * (
    sin(slowT * 0.87) * 0.5 +
    sin(slowT * 1.31 + 1.9) * 0.3 +
    sin(slowT * 0.53 + 4.1) * 0.2
  );
  mirageIntensity = mirageIntensity * mirageIntensity;
  float mirageSpeed = 1.0 + mirageIntensity * 2.0;

  // Horizon haze band
  float horizDist = abs(uv.y - horizon);
  float mirageSpread = 0.05 + mirageIntensity * 0.10;
  float horizHaze = smoothstep(mirageSpread, 0.0, horizDist);

  // Sun circumference haze
  vec2 toSun = vec2((uv.x - sunCenter.x) * aspect, uv.y - sunCenter.y);
  float distToSun = length(toSun);
  float sunEdgeDist = abs(distToSun - sunRadius);
  float sunHaze = smoothstep(0.06, 0.0, sunEdgeDist);

  float mirageMask = max(horizHaze, sunHaze);
  float mirageAmt = mirageMask * mirageIntensity;
  uv.y += mirageAmt * (
    sin(uv.x * 40.0 * aspect + rawTime * 1.3 * mirageSpeed) * 0.005 +
    sin(uv.x * 25.0 * aspect + rawTime * 0.9 * mirageSpeed + 1.7) * 0.004 +
    sin(uv.x * 60.0 * aspect + rawTime * 2.1 * mirageSpeed + 3.2) * 0.003
  );
  uv.x += sunHaze * mirageIntensity * (
    sin(uv.y * 50.0 + rawTime * 1.7 * mirageSpeed) * 0.003 +
    sin(uv.y * 35.0 + rawTime * 1.1 * mirageSpeed + 2.9) * 0.002
  );

  vec3 col = vec3(0.0);

  // --- Sky gradient ---
  if (uv.y > horizon) {
    float skyT = clamp((uv.y - horizon) / (1.0 - horizon), 0.0, 1.0);
    vec3 skyTop = c3 * 0.1;
    vec3 skyBot = mix(c1, c2, 0.3) * 0.6;
    col = mix(skyBot, skyTop, skyT);

    // --- Sun rendering (position already computed above) ---
    vec2 sunUV = vec2((uv.x - sunCenter.x) * aspect, uv.y - sunCenter.y);
    float sunDist = length(sunUV);

    if (sunDist < sunRadius) {
      float stripeFreq = 40.0;
      float stripe = step(0.5, fract(uv.y * stripeFreq));
      float gapGrow = smoothstep(sunCenter.y, sunCenter.y - sunRadius, uv.y);
      float stripeWidth = mix(0.0, 0.7, gapGrow);
      float sunMask = 1.0 - step(1.0 - stripeWidth, stripe) * step(0.3, gapGrow);

      vec3 sunTop = c2;
      vec3 sunBot = c1;
      float sunGrad = (uv.y - (sunCenter.y - sunRadius)) / (2.0 * sunRadius);
      vec3 sunCol = mix(sunBot, sunTop, clamp(sunGrad, 0.0, 1.0));
      col = mix(col, sunCol * 1.5, sunMask * smoothstep(sunRadius, sunRadius - 0.005, sunDist));
    }

    // Sun glow
    float glow = exp(-sunDist * 4.0) * 0.4;
    col += mix(c1, c2, 0.5) * glow;

    // Stars (above horizon, never over sun or its glow)
    float starMask = smoothstep(sunRadius, sunRadius + 0.08, sunDist);
    if (skyT > 0.3) {
      vec2 starUV = floor(uv * vec2(80.0 * aspect, 80.0));
      float starHash = fract(sin(dot(starUV, vec2(127.1, 311.7))) * 43758.5453);
      float twinkle = sin(t * 3.0 + starHash * 6.28) * 0.5 + 0.5;
      if (starHash > 0.985) {
        col += vec3(0.6 + 0.4 * twinkle) * (skyT - 0.3) * 2.0 * starMask;
      }
    }
  }

  // --- Ground: perspective grid ---
  if (uv.y <= horizon) {
    float groundT = (horizon - uv.y) / horizon;
    float z = 1.0 / (groundT + heightOffset);
    float x = (uv.x - 0.5) * aspect * z;

    // Scrolling
    z += t * 8.0;

    // Grid lines with screen-space anti-aliasing
    float gridSpaceX = x * 0.5;
    float gridSpaceZ = z * 0.2;
    float gridX = abs(fract(gridSpaceX) - 0.5);
    float gridZ = abs(fract(gridSpaceZ) - 0.5);
    float aaX = fwidth(gridSpaceX) * 1.5;
    float aaZ = fwidth(gridSpaceZ) * 1.5;
    float lineWidth = 0.03;
    float lineX = smoothstep(lineWidth + aaX, lineWidth, gridX);
    float lineZ = smoothstep(lineWidth + aaZ, lineWidth, gridZ);
    float grid = max(lineX, lineZ);

    // Distance fade
    float fade = 1.0 - smoothstep(0.0, 0.8, groundT);
    grid *= (0.3 + 0.7 * fade);

    // Determine color by cell
    float cellX = floor(x * 0.5);
    float cellZ = floor(z * 0.2);
    int ci = int(mod(abs(cellX) + abs(cellZ), 4.0));
    vec3 gridCol;
    if (ci == 0) gridCol = c1;
    else if (ci == 1) gridCol = c2;
    else if (ci == 2) gridCol = c3;
    else gridCol = c4;

    // Dark ground base with grid overlay
    vec3 groundBase = c3 * 0.02;
    col = groundBase + gridCol * grid;

    // Horizon glow on ground
    float horizGlow = exp(-groundT * 6.0) * 0.3;
    col += c1 * horizGlow;
  }

  // --- Scanlines ---
  float scanline = 0.92 + 0.08 * sin(gl_FragCoord.y * 3.0);
  col *= scanline;

  // Slight vignette (use rawUV so vignette stays stable during fisheye)
  vec2 vig = rawUV - 0.5;
  col *= 1.0 - dot(vig, vig) * 0.5;

  fragColor = vec4(col * 0.5, 1.0);
}
