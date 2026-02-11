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

float hash(vec2 p) {
  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(vec2 p) {
  float v = 0.0;
  float a = 0.5;
  vec2 shift = vec2(100.0);
  for (int i = 0; i < 5; i++) {
    v += a * noise(p);
    p = p * 2.0 + shift;
    a *= 0.5;
  }
  return v;
}

float pyramidShape(vec2 uv, vec2 base, float halfWidth, float height) {
  vec2 p = uv - base;
  p.x = abs(p.x);
  if (p.y < 0.0) return 1.0;
  if (p.y > height) return 1.0;
  float edge = halfWidth * (1.0 - p.y / height);
  return p.x - edge;
}

// --- Matrix glyph rendering ---
float strokeLine(float d, float w) {
  return 1.0 - smoothstep(w - 0.02, w + 0.03, d);
}

float glyph(vec2 cellUV, float seed) {
  vec2 p = cellUV - 0.5;
  float w = 0.065;
  int t = int(floor(fract(seed * 7.31) * 20.0));
  float s = 0.0;

  if (t == 0) {
    s = strokeLine(abs(p.x), w);
  } else if (t == 1) {
    s = strokeLine(abs(p.y), w);
  } else if (t == 2) {
    s = strokeLine(min(abs(p.x), abs(p.y)), w);
  } else if (t == 3) {
    s = strokeLine(min(abs(p.x - p.y), abs(p.x + p.y)) * 0.7071, w);
  } else if (t == 4) {
    s = strokeLine(abs(length(p) - 0.28), w);
  } else if (t == 5) {
    s = 1.0 - smoothstep(0.15, 0.19, length(p));
  } else if (t == 6) {
    float bx = max(abs(p.x), abs(p.y));
    s = strokeLine(abs(bx - 0.28), w);
  } else if (t == 7) {
    float d = min(min(abs(p.y - 0.22), abs(p.y)), abs(p.y + 0.22));
    s = strokeLine(d, w * 0.55);
  } else if (t == 8) {
    float dd = abs(p.x) + abs(p.y);
    s = strokeLine(abs(dd - 0.32), w);
  } else if (t == 9) {
    float d1 = min(abs(p.x), abs(p.y));
    float d2 = min(abs(p.x - p.y), abs(p.x + p.y)) * 0.7071;
    s = strokeLine(min(d1, d2), w * 0.75);
  } else if (t == 10) {
    float cd = abs(length(p) - 0.3);
    s = strokeLine(cd, w) * smoothstep(-0.05, 0.05, p.y);
  } else if (t == 11) {
    float vert = strokeLine(abs(p.x + 0.15), w) * step(0.0, p.y);
    float horiz = strokeLine(abs(p.y), w) * step(-0.15, p.x);
    s = max(vert, horiz);
  } else if (t == 12) {
    vec2 tp = vec2(abs(p.x), p.y + 0.05);
    float tri = max(-tp.y - 0.2, tp.x * 0.866 + tp.y * 0.5 - 0.22);
    s = strokeLine(abs(tri), w);
  } else if (t == 13) {
    vec2 tp = vec2(abs(p.x), -p.y + 0.05);
    float tri = max(-tp.y - 0.2, tp.x * 0.866 + tp.y * 0.5 - 0.22);
    s = strokeLine(abs(tri), w);
  } else if (t == 14) {
    float stem = strokeLine(abs(p.x), w) * step(p.y, 0.15);
    float chevron = strokeLine(abs(abs(p.x) + p.y - 0.35) * 0.707, w)
                  * step(0.12, p.y) * step(p.y, 0.4);
    s = max(stem, chevron);
  } else if (t == 15) {
    float circ = strokeLine(abs(length(p) - 0.28), w * 0.7);
    float pl = strokeLine(min(abs(p.x), abs(p.y)), w * 0.5);
    s = max(circ, pl);
  } else if (t == 16) {
    float vert = strokeLine(abs(p.x), w) * step(p.y, 0.25);
    float horiz = strokeLine(abs(p.y - 0.25), w);
    s = max(vert, horiz);
  } else if (t == 17) {
    float wave = abs(p.y - 0.15 * sin(p.x * 10.0));
    s = strokeLine(wave, w);
  } else if (t == 18) {
    float cd = abs(length(p) - 0.3);
    s = strokeLine(cd, w) * smoothstep(0.05, -0.05, p.y);
  } else {
    vec2 g = floor(cellUV * vec2(3.0, 5.0));
    float on = step(0.45, hash(g + seed * 17.3));
    vec2 f = fract(cellUV * vec2(3.0, 5.0));
    float mask = smoothstep(0.0, 0.15, f.x) * smoothstep(1.0, 0.85, f.x)
               * smoothstep(0.0, 0.15, f.y) * smoothstep(1.0, 0.85, f.y);
    s = on * mask;
  }

  float margin = smoothstep(0.0, 0.08, cellUV.x) * smoothstep(1.0, 0.92, cellUV.x)
               * smoothstep(0.0, 0.08, cellUV.y) * smoothstep(1.0, 0.92, cellUV.y);
  return s * margin;
}

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float aspect = u_resolution.x / u_resolution.y;
  float tempo = u_bpm / 120.0;
  float t = mix(u_time * 0.15 * tempo, u_beat * 0.15, u_playing);
  float rawTime = mix(u_time, u_beat * 60.0 / max(u_bpm, 1.0), u_playing);
  float syncTime = mix(u_time, u_beat / tempo, u_playing);

  float horizon = 0.32;

  // --- Day/night cycle ---
  // 90s full cycle with explicit phase zones:
  //   Dawn  0.00–0.18  (~16s)  gradual brightening
  //   Day   0.18–0.50  (~29s)  full bright plateau
  //   Dusk  0.50–0.78  (~25s)  long, slow sunset
  //   Night 0.78–1.00  (~20s)  darkness
  float cyclePeriod = 90.0;
  float cycle = fract(rawTime / cyclePeriod);

  float dawnEnd   = 0.18;
  float duskStart = 0.50;
  float duskEnd   = 0.78;

  float arcLight;
  if (cycle < dawnEnd) {
    arcLight = smoothstep(0.0, dawnEnd, cycle);
  } else if (cycle < duskStart) {
    arcLight = 1.0;
  } else if (cycle < duskEnd) {
    arcLight = smoothstep(duskEnd, duskStart, cycle);
  } else {
    arcLight = 0.0;
  }

  float daylight = arcLight;
  // Golden hour: peaks during the transitions (dawn and dusk)
  float goldenHour = smoothstep(0.0, 0.4, arcLight) * smoothstep(0.85, 0.4, arcLight);
  float nightAmount = 1.0 - daylight;

  // --- Sky ---
  vec3 col = vec3(0.0);

  if (uv.y > horizon) {
    float skyT = (uv.y - horizon) / (1.0 - horizon);

    // Night sky
    vec3 nightBot = u_color4 * 0.2;
    vec3 nightTop = u_color4 * 0.08;
    vec3 nightSky = mix(nightBot, nightTop, skyT);

    // Sunset sky
    vec3 sunsetBottom = mix(u_color1, u_color2, 0.3) * 0.9;
    vec3 sunsetMid = mix(u_color2, u_color3, 0.5) * 0.5;
    vec3 sunsetTop = u_color4 * 0.4;
    vec3 sunsetSky;
    if (skyT < 0.3) {
      sunsetSky = mix(sunsetBottom, sunsetMid, skyT / 0.3);
    } else {
      sunsetSky = mix(sunsetMid, sunsetTop, (skyT - 0.3) / 0.7);
    }

    // Day sky — bright and washed out
    vec3 dayBottom = mix(u_color1, u_color2, 0.5) * 1.1;
    vec3 dayTop = mix(u_color3, u_color4, 0.3) * 0.55;
    vec3 daySky = mix(dayBottom, dayTop, skyT);

    // Blend: night -> sunset -> day -> sunset -> night
    vec3 sky = nightSky;
    sky = mix(sky, sunsetSky, goldenHour + daylight * 0.2);
    sky = mix(sky, daySky, daylight * (1.0 - goldenHour * 0.5));

    col = sky;

    // Soft diffuse clouds
    vec2 cloudUV = vec2(uv.x * aspect * 1.5 + t * 0.08, skyT * 2.0 + t * 0.02);
    float cloud1 = fbm(cloudUV * 2.0 + vec2(0.0, 7.3));
    float cloud2 = fbm(cloudUV * 3.0 + vec2(13.0, 0.0) - t * 0.03);
    cloud1 = smoothstep(0.40, 0.70, cloud1);
    cloud2 = smoothstep(0.45, 0.72, cloud2);
    float cloudBand = smoothstep(0.0, 0.08, skyT) * smoothstep(0.5, 0.2, skyT);
    float cloudAlpha = (cloud1 * 0.6 + cloud2 * 0.4) * cloudBand;

    vec3 cloudLitSunset = mix(u_color1, u_color2, cloud2) * 0.5;
    vec3 cloudLitDay = mix(u_color1, u_color3, 0.3) * 0.3;
    vec3 cloudLit = mix(cloudLitSunset, cloudLitDay, daylight * (1.0 - goldenHour));
    vec3 cloudDark = u_color4 * 0.1;
    float cloudVis = max(goldenHour, daylight * 0.5);
    col = mix(col, mix(cloudDark, cloudLit, cloud1), cloudAlpha * 0.35 * cloudVis);

    // Horizon haze glow
    float horizGlow = exp(-(skyT * skyT) * 15.0) * mix(0.03, 0.3, max(goldenHour, daylight * 0.3));
    col += u_color1 * horizGlow;

    // --- Matrix rain in the sky ---
    // Ghostly during day, prominent at night
    float matrixIntensity = mix(0.08, 0.5, nightAmount);

    vec3 matrixCol = vec3(0.0);
    for (int layer = 0; layer < 4; layer++) {
      float fi = float(layer);
      float depth = 0.5 + fi * 0.4;

      // Column grid — bigger glyphs for closer layers
      float cols = 16.0 + fi * 5.0;
      float cellW = u_resolution.x / cols;
      float cellH = cellW * 1.6;

      vec2 fragPos = gl_FragCoord.xy;

      vec2 cell = vec2(
        floor(fragPos.x / cellW),
        floor(fragPos.y / cellH)
      );
      vec2 cellUV = vec2(
        fract(fragPos.x / cellW),
        fract(fragPos.y / cellH)
      );

      // Per-column stream
      float colSeed = hash(vec2(cell.x, fi * 73.13));
      float speed = (1.0 + colSeed * 2.0) * tempo;
      float phase = colSeed * 100.0;
      float streamLen = 3.0 + colSeed * 6.0;
      float gapLen = 16.0 + colSeed * 24.0;
      float period = streamLen + gapLen;

      float cyclePos = mod(cell.y - syncTime * speed + phase, period);

      if (cyclePos < streamLen) {
        float dist = cyclePos;

        float charSeed = hash(cell + vec2(fi * 37.0, floor(syncTime * (1.5 + colSeed * 2.0) * tempo)));
        float g = glyph(cellUV, charSeed);

        // Head is bright, tail fades
        float brightness;
        if (dist < 1.0) {
          brightness = 1.0;
        } else {
          brightness = max(0.05, 1.0 - (dist - 1.0) / (streamLen - 1.0));
          brightness *= brightness;
        }

        // Depth fog
        float fog = 1.0 / (1.0 + fi * 0.6);

        // Color from palette per column
        int colorIdx = int(mod(cell.x + fi, 4.0));
        vec3 baseColor;
        if (colorIdx == 0) baseColor = u_color1;
        else if (colorIdx == 1) baseColor = u_color2;
        else if (colorIdx == 2) baseColor = u_color3;
        else baseColor = u_color4;

        vec3 glyphCol;
        if (dist < 1.0) {
          glyphCol = mix(baseColor * 1.5, vec3(1.0), 0.6);
        } else {
          glyphCol = baseColor * 1.5 * brightness;
        }

        matrixCol += glyphCol * g * fog;
      }
    }

    // Fade matrix near horizon so it doesn't clash with the landscape
    float matrixFade = smoothstep(0.0, 0.15, skyT);
    col += matrixCol * matrixIntensity * matrixFade;

    // --- Stars (fade in at night) ---
    if (nightAmount > 0.05 && skyT > 0.15) {
      vec2 starUV = floor(uv * vec2(90.0 * aspect, 90.0));
      float starHash = hash(starUV);
      float twinkle = sin(t * 4.0 + starHash * 6.28) * 0.5 + 0.5;
      if (starHash > 0.986) {
        float starBright = (0.5 + 0.5 * twinkle) * nightAmount;
        starBright *= smoothstep(0.15, 0.4, skyT);
        col += vec3(starBright);
      }
    }
  }

  // --- Desert ground ---
  if (uv.y <= horizon) {
    float groundT = (horizon - uv.y) / horizon;

    vec3 sandNearDay = mix(u_color1, u_color3, 0.4) * 0.45;
    vec3 sandFarDay = mix(u_color1, u_color2, 0.3) * 0.55;
    vec3 sandNearNight = u_color4 * 0.12;
    vec3 sandFarNight = u_color4 * 0.08;

    vec3 sandNear = mix(sandNearNight, sandNearDay, max(daylight, goldenHour * 0.7));
    vec3 sandFar = mix(sandFarNight, sandFarDay, max(daylight, goldenHour * 0.7));
    vec3 sandCol = mix(sandFar, sandNear, groundT);

    float sandNoise = fbm(vec2(uv.x * 20.0 * aspect, uv.y * 30.0 + t * 0.5)) * 0.08;
    sandCol += sandNoise * u_color1 * 0.3 * max(daylight, goldenHour * 0.5);

    col = sandCol;

    // Horizon glow on sand
    float sandHorizGlow = exp(-groundT * 8.0) * mix(0.03, 0.3, max(goldenHour, daylight * 0.4));
    col += mix(u_color1, u_color2, 0.3) * sandHorizGlow;
  }

  // --- Pyramids ---
  float p1 = pyramidShape(uv, vec2(0.55, horizon - 0.005), 0.11, 0.16);
  float p2 = pyramidShape(uv, vec2(0.35, horizon - 0.003), 0.09, 0.14);
  float p3 = pyramidShape(uv, vec2(0.20, horizon - 0.002), 0.055, 0.08);

  float pyramidMask = 1.0;
  pyramidMask = min(pyramidMask, smoothstep(-0.003, 0.003, p1));
  pyramidMask = min(pyramidMask, smoothstep(-0.003, 0.003, p2));
  pyramidMask = min(pyramidMask, smoothstep(-0.003, 0.003, p3));

  // Pyramid color: darker at night, subtle edge glow from ambient light
  vec3 pyramidCol = u_color4 * mix(0.06, 0.15, max(daylight, goldenHour * 0.5));

  // Edge highlights based on ambient daylight (no directional sun)
  float lightStr = max(daylight, goldenHour * 0.8);
  float edgeHighlight1 = smoothstep(0.006, 0.0, p1) * smoothstep(-0.006, 0.0, p1);
  float heightFactor1 = smoothstep(horizon, horizon + 0.14, uv.y);
  pyramidCol += u_color1 * edgeHighlight1 * heightFactor1 * 0.25 * lightStr;

  float edgeHighlight2 = smoothstep(0.005, 0.0, p2) * smoothstep(-0.005, 0.0, p2);
  float heightFactor2 = smoothstep(horizon, horizon + 0.12, uv.y);
  pyramidCol += u_color1 * edgeHighlight2 * heightFactor2 * 0.2 * lightStr;

  col = mix(pyramidCol, col, pyramidMask);

  // --- Dust clouds — two smooth FBM layers, no grid artifacts ---
  {
    // Layer 1: slow, broad dust bank
    float speed1 = 0.08;
    float yBase1 = horizon - 0.03;
    float ySpread1 = 0.14;

    vec2 dustUV1 = vec2(
      uv.x * aspect * 2.0 - rawTime * speed1,
      (uv.y - yBase1) * 5.0
    );
    float dust1 = fbm(dustUV1 + vec2(0.0, 3.7));
    dust1 = smoothstep(0.30, 0.62, dust1);

    float yDist1 = (uv.y - yBase1) / ySpread1;
    float vertFade1 = exp(-yDist1 * yDist1 * 2.0);

    float dustAlpha1 = dust1 * vertFade1 * 0.28;

    // Layer 2: faster, thinner wisps blowing through
    float speed2 = 0.22;
    float yBase2 = horizon - 0.06;
    float ySpread2 = 0.10;

    vec2 dustUV2 = vec2(
      uv.x * aspect * 3.5 - rawTime * speed2,
      (uv.y - yBase2) * 8.0 + 17.0
    );
    float dust2 = fbm(dustUV2 + vec2(41.0, 0.0));
    dust2 = smoothstep(0.35, 0.65, dust2);

    float yDist2 = (uv.y - yBase2) / ySpread2;
    float vertFade2 = exp(-yDist2 * yDist2 * 2.5);

    float dustAlpha2 = dust2 * vertFade2 * 0.22;

    // Both layers always glow — stronger at night
    float glowStr = mix(0.7, 1.0, max(nightAmount * 0.6, max(daylight, goldenHour * 0.8)));

    vec3 dustColDay = mix(u_color1, u_color3, 0.3) * 0.6;
    vec3 dustColNight = mix(u_color1, u_color2, 0.45) * 0.5;
    vec3 dustCol = mix(dustColNight, dustColDay, daylight);

    col += dustCol * dustAlpha1 * glowStr;
    col += dustCol * dustAlpha2 * glowStr;
  }

  // --- Dust glow halo around horizon ---
  float dustPulse = 0.7 + 0.3 * sin(rawTime * 0.4) * sin(rawTime * 0.27 + 1.3);
  float dustBand = exp(-pow(abs(uv.y - horizon) / 0.12, 2.0)) * dustPulse;
  float dustBandNoise = fbm(vec2(uv.x * aspect * 3.0 - rawTime * 0.1, rawTime * 0.05 + 7.0));
  dustBand *= 0.6 + 0.4 * dustBandNoise;
  vec3 dustGlowDay = mix(u_color1, u_color2, 0.3) * 0.2;
  vec3 dustGlowNight = mix(u_color1, u_color2, 0.5) * 0.15;
  col += mix(dustGlowNight, dustGlowDay, daylight) * dustBand;

  // --- Atmospheric haze near horizon ---
  float hazeStrength = smoothstep(0.18, 0.0, abs(uv.y - horizon));
  vec3 hazeColDay = mix(u_color1, u_color2, 0.3) * 0.18;
  vec3 hazeColNight = mix(u_color1, u_color2, 0.5) * 0.10;
  col += mix(hazeColNight, hazeColDay, daylight) * hazeStrength;

  // --- Scanlines ---
  float scanline = 0.93 + 0.07 * sin(gl_FragCoord.y * 3.0);
  col *= scanline;

  // Vignette
  vec2 vig = uv - 0.5;
  col *= 1.0 - dot(vig, vig) * 0.6;

  fragColor = vec4(col * 0.5, 1.0);
}
