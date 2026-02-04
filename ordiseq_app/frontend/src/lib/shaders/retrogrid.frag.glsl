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

  if (uv.y < horizon) {
    // Ground plane with perspective - vanishing point at horizon
    float depth = (horizon - uv.y) / horizon;
    float z = 1.0 / (depth + 0.0001);

    // Camera movement
    float speed = 0.5;
    float zOffset = u_time * speed;

    // Perspective-correct X coordinate
    float x = (uv.x - 0.5) * z * 2.5;

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

    // Fade to black approaching horizon
    float fade = 1.0 - smoothstep(10.0, 50.0, z);
    grid *= fade;

    fragColor = vec4(pink * grid, 1.0);
  } else {
    // Sky with horizon glow
    float horizonGlow = exp(-pow((uv.y - horizon) * 30.0, 2.0));
    fragColor = vec4(pink * horizonGlow * 0.5, 1.0);
  }
}
