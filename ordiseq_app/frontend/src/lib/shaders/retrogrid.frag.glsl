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
  float horizon = 0.35;

  // Grid cutoff - stop drawing grid here to avoid aliasing
  float gridCutoff = 0.18;

  if (uv.y < gridCutoff) {
    // Ground plane with perspective
    float depth = (horizon - uv.y) / horizon;
    float z = 1.0 / (depth + 0.001);

    // Camera movement
    float speed = 0.5;
    float zOffset = u_time * speed;

    // Perspective-correct X coordinate
    float x = (uv.x - 0.5) * z * 2.5;

    // Crisp grid lines
    float gridSize = 0.5;
    float lineWidth = 0.025;

    // Horizontal lines (Z direction)
    float zGrid = mod(z + zOffset, gridSize);
    float hLine = step(zGrid, lineWidth) + step(gridSize - lineWidth, zGrid);

    // Vertical lines (X direction)
    float xGrid = mod(x + gridSize * 0.5, gridSize);
    float vLine = step(xGrid, lineWidth) + step(gridSize - lineWidth, xGrid);

    // Combine grid lines
    float grid = min(1.0, hLine + vLine);

    fragColor = vec4(pink * grid, 1.0);
  } else if (uv.y < horizon) {
    // Dark zone between grid and horizon - no interference
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
  } else {
    // Sky with horizon line
    float horizonLine = exp(-pow((uv.y - horizon) * 60.0, 2.0));
    fragColor = vec4(pink * horizonLine * 0.9, 1.0);
  }
}
