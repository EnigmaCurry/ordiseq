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

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float aspect = u_resolution.x / u_resolution.y;
  float tempo = u_bpm / 120.0;
  float t = mix(u_time * 0.3 * tempo, u_beat * 0.3, u_playing);

  // Horizon line at 40% from bottom
  float horizon = 0.4;
  vec3 col = vec3(0.0);

  // --- Sky gradient ---
  if (uv.y > horizon) {
    float skyT = (uv.y - horizon) / (1.0 - horizon);
    // Dark purple at top, warm orange-pink at horizon
    vec3 skyTop = u_color3 * 0.1;
    vec3 skyBot = mix(u_color1, u_color2, 0.3) * 0.6;
    col = mix(skyBot, skyTop, skyT);

    // --- Sun ---
    vec2 sunCenter = vec2(0.5, horizon + 0.22);
    vec2 sunUV = vec2((uv.x - sunCenter.x) * aspect, uv.y - sunCenter.y);
    float sunDist = length(sunUV);
    float sunRadius = 0.15;

    if (sunDist < sunRadius) {
      // Sun body with horizontal stripe cutouts
      float stripeFreq = 40.0;
      float stripe = step(0.5, fract(uv.y * stripeFreq));
      // Bottom of sun has more gaps (classic synthwave look)
      float gapGrow = smoothstep(sunCenter.y, sunCenter.y - sunRadius, uv.y);
      float stripeWidth = mix(0.0, 0.7, gapGrow);
      float sunMask = 1.0 - step(1.0 - stripeWidth, stripe) * step(0.3, gapGrow);

      vec3 sunTop = u_color2;
      vec3 sunBot = u_color1;
      float sunGrad = (uv.y - (sunCenter.y - sunRadius)) / (2.0 * sunRadius);
      vec3 sunCol = mix(sunBot, sunTop, clamp(sunGrad, 0.0, 1.0));
      col = mix(col, sunCol * 1.5, sunMask * smoothstep(sunRadius, sunRadius - 0.005, sunDist));
    }

    // Sun glow
    float glow = exp(-sunDist * 4.0) * 0.4;
    col += mix(u_color1, u_color2, 0.5) * glow;

    // Stars (above horizon)
    if (skyT > 0.3) {
      vec2 starUV = floor(uv * vec2(80.0 * aspect, 80.0));
      float starHash = fract(sin(dot(starUV, vec2(127.1, 311.7))) * 43758.5453);
      float twinkle = sin(t * 3.0 + starHash * 6.28) * 0.5 + 0.5;
      if (starHash > 0.985) {
        col += vec3(0.6 + 0.4 * twinkle) * (skyT - 0.3) * 2.0;
      }
    }
  }

  // --- Ground: perspective grid ---
  if (uv.y <= horizon) {
    float groundT = (horizon - uv.y) / horizon;
    // Perspective Z from screen Y
    float z = 1.0 / (groundT + 0.01);
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
    if (ci == 0) gridCol = u_color1;
    else if (ci == 1) gridCol = u_color2;
    else if (ci == 2) gridCol = u_color3;
    else gridCol = u_color4;

    // Dark ground base with grid overlay
    vec3 groundBase = u_color3 * 0.02;
    col = groundBase + gridCol * grid;

    // Horizon glow on ground
    float horizGlow = exp(-groundT * 6.0) * 0.3;
    col += u_color1 * horizGlow;
  }

  // --- Scanlines ---
  float scanline = 0.92 + 0.08 * sin(gl_FragCoord.y * 3.0);
  col *= scanline;

  // Slight vignette
  vec2 vig = uv - 0.5;
  col *= 1.0 - dot(vig, vig) * 0.5;

  fragColor = vec4(col * 0.5, 1.0);
}
