#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform float u_pitch;      // 0.1 to 0.9 - horizon position (camera pitch)
uniform float u_direction;  // -1.0 to 1.0 - direction of travel
uniform float u_speed;      // 0.0 to 2.0 - movement speed
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Hot pink color
  vec3 pink = vec3(1.0, 0.08, 0.58);

  // Horizon position from pitch
  float horizon = clamp(u_pitch, 0.1, 0.9);

  // Shift vanishing point based on direction (camera points where it travels)
  float vanishX = 0.5 - u_direction * 0.3;

  if (uv.y < horizon) {
    // Ground plane with perspective - vanishing point shifts with direction
    float depth = (horizon - uv.y) / horizon;
    float z = 1.0 / (depth + 0.0001);

    // Camera movement - travel in the direction we're pointing
    float timeOffset = u_time * u_speed;
    float zOffset = timeOffset;
    float xOffset = timeOffset * u_direction * 2.0;

    // Perspective-correct X coordinate relative to shifted vanishing point
    float x = (uv.x - vanishX) * z * 2.5 + xOffset;

    // Grid with constant line width
    float gridSize = 0.5;
    float lineWidth = 0.02;

    // Use screen-space derivatives for anti-aliasing
    float zVal = z + zOffset;
    float zGrid = mod(zVal, gridSize);
    float zAA = fwidth(zVal) * 1.5;
    float hLine = smoothstep(lineWidth + zAA, lineWidth, zGrid) +
                  smoothstep(gridSize - lineWidth - zAA, gridSize - lineWidth, zGrid);

    float xGrid = mod(x + gridSize * 0.5, gridSize);
    float xAA = fwidth(x) * 1.5;
    float vLine = smoothstep(lineWidth + xAA, lineWidth, xGrid) +
                  smoothstep(gridSize - lineWidth - xAA, gridSize - lineWidth, xGrid);

    // Combine grid lines
    float grid = min(1.0, hLine + vLine);

    // Fade to black approaching horizon (stronger fade to limit convergence brightness)
    float fade = 1.0 - smoothstep(5.0, 20.0, z);
    grid *= fade * fade;  // Square for more aggressive fade near horizon

    fragColor = vec4(pink * grid, 1.0);
  } else {
    // Sky - pure black, no glow
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
  }
}
