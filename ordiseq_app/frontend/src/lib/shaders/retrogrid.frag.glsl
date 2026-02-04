#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Hot pink color
  vec3 pink = vec3(1.0, 0.08, 0.58);

  // Horizon position
  float horizon = 0.4;

  // Grid cutoff - small buffer before horizon
  float gridCutoff = 0.36;

  if (uv.y < gridCutoff) {
    // Ground plane with perspective
    float depth = (horizon - uv.y) / horizon;
    float z = 1.0 / (depth + 0.001);

    // Camera movement
    float speed = 0.5;
    float zOffset = u_time * speed;

    // Perspective-correct X coordinate
    float x = (uv.x - 0.5) * z * 2.5;

    // Grid - line width increases with distance to stay visible and avoid aliasing
    float gridSize = 0.5;
    float lineWidth = 0.02 + z * 0.015;

    // Horizontal lines (Z direction)
    float zGrid = mod(z + zOffset, gridSize);
    float hLine = step(zGrid, lineWidth) + step(gridSize - lineWidth, zGrid);

    // Vertical lines (X direction)
    float xGrid = mod(x + gridSize * 0.5, gridSize);
    float vLine = step(xGrid, lineWidth) + step(gridSize - lineWidth, xGrid);

    // Combine grid lines
    float grid = min(1.0, hLine + vLine);

    // Fade intensity slightly with distance (not fog, just dimming)
    float intensity = 1.0 - smoothstep(5.0, 25.0, z) * 0.6;

    fragColor = vec4(pink * grid * intensity, 1.0);
  } else if (uv.y < horizon) {
    // Thin dark buffer
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
  } else {
    // Sky with horizon line
    float horizonLine = exp(-pow((uv.y - horizon) * 60.0, 2.0));
    fragColor = vec4(pink * horizonLine * 0.9, 1.0);
  }
}
