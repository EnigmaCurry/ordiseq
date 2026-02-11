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
  vec2 uv = (gl_FragCoord.xy - 0.5 * u_resolution) / min(u_resolution.x, u_resolution.y);
  float tempo = u_bpm / 120.0;
  float t = mix(u_time * 0.8 * tempo, u_beat * 0.8, u_playing);

  // Wormhole winding: offset the tunnel center in a sinuous path through space
  // Each "depth layer" has its own offset, creating a curving worm shape
  float radius = length(uv);
  float z = 0.8 / (radius + 0.001);
  float zWorld = z + t * 3.0;

  // Sinuous path: the tunnel center snakes through space
  // Multiple sine waves at different frequencies for organic winding
  float windX = sin(zWorld * 0.15) * 0.4
              + sin(zWorld * 0.08 + 1.7) * 0.25
              + sin(zWorld * 0.23 + 3.1) * 0.1;
  float windY = cos(zWorld * 0.12 + 0.5) * 0.35
              + cos(zWorld * 0.07 + 2.3) * 0.2
              + sin(zWorld * 0.19 + 4.0) * 0.1;

  // Offset UV by the winding path, scaled by depth (closer = more offset)
  vec2 tunnelUV = uv - vec2(windX, windY) * radius;
  float tunnelR = length(tunnelUV);
  float angle = atan(tunnelUV.y, tunnelUV.x);

  // Recompute z from the curved tunnel radius
  float zCurved = 0.8 / (tunnelR + 0.001);
  float zFinal = zCurved + t * 3.0;

  // Number of sides (octagonal wireframe)
  float sides = 8.0;
  float sectorAngle = 3.14159 * 2.0 / sides;

  // Twist the tunnel as it winds (rotate the octagon along depth)
  float twist = zFinal * 0.3;
  float twistedAngle = angle + twist;

  float sector = mod(twistedAngle + 3.14159, sectorAngle);
  float sectorEdge = abs(sector - sectorAngle * 0.5);

  // Wireframe rings (depth lines)
  float ringFreq = 2.0;
  float rings = abs(fract(zFinal * ringFreq) - 0.5);
  float ringLine = smoothstep(0.02, 0.0, rings);

  // Wireframe longitudinal lines (side edges)
  float edgeWidth = 0.06 * tunnelR;
  float sideLine = smoothstep(edgeWidth + 0.01, edgeWidth, sectorEdge);

  // Combined wireframe
  float wire = max(ringLine, sideLine);

  // Color based on ring depth
  float ringIdx = mod(floor(zFinal * ringFreq), 4.0);
  vec3 wireCol;
  if (ringIdx < 1.0) wireCol = u_color1;
  else if (ringIdx < 2.0) wireCol = u_color2;
  else if (ringIdx < 3.0) wireCol = u_color3;
  else wireCol = u_color4;

  // Side lines use alternating color
  float sideIdx = mod(floor((twistedAngle + 3.14159) / sectorAngle), 4.0);
  vec3 sideCol;
  if (sideIdx < 1.0) sideCol = u_color1;
  else if (sideIdx < 2.0) sideCol = u_color2;
  else if (sideIdx < 3.0) sideCol = u_color3;
  else sideCol = u_color4;

  vec3 col = mix(sideCol, wireCol, step(ringLine, sideLine)) * wire;

  // Neon glow effect
  float glowRing = exp(-rings * 40.0) * 0.3;
  float glowSide = exp(-sectorEdge / max(edgeWidth, 0.001) * 2.0) * 0.2 * step(0.01, tunnelR);
  col += wireCol * glowRing + sideCol * glowSide;

  // Depth fade: bright near camera, fade into the wormhole
  float depthFade = smoothstep(0.0, 0.5, tunnelR);
  col *= depthFade;

  // Wormhole center glow - shifts color as you travel
  float centerGlow = exp(-tunnelR * 8.0) * 0.35;
  vec3 portalColor = mix(u_color1, u_color3, sin(t * 0.7) * 0.5 + 0.5);
  portalColor = mix(portalColor, u_color2, sin(t * 0.4 + 2.0) * 0.5 + 0.5);
  col += portalColor * centerGlow;

  // Starfield behind the wireframe (visible through gaps)
  if (wire < 0.1) {
    vec2 starUV = tunnelUV * 5.0 + vec2(windX, windY) * 0.5;
    vec2 starCell = floor(starUV * 30.0);
    float starHash = fract(sin(dot(starCell, vec2(127.1, 311.7))) * 43758.5453);
    if (starHash > 0.97) {
      float twinkle = sin(t * 4.0 + starHash * 6.28) * 0.5 + 0.5;
      float starBright = (0.4 + 0.6 * twinkle) * depthFade * 0.3;
      col += vec3(starBright);
    }
  }

  // Beat pulse
  float beatPulse = exp(-fract(mix(t * tempo, u_beat, u_playing)) * 4.0) * 0.15;
  col += wireCol * beatPulse * wire;

  // Scanlines
  float scanline = 0.9 + 0.1 * sin(gl_FragCoord.y * 3.0);
  col *= scanline;

  // Vignette
  float vig = 1.0 - dot(uv, uv) * 0.3;
  col *= vig;

  fragColor = vec4(col * 0.5, 1.0);
}
