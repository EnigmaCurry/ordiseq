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

// Draws a classic saucer UFO shape, blends into col
void drawSaucer(vec2 d, float sW, float sH, float time,
                vec3 ca, vec3 cb, vec3 cc, vec3 cd,
                float glowStr, float colorSeed,
                inout vec3 col, float vis) {
  float sDist = length(d / vec2(sW, sH));
  float saucer = smoothstep(1.1, 0.85, sDist);
  float dOffy = sH * 1.125;
  float dDist = length((d - vec2(0.0, dOffy)) / vec2(sW * 0.4, sH * 2.0));
  float dome = smoothstep(1.1, 0.85, dDist) * smoothstep(0.0, sH * 0.5, d.y);
  float rimSum = 0.0;
  for (int i = 0; i < 6; i++) {
    float a = float(i) * 1.0472 + time * 1.5;
    vec2 lp = vec2(cos(a) * sW * 0.88, sin(a) * sH * 0.88);
    rimSum += exp(-length(d - lp) * min(11.0 / sW, 400.0));
  }
  float botGlow = exp(-length(d / vec2(sW * 0.6, sH * 1.25) + vec2(0.0, 1.2)) * 2.0);
  botGlow *= glowStr;
  float body = max(saucer, dome);
  vec3 uCol = cc * 0.12;
  uCol = mix(uCol, mix(cb, vec3(0.7), 0.25) * 0.5, dome / max(body, 0.001));
  uCol += mix(ca, cb, 0.5) * botGlow * 0.7;
  uCol += mix(ca, cd, fract(time * 0.2 + colorSeed)) * rimSum;
  col = mix(col, uCol, body * vis);
  col += mix(ca, cb, 0.5) * exp(-length(d) * min(1.3 / sW, 50.0)) * 0.1 * vis;
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
  float heightOffset = 0.01 + (camH * 0.5 + 0.5) * 0.0012;

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
    vec3 gridResult = groundBase + gridCol * grid;

    // --- Road: center lane heading toward the sun ---
    float roadHalf = 3.0;
    float roadAA = fwidth(x) * 2.0;
    float roadBlend = smoothstep(roadHalf + roadAA, roadHalf - roadAA, abs(x));

    vec3 roadSurface = c4 * 0.12;
    float markW = 0.12;
    float markAA = fwidth(x) * 1.5;

    // Solid edge lines
    float edgeLine = smoothstep(markW + markAA, markW, abs(abs(x) - roadHalf));

    // Dashed center line
    float dashZ = step(0.5, fract(z * 0.05));
    float centerLine = smoothstep(markW + markAA, markW, abs(x)) * dashZ;

    float marks = max(edgeLine, centerLine) * (0.3 + 0.7 * fade);
    vec3 roadCol = mix(roadSurface, c2, marks);

    col = mix(gridResult, roadCol, roadBlend);

    // Horizon glow on ground
    float horizGlow = exp(-groundT * 6.0) * 0.3;
    col += c1 * horizGlow;
  }

  // --- UFO encounters ---
  {
    float ufoEpoch = 55.0;
    float ufoSlot = floor(rawTime / ufoEpoch);
    float ufoRng = fract(sin(ufoSlot * 127.31 + 91.7) * 43758.5453);

    if (ufoRng > 0.45) {
      float lt = fract(rawTime / ufoEpoch);
      float typeRng = fract(ufoRng * 3.71);

      if (typeRng < 0.34) {
        // ====== Encounter 1: Solo saucer with tractor beam ======
        float ufoVis = smoothstep(0.05, 0.14, lt) * smoothstep(0.95, 0.86, lt);
        if (ufoVis > 0.001) {
          float dir = step(0.5, fract(ufoRng * 7.31)) * 2.0 - 1.0;
          float ft = lt * lt * (3.0 - 2.0 * lt);
          ft = ft * ft * (3.0 - 2.0 * ft);
          float ufoX = 0.5 + dir * (0.7 - 1.4 * ft);
          float ufoY = horizon + 0.20 + sin(ft * 3.14159) * 0.05
                       + sin(rawTime * 1.7) * 0.003;
          vec2 dU = vec2((uv.x - ufoX) * aspect, uv.y - ufoY);
          float beamStr = smoothstep(0.15, 0.05, abs(ufoX - 0.5));

          // Tractor beam
          if (beamStr > 0.01 && uv.y < ufoY - 0.005) {
            float bTop = ufoY - 0.005;
            float bBot = horizon - 0.25;
            float bT = clamp((bTop - uv.y) / (bTop - bBot), 0.0, 1.0);
            float hw = mix(0.012, 0.11, bT * bT) / aspect;
            float be = smoothstep(1.0, 0.4, abs(uv.x - ufoX) / hw);
            be *= beamStr * ufoVis * (1.0 - bT * 0.35);
            be *= 0.55 + 0.45 * sin(uv.y * 80.0 - rawTime * 4.0);
            float spark = sin(uv.y * 140.0 + rawTime * 2.5)
                        * sin(uv.x * 100.0 * aspect - rawTime * 0.8);
            be += max(spark, 0.0) * 0.15 * be;
            col += mix(c1, c2, 0.3) * be * 0.3;
          }

          drawSaucer(dU, 0.05, 0.008, rawTime, c1, c2, c3, c4,
                     0.4 + 0.6 * beamStr, 0.0, col, ufoVis);
        }

      } else if (typeRng < 0.67) {
        // ====== Encounter 2: Triangle formation — warp to sun, dance, vanish ======
        float ufoVis = smoothstep(0.06, 0.14, lt) * smoothstep(0.88, 0.78, lt);
        if (ufoVis > 0.001) {
          float dir = step(0.5, fract(ufoRng * 5.13)) * 2.0 - 1.0;

          // Phase boundaries
          float warpEnd = 0.20;
          float danceStart = 0.22;
          float danceEnd = 0.72;

          // Dance position (computed for all phases, overridden during warp)
          float danceLt = clamp(lt, danceStart, danceEnd);
          float dp = (danceLt - danceStart) / (danceEnd - danceStart);
          float jumpT = dp * 7.0;
          float jumpIdx = floor(jumpT);
          float snap = smoothstep(0.0, 0.04, fract(jumpT));
          float ja1 = jumpIdx * 1.618 * 6.2832;
          float ja2 = (jumpIdx + 1.0) * 1.618 * 6.2832;
          float jr1 = 0.09 + sin(jumpIdx * 2.3) * 0.03;
          float jr2 = 0.09 + sin((jumpIdx + 1.0) * 2.3) * 0.03;
          vec2 fp1 = sunCenter + vec2(cos(ja1) * jr1, sin(ja1) * jr1 * 0.5);
          vec2 fp2 = sunCenter + vec2(cos(ja2) * jr2, sin(ja2) * jr2 * 0.5);
          vec2 formCenter = mix(fp1, fp2, snap);
          float formRot = rawTime * 2.5 + snap * 0.8;
          float formSize = 0.045 + sin(jumpIdx * 1.7) * 0.008;

          if (lt < warpEnd) {
            // Non-Newtonian warp in: constant velocity, instant start
            float wt = clamp((lt - 0.06) / (warpEnd - 0.06), 0.0, 1.0);
            vec2 startP = vec2(0.5 + dir * 0.9, horizon + 0.32);
            formCenter = mix(startP, sunCenter + vec2(0.0, 0.06), wt);
            formRot = rawTime * 2.0;
            formSize = 0.05;
          } else if (lt > danceEnd) {
            // Non-Newtonian exit: instant acceleration upward
            float exitT = clamp((lt - danceEnd) / (0.82 - danceEnd), 0.0, 1.0);
            float exitD = smoothstep(0.0, 0.12, exitT);
            formCenter += vec2(dir * 0.2, 0.7) * exitD;
            formSize *= max(1.0 - exitT * 1.5, 0.0);
          }

          // Compute saucer positions
          vec2 sPos[3];
          for (int i = 0; i < 3; i++) {
            float ta = formRot + float(i) * 2.0944;
            sPos[i] = formCenter + vec2(cos(ta) * formSize,
                                        sin(ta) * formSize * 0.6);
          }

          // Energy connections between formation members
          for (int i = 0; i < 3; i++) {
            int ni = i < 2 ? i + 1 : 0;
            vec2 pa = vec2((uv.x - sPos[i].x) * aspect, uv.y - sPos[i].y);
            vec2 ba = vec2((sPos[ni].x - sPos[i].x) * aspect,
                            sPos[ni].y - sPos[i].y);
            float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
            float ld = length(pa - ba * h);
            float line = exp(-ld * 250.0) * 0.15;
            line *= 0.6 + 0.4 * sin(h * 12.0 - rawTime * 3.0);
            col += mix(c1, c4, 0.5) * line * ufoVis;
          }

          // Draw the three saucers
          for (int i = 0; i < 3; i++) {
            vec2 sd = vec2((uv.x - sPos[i].x) * aspect, uv.y - sPos[i].y);
            drawSaucer(sd, 0.028, 0.005, rawTime, c1, c2, c3, c4,
                       0.5, float(i) * 0.33, col, ufoVis);
          }
        }

      } else {
        // ====== Encounter 3: Cylindrical UFO rotating above road ======
        float ufoVis = smoothstep(0.06, 0.16, lt) * smoothstep(0.93, 0.82, lt);
        if (ufoVis > 0.001) {
          // Descent / hover / ascent
          float hoverY = horizon + 0.22;
          float cylY;
          if (lt < 0.20) {
            float dsc = clamp((lt - 0.06) / 0.14, 0.0, 1.0);
            cylY = mix(1.1, hoverY, dsc * dsc * (3.0 - 2.0 * dsc));
          } else if (lt < 0.75) {
            cylY = hoverY + sin(rawTime * 0.7) * 0.008;
          } else {
            float asc = clamp((lt - 0.75) / 0.18, 0.0, 1.0);
            cylY = mix(hoverY, 1.1, asc * asc);
          }
          float cylX = 0.5 + sin(rawTime * 0.13) * 0.03;
          vec2 dC = vec2((uv.x - cylX) * aspect, uv.y - cylY);

          // Rounded-rectangle SDF (capsule ends = cylinder profile)
          float cW = 0.07, cH = 0.016, cR = 0.016;
          vec2 q = abs(dC) - vec2(cW - cR, max(cH - cR, 0.001));
          float cylSDF = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - cR;
          float cylMask = smoothstep(0.003, 0.0, cylSDF);

          // Surface bands showing axial rotation
          float sa = asin(clamp(dC.y / (cH * 0.95), -1.0, 1.0));
          float rot = rawTime * 1.8;
          float band1 = sin((sa + rot) * 5.0) * 0.5 + 0.5;
          float band2 = sin((sa + rot) * 11.0 + 1.7) * 0.5 + 0.5;

          // Running lights along the length
          float lightPhase = fract(dC.x / (cW * 2.0) * 8.0 + rawTime * 0.4);
          float lightBelt = smoothstep(0.5, 0.2, abs(sa) / 1.5708);
          float lights = smoothstep(0.15, 0.08, abs(lightPhase - 0.5))
                       * lightBelt * cylMask;

          // Cylinder color
          vec3 cylCol = c3 * 0.08 + c4 * 0.05;
          cylCol += c2 * band1 * 0.1;
          cylCol += c1 * band2 * 0.05;
          cylCol += mix(c1, c2, fract(lightPhase * 3.0 + rawTime * 0.2))
                   * lights * 1.5;

          // Edge rim highlight
          float edgeH = smoothstep(0.0, 0.004, -cylSDF)
                      * smoothstep(0.012, 0.003, -cylSDF);
          cylCol += mix(c1, c2, 0.5) * edgeH * 0.3;

          col = mix(col, cylCol, cylMask * ufoVis);

          // Glow halo
          col += mix(c1, c2, 0.5) * exp(-abs(cylSDF) * 50.0) * 0.12 * ufoVis;

          // Searchlight sweeping the ground
          float searchA = rawTime * 0.6;
          float searchX = cylX + sin(searchA) * 0.12 / aspect;
          // Light cone from cylinder to ground
          float coneTopY = cylY - 0.01;
          float groundSearchY = horizon - 0.08;
          if (uv.y < coneTopY && uv.y > groundSearchY - 0.05) {
            float ct = clamp((coneTopY - uv.y) / (coneTopY - groundSearchY), 0.0, 1.0);
            float coneHW = mix(0.008, 0.045, ct) / aspect;
            float coneX = mix(cylX, searchX, ct);
            float coneDist = abs(uv.x - coneX) / coneHW;
            float cone = smoothstep(1.0, 0.5, coneDist) * 0.12 * ct;
            col += mix(c1, c2, 0.3) * cone * ufoVis;
          }
          // Ground spot
          if (uv.y <= horizon) {
            vec2 searchD = vec2((uv.x - searchX) * aspect,
                                (uv.y - groundSearchY) * 2.5);
            float searchLight = exp(-dot(searchD, searchD) * 50.0) * 0.2;
            col += mix(c1, c2, 0.3) * searchLight * ufoVis;
          }
        }
      }
    }
  }

  // --- Scanlines ---
  float scanline = 0.92 + 0.08 * sin(gl_FragCoord.y * 3.0);
  col *= scanline;

  // Slight vignette (use rawUV so vignette stays stable during fisheye)
  vec2 vig = rawUV - 0.5;
  col *= 1.0 - dot(vig, vig) * 0.5;

  fragColor = vec4(col * 0.5, 1.0);
}
