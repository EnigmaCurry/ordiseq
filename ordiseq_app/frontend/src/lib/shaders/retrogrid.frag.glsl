#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform float u_pitch;  // 0.1 to 0.9 - camera pitch angle
uniform float u_speed;  // 0.0 to 2.0 - movement speed
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  vec2 centered = uv - 0.5;

  // Aspect ratio correction
  float aspect = u_resolution.x / u_resolution.y;
  centered.x *= aspect;

  // Hot pink color
  vec3 pink = vec3(1.0, 0.08, 0.58);

  // Fisheye projection - map screen to sphere
  float r = length(centered);
  float maxR = 0.8;

  if (r < maxR) {
    // Fisheye distortion - bend the view onto a sphere
    float phi = r * 3.14159 * 0.8;  // Angular distance from center
    float theta = atan(centered.y, centered.x);  // Angle around center

    // Spherical to cartesian for ray direction
    vec3 rayDir;
    rayDir.x = sin(phi) * cos(theta);
    rayDir.y = cos(phi);
    rayDir.z = sin(phi) * sin(theta);

    // Tilt camera down based on pitch
    float pitchAngle = (1.0 - u_pitch) * 1.5;
    float cy = cos(pitchAngle);
    float sy = sin(pitchAngle);
    vec3 tilted = vec3(
      rayDir.x,
      rayDir.y * cy - rayDir.z * sy,
      rayDir.y * sy + rayDir.z * cy
    );

    // Ray-plane intersection for ground (y = -1)
    if (tilted.y < -0.001) {
      float t = -1.0 / tilted.y;
      float x = tilted.x * t;
      float z = tilted.z * t;

      // Camera movement
      z += u_time * u_speed * 5.0;

      // Grid
      float gridSize = 1.0;
      float lineWidth = 0.03;

      // Anti-aliased grid lines
      float xGrid = mod(x, gridSize);
      float zGrid = mod(z, gridSize);
      float xAA = fwidth(x) * 1.5;
      float zAA = fwidth(z) * 1.5;

      float hLine = smoothstep(lineWidth + zAA, lineWidth, zGrid) +
                    smoothstep(gridSize - lineWidth - zAA, gridSize - lineWidth, zGrid);
      float vLine = smoothstep(lineWidth + xAA, lineWidth, xGrid) +
                    smoothstep(gridSize - lineWidth - xAA, gridSize - lineWidth, xGrid);

      float grid = min(1.0, hLine + vLine);

      // Distance fade
      float dist = length(vec2(x, z - u_time * u_speed * 5.0));
      float fade = 1.0 - smoothstep(5.0, 30.0, dist);
      grid *= fade;

      fragColor = vec4(pink * grid, 1.0);
    } else {
      // Sky
      fragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
  } else {
    // Outside fisheye - black
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
  }
}
