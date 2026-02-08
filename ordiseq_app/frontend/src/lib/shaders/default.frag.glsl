#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  // Subtle animated gradient
  float t = u_time * 0.1;
  vec3 color1 = vec3(0.157, 0.165, 0.212); // #282a36
  vec3 color2 = vec3(0.267, 0.278, 0.353); // #44475a

  float noise = sin(uv.x * 10.0 + t) * sin(uv.y * 10.0 + t * 0.7) * 0.5 + 0.5;
  float wave = sin(uv.x * 3.0 + uv.y * 2.0 + t) * 0.5 + 0.5;

  vec3 color = mix(color1, color2, wave * 0.3 + noise * 0.1);
  fragColor = vec4(color, 1.0);
}
