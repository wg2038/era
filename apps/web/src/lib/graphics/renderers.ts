import { INK_WASH_WGSL } from './shader-wgsl';
import type { GPUCanvasContext } from './webgpu';

export interface RendererInstance {
  setEraYear(year: number): void;
  destroy(): void;
}

/**
 * 1. WebGPU 原生管线渲染器
 */
export async function createWebGPURenderer(
  canvas: HTMLCanvasElement,
  prefersReducedMotion = false,
): Promise<RendererInstance | null> {
  if (!navigator.gpu) return null;

  try {
    const adapter = await navigator.gpu.requestAdapter({
      powerPreference: 'low-power',
    });
    if (!adapter) return null;

    const device = await adapter.requestDevice();
    const context = canvas.getContext('webgpu') as unknown as GPUCanvasContext;
    if (!context) return null;

    const format = navigator.gpu.getPreferredCanvasFormat();
    context.configure({
      device,
      format,
      alphaMode: 'opaque',
    });

    const shaderModule = device.createShaderModule({
      code: INK_WASH_WGSL,
    });

    // Uniform buffer: vec2 resolution (8 bytes) + f32 time (4 bytes) + f32 eraTone (4 bytes) = 16 bytes
    const uniformBuffer = device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    const pipeline = device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: shaderModule,
        entryPoint: 'vs_main',
      },
      fragment: {
        module: shaderModule,
        entryPoint: 'fs_main',
        targets: [{ format }],
      },
      primitive: {
        topology: 'triangle-list',
      },
    });

    const bindGroup = device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{ binding: 0, resource: { buffer: uniformBuffer } }],
    });

    let animationFrameId: number | null = null;
    const startTime = performance.now();
    let currentEraTone = 0.35;
    let isDestroyed = false;

    function render(now: number) {
      if (isDestroyed) return;

      const elapsed = prefersReducedMotion ? 0 : (now - startTime) * 0.001;

      // Update uniforms
      const uniformData = new Float32Array([canvas.width, canvas.height, elapsed, currentEraTone]);
      device.queue.writeBuffer(uniformBuffer, 0, uniformData.buffer);

      const commandEncoder = device.createCommandEncoder();
      const currentTexture = context.getCurrentTexture();
      const textureView = currentTexture.createView();

      const passEncoder = commandEncoder.beginRenderPass({
        colorAttachments: [
          {
            view: textureView,
            clearValue: { r: 0.93, g: 0.9, b: 0.84, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store',
          },
        ],
      });

      passEncoder.setPipeline(pipeline);
      passEncoder.setBindGroup(0, bindGroup);
      passEncoder.draw(6);
      passEncoder.end();

      device.queue.submit([commandEncoder.finish()]);

      if (!prefersReducedMotion) {
        animationFrameId = requestAnimationFrame(render);
      }
    }

    render(performance.now());

    return {
      setEraYear(year: number) {
        // Map 1900-2025 to 0.0 - 1.0
        currentEraTone = Math.max(0, Math.min(1, (year - 1900) / 125));
        if (prefersReducedMotion) {
          render(performance.now());
        }
      },
      destroy() {
        isDestroyed = true;
        if (animationFrameId !== null) {
          cancelAnimationFrame(animationFrameId);
        }
        device.destroy();
      },
    };
  } catch (err) {
    console.warn('WebGPU initialization failed, falling back:', err);
    return null;
  }
}

/**
 * 2. WebGL2 降级着色器渲染器
 */
