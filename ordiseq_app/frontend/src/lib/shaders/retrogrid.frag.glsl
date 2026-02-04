#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Pure black sky
  vec3 skyColor = vec3(0.0);

  // Horizon line position
  float horizon = 0.35;

  if (uv.y < horizon) {
    // Ground plane with perspective
    float depth = (horizon - uv.y) / horizon;
    float z = 1.0 / (depth + 0.001);

    // Camera movement (slowly tracking forward)
    float speed = 0.5;
    float zOffset = u_time * speed;

    // Perspective-correct X coordinate
    float x = (uv.x - 0.5) * z * 2.5;

    // Crisp grid lines
    float gridSize = 0.5;
    float lineWidth = 0.02;

    // Horizontal lines (Z direction) - crisp edges
    float zGrid = mod(z + zOffset, gridSize);
    float hLine = step(zGrid, lineWidth) + step(gridSize - lineWidth, zGrid);

    // Vertical lines (X direction) - crisp edges
    float xGrid = mod(x + gridSize * 0.5, gridSize);
    float vLine = step(xGrid, lineWidth) + step(gridSize - lineWidth, xGrid);

    // Combine grid lines
    float grid = min(1.0, hLine + vLine);

    // Sharp cutoff at distance (no fog, just stop rendering far lines)
    float maxDist = 20.0;
    grid *= step(z, maxDist);

    // Suppress grid near horizon to avoid aliasing (sharp cutoff)
    float horizonCutoff = smoothstep(horizon, horizon - 0.03, uv.y);
    grid *= horizonCutoff;

    // Hot pink color - bright and crisp
    vec3 gridColor = vec3(1.0, 0.08, 0.58);

    // Pure black ground
    vec3 groundColor = vec3(0.0);
    vec3 color = groundColor + gridColor * grid;

    fragColor = vec4(color, 1.0);
  } else {
    // Sky - pure black with thin horizon line
    vec3 color = skyColor;

    // Thin bright horizon line
    float horizonLine = exp(-pow((uv.y - horizon) * 50.0, 2.0));
    color += vec3(1.0, 0.08, 0.58) * horizonLine * 0.8;

    fragColor = vec4(color, 1.0);
  }
}
