<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { shaderConfig, type ShaderConfig } from "./shaderStore";

  let canvas: HTMLCanvasElement;
  let gl: WebGL2RenderingContext | null = null;
  let program: WebGLProgram | null = null;
  let animationId: number;
  let startTime: number;
  let uniformLocations: Map<string, WebGLUniformLocation | null> = new Map();
  let currentConfig: ShaderConfig | null = null;

  const vertexShaderSource = `#version 300 es
    in vec4 a_position;
    void main() {
      gl_Position = a_position;
    }
  `;

  function createShader(
    gl: WebGL2RenderingContext,
    type: number,
    source: string
  ): WebGLShader | null {
    const shader = gl.createShader(type);
    if (!shader) return null;

    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error("Shader compile error:", gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }

  function createProgram(
    gl: WebGL2RenderingContext,
    vertexShader: WebGLShader,
    fragmentShader: WebGLShader
  ): WebGLProgram | null {
    const program = gl.createProgram();
    if (!program) return null;

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error("Program link error:", gl.getProgramInfoLog(program));
      gl.deleteProgram(program);
      return null;
    }

    return program;
  }

  function setupProgram(shaderSource: string): boolean {
    if (!gl) return false;

    // Clean up old program
    if (program) {
      gl.deleteProgram(program);
      program = null;
    }

    const vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
    const fragShader = createShader(gl, gl.FRAGMENT_SHADER, shaderSource);

    if (!vertexShader || !fragShader) return false;

    program = createProgram(gl, vertexShader, fragShader);
    if (!program) return false;

    // Set up fullscreen quad
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      gl.STATIC_DRAW
    );

    const positionLocation = gl.getAttribLocation(program, "a_position");
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);

    // Cache uniform locations
    uniformLocations.clear();
    uniformLocations.set("u_time", gl.getUniformLocation(program, "u_time"));
    uniformLocations.set(
      "u_resolution",
      gl.getUniformLocation(program, "u_resolution")
    );

    return true;
  }

  function updateUniformLocations(uniforms: Record<string, number | number[]>) {
    if (!gl || !program) return;

    for (const name of Object.keys(uniforms)) {
      if (!uniformLocations.has(name)) {
        uniformLocations.set(name, gl.getUniformLocation(program, name));
      }
    }
  }

  function resize() {
    if (!canvas || !gl) return;

    const dpr = window.devicePixelRatio || 1;
    const width = canvas.clientWidth * dpr;
    const height = canvas.clientHeight * dpr;

    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
      gl.viewport(0, 0, width, height);
    }
  }

  function render(timestamp: number) {
    if (!gl || !program || !currentConfig) {
      animationId = requestAnimationFrame(render);
      return;
    }

    resize();

    const time = (timestamp - startTime) / 1000;

    gl.useProgram(program);

    // Set built-in uniforms
    const timeLocation = uniformLocations.get("u_time");
    if (timeLocation) {
      gl.uniform1f(timeLocation, time);
    }

    const resolutionLocation = uniformLocations.get("u_resolution");
    if (resolutionLocation) {
      gl.uniform2f(resolutionLocation, canvas.width, canvas.height);
    }

    // Set custom uniforms
    for (const [name, value] of Object.entries(currentConfig.uniforms)) {
      const location = uniformLocations.get(name);
      if (location) {
        if (Array.isArray(value)) {
          switch (value.length) {
            case 2:
              gl.uniform2fv(location, value);
              break;
            case 3:
              gl.uniform3fv(location, value);
              break;
            case 4:
              gl.uniform4fv(location, value);
              break;
          }
        } else {
          gl.uniform1f(location, value);
        }
      }
    }

    gl.drawArrays(gl.TRIANGLES, 0, 6);

    animationId = requestAnimationFrame(render);
  }

  onMount(() => {
    gl = canvas.getContext("webgl2");
    if (!gl) {
      console.error("WebGL2 not supported");
      return;
    }

    startTime = performance.now();

    // Subscribe to shader config changes
    const unsubscribe = shaderConfig.subscribe((config) => {
      currentConfig = config;

      if (gl) {
        if (setupProgram(config.fragmentShader)) {
          updateUniformLocations(config.uniforms);
        }
      }
    });

    animationId = requestAnimationFrame(render);
    window.addEventListener("resize", resize);

    return () => {
      unsubscribe();
    };
  });

  onDestroy(() => {
    cancelAnimationFrame(animationId);
    window.removeEventListener("resize", resize);
    if (gl && program) {
      gl.deleteProgram(program);
    }
  });
</script>

<canvas bind:this={canvas} class="shader-background"></canvas>

<style>
  .shader-background {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: -2;
  }
</style>
