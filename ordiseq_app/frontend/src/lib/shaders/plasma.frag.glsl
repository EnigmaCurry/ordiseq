#version 300 es
precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
out vec4 fragColor;

void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  float t = u_time * 0.5;

  float v = 0.0;
  v += sin((uv.x * 10.0 + t));
  v += sin((uv.y * 10.0 + t) / 2.0);
  v += sin((uv.x * 10.0 + uv.y * 10.0 + t) / 2.0);

  vec3 col = vec3(0.157, 0.165, 0.212);
  col += 0.1 * vec3(sin(v), sin(v + 2.094), sin(v + 4.188));

  fragColor = vec4(col, 1.0);
}
