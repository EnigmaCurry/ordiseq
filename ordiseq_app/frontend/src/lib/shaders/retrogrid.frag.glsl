#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Sky gradient (dark purple to black)
  vec3 skyColor = mix(
    vec3(0.02, 0.0, 0.05),
    vec3(0.1, 0.0, 0.15),
    uv.y
  );

  // Horizon line position
  float horizon = 0.4;

  if (uv.y < horizon) {
    // Ground plane with perspective
    float depth = (horizon - uv.y) / horizon;
    float z = 1.0 / (depth + 0.01);

    // Camera movement (slowly tracking forward)
    float speed = 0.5;
    float zOffset = u_time * speed;

    // Perspective-correct X coordinate
    float x = (uv.x - 0.5) * z * 2.0;

    // Grid lines
    float gridSize = 0.5;
    float lineWidth = 0.03;

    // Horizontal lines (Z direction)
    float zGrid = mod(z + zOffset, gridSize);
    float hLine = smoothstep(lineWidth, 0.0, zGrid) + smoothstep(gridSize - lineWidth, gridSize, zGrid);

    // Vertical lines (X direction)
    float xGrid = mod(x + gridSize * 0.5, gridSize);
    float vLine = smoothstep(lineWidth, 0.0, xGrid) + smoothstep(gridSize - lineWidth, gridSize, xGrid);

    // Combine grid lines
    float grid = max(hLine, vLine);

    // Distance fade
    float fade = 1.0 - smoothstep(0.0, 15.0, z);
    grid *= fade;

    // Hot pink color: #ff1493 (deep pink) to #ff69b4 (hot pink)
    vec3 gridColor = vec3(1.0, 0.08, 0.58);

    // Add glow effect
    float glow = grid * 0.5;
    vec3 glowColor = vec3(1.0, 0.4, 0.7) * glow;

    // Dark ground with grid
    vec3 groundColor = vec3(0.02, 0.0, 0.03);
    vec3 color = groundColor + gridColor * grid * 0.8 + glowColor;

    fragColor = vec4(color, 1.0);
  } else {
    // Sky with subtle gradient
    vec3 color = skyColor;

    // Horizon glow
    float horizonGlow = exp(-pow((uv.y - horizon) * 8.0, 2.0));
    color += vec3(1.0, 0.2, 0.5) * horizonGlow * 0.3;

    fragColor = vec4(color, 1.0);
  }
}