export function createWebGL2Renderer(
  canvas: HTMLCanvasElement,
  prefersReducedMotion = false,
): RendererInstance | null {
  const gl = canvas.getContext('webgl2');
  if (!gl) return null;

  const vsSource = `#version 300 es
    in vec2 position;
    out vec2 vUv;
    void main() {
      vUv = position * 0.5 + 0.5;
      gl_Position = vec4(position, 0.0, 1.0);
    }
  `;

  const fsSource = `#version 300 es
    precision highp float;
    in vec2 vUv;
    out vec4 fragColor;
    uniform vec2 uResolution;
    uniform float uTime;
    uniform float uEraTone;

    float hash(vec2 p) {
      vec3 p3 = fract(vec3(p.xyx) * 0.1031);
      p3 += dot(p3, p3.yzx + 33.33);
      return fract((p3.x + p3.y) * p3.z);
    }

    float noise(vec2 p) {
      vec2 i = floor(p);
      vec2 f = fract(p);
      vec2 u = f * f * (3.0 - 2.0 * f);
      return mix(
        mix(hash(i + vec2(0.0, 0.0)), hash(i + vec2(1.0, 0.0)), u.x),
        mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
        u.y
      );
    }

    float fbm(vec2 p) {
      float v = 0.0;
      float a = 0.5;
      for (int i = 0; i < 4; i++) {
        v += a * noise(p);
        p = p * 2.0 + vec2(100.0);
        a *= 0.5;
      }
      return v;
    }

    void main() {
      vec2 uv = vUv;
      uv.x *= uResolution.x / uResolution.y;

      float t = uTime * 0.05;
      float fiber = noise(uv * 100.0) * 0.03;
      vec3 paper = vec3(0.93, 0.90, 0.84) - fiber;

      vec2 q = vec2(fbm(uv + vec2(0.0, t)), fbm(uv + vec2(5.2, 1.3 - t)));
      float ink = fbm(uv + 3.0 * q);

      vec3 eraColor = mix(vec3(0.12, 0.10, 0.08), vec3(0.06, 0.12, 0.10), uEraTone);
      float inkFactor = smoothstep(0.28, 0.80, ink) * 0.42;

      fragColor = vec4(mix(paper, eraColor, inkFactor), 1.0);
    }
  `;

  function createShader(gl: WebGL2RenderingContext, type: number, src: string) {
    const s = gl.createShader(type);
    if (!s) return null;
    gl.shaderSource(s, src);
    gl.compileShader(s);
    return s;
  }

  const vs = createShader(gl, gl.VERTEX_SHADER, vsSource);
  const fs = createShader(gl, gl.FRAGMENT_SHADER, fsSource);
  if (!vs || !fs) return null;

  const prog = gl.createProgram();
  if (!prog) return null;
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);

  const posBuf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, posBuf);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
    gl.STATIC_DRAW,
  );

  const posLoc = gl.getAttribLocation(prog, 'position');
  const uResLoc = gl.getUniformLocation(prog, 'uResolution');
  const uTimeLoc = gl.getUniformLocation(prog, 'uTime');
  const uEraLoc = gl.getUniformLocation(prog, 'uEraTone');

  let animationFrameId: number | null = null;
  const startTime = performance.now();
  let currentEraTone = 0.35;
  let isDestroyed = false;

  function render(now: number) {
    if (isDestroyed || !gl) return;
    const elapsed = prefersReducedMotion ? 0 : (now - startTime) * 0.001;

    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.useProgram(prog);

    gl.enableVertexAttribArray(posLoc);
    gl.bindBuffer(gl.ARRAY_BUFFER, posBuf);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

    gl.uniform2f(uResLoc, canvas.width, canvas.height);
    gl.uniform1f(uTimeLoc, elapsed);
    gl.uniform1f(uEraLoc, currentEraTone);

    gl.drawArrays(gl.TRIANGLES, 0, 6);

    if (!prefersReducedMotion) {
      animationFrameId = requestAnimationFrame(render);
    }
  }

  render(performance.now());

  return {
    setEraYear(year: number) {
      currentEraTone = Math.max(0, Math.min(1, (year - 1900) / 125));
      if (prefersReducedMotion) render(performance.now());
    },
    destroy() {
      isDestroyed = true;
      if (animationFrameId !== null) cancelAnimationFrame(animationFrameId);
      gl.deleteProgram(prog);
      gl.deleteShader(vs);
      gl.deleteShader(fs);
      gl.deleteBuffer(posBuf);
    },
  };
}

/**
 * 3. Canvas2D 极简保底渲染器（无 GPU / 无 WebGL 环境）
 */
export function createCanvas2DRenderer(
  canvas: HTMLCanvasElement,
  prefersReducedMotion = false,
): RendererInstance {
  const ctx = canvas.getContext('2d');
  let animationFrameId: number | null = null;
  let currentEraTone = 0.35;
  let isDestroyed = false;
  const startTime = performance.now();

  function draw(now: number) {
    if (isDestroyed || !ctx) return;
    const width = canvas.width;
    const height = canvas.height;
    const t = prefersReducedMotion ? 0 : (now - startTime) * 0.0005;

    // 宣纸底色
    ctx.fillStyle = '#f2ece0';
    ctx.fillRect(0, 0, width, height);

    // 水墨扩散光晕圆斑
    const grad = ctx.createRadialGradient(
      width * (0.5 + 0.1 * Math.sin(t)),
      height * (0.5 + 0.1 * Math.cos(t)),
      width * 0.05,
      width * 0.5,
      height * 0.5,
      width * 0.65,
    );

    const inkAlpha = 0.07 + 0.03 * Math.sin(t * 2);
    const r = Math.round(32 * (1 - currentEraTone) + 18 * currentEraTone);
    const g = Math.round(28 * (1 - currentEraTone) + 30 * currentEraTone);
    const b = Math.round(24 * (1 - currentEraTone) + 26 * currentEraTone);
    grad.addColorStop(0, `rgba(${r}, ${g}, ${b}, ${inkAlpha * 1.5})`);
    grad.addColorStop(0.5, `rgba(${r + 10}, ${g + 10}, ${b + 10}, ${inkAlpha})`);
    grad.addColorStop(1, 'rgba(242, 236, 224, 0)');

    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, width, height);

    if (!prefersReducedMotion) {
      animationFrameId = requestAnimationFrame(draw);
    }
  }

  draw(performance.now());

  return {
    setEraYear(year: number) {
      currentEraTone = Math.max(0, Math.min(1, (year - 1900) / 125));
      if (prefersReducedMotion) draw(performance.now());
    },
    destroy() {
      isDestroyed = true;
      if (animationFrameId !== null) cancelAnimationFrame(animationFrameId);
    },
  };
}
