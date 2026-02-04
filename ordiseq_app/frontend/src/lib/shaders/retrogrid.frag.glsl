#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform float u_pitch;    // 0.1 to 0.9 - camera pitch angle
uniform float u_speed;    // 0.0 to 2.0 - movement speed
uniform float u_zoom;     // 0.5 to 2.0 - zoom level (affects central area size)
uniform float u_fisheye;  // 0.0 to 1.0 - blend between perspective (0) and fisheye (1)
uniform vec3 u_color1;    // Grid color 1
uniform vec3 u_color2;    // Grid color 2
uniform vec3 u_color3;    // Grid color 3
uniform vec3 u_color4;    // Grid color 4
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  vec2 centered = uv - 0.5;

  // Aspect ratio correction
  float aspect = u_resolution.x / u_resolution.y;
  centered.x *= aspect;

  // Apply zoom - smaller zoom = larger central area
  centered *= u_zoom;

  // Fisheye projection - map screen to sphere
  float r = length(centered);
  float maxR = 0.8 * u_zoom;

  if (r < maxR) {
    // Fisheye distortion - bend the view onto a sphere
    float phi_fisheye = r * 3.14159 * 0.8;  // Angular distance from center (fisheye)
    float phi_persp = atan(r, 1.0);          // Angular distance (perspective)
    float phi = mix(phi_persp, phi_fisheye, u_fisheye);  // Blend between them
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

      // Determine which cell we're in for color selection
      float cellX = floor(x);
      float cellZ = floor(z);
      int colorIdx = int(mod(cellX + cellZ, 4.0));

      // Select color based on cell position
      vec3 lineColor;
      if (colorIdx == 0) lineColor = u_color1;
      else if (colorIdx == 1) lineColor = u_color2;
      else if (colorIdx == 2) lineColor = u_color3;
      else lineColor = u_color4;

      float grid = min(1.0, hLine + vLine);

      // Distance fade
      float dist = length(vec2(x, z - u_time * u_speed * 5.0));
      float fade = 1.0 - smoothstep(5.0, 30.0, dist);
      grid *= fade;

      fragColor = vec4(lineColor * grid, 1.0);
    } else {
      // Sky
      fragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
  } else {
    // Outside fisheye - black
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);
  }
}
