<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import {
    createCanvas2DRenderer,
    createWebGL2Renderer,
    createWebGPURenderer,
    type RendererInstance,
  } from '../graphics/renderers';

  interface Props {
    year?: number;
  }

  let { year = 1930 }: Props = $props();

  let canvasEl: HTMLCanvasElement | null = $state(null);
  let renderer: RendererInstance | null = null;
  let backendName: 'WebGPU' | 'WebGL2' | 'Canvas2D' = $state('WebGPU');

  function onResize() {
    if (!canvasEl) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvasEl.width = Math.floor(window.innerWidth * dpr);
    canvasEl.height = Math.floor(window.innerHeight * dpr);
    renderer?.setEraYear(year);
  }

  onMount(async () => {
    if (!canvasEl) return;

    // 响应式画布分辨率缩放 (DevicePixelRatio)
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvasEl.width = Math.floor(window.innerWidth * dpr);
    canvasEl.height = Math.floor(window.innerHeight * dpr);

    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    // 1. 尝试 WebGPU 原生管线
    let activeRenderer = await createWebGPURenderer(canvasEl, prefersReducedMotion);
    if (activeRenderer) {
      backendName = 'WebGPU';
    } else {
      // 2. 尝试 WebGL2 降级
      activeRenderer = createWebGL2Renderer(canvasEl, prefersReducedMotion);
      if (activeRenderer) {
        backendName = 'WebGL2';
      } else {
        // 3. Canvas2D 保底
        activeRenderer = createCanvas2DRenderer(canvasEl, prefersReducedMotion);
        backendName = 'Canvas2D';
      }
    }

    renderer = activeRenderer;
    renderer.setEraYear(year);

    window.addEventListener('resize', onResize);
  });

  $effect(() => {
    renderer?.setEraYear(year);
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('resize', onResize);
    }
    renderer?.destroy();
  });
</script>

<canvas bind:this={canvasEl} class="ink-canvas" aria-hidden="true"></canvas>

<div class="render-badge">
  <span class="badge-dot" class:webgpu={backendName === 'WebGPU'} class:fallback={backendName !== 'WebGPU'}></span>
  <span>图形层: {backendName} 宣纸水墨扩散渲染</span>
</div>

<style>
  .ink-canvas {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: -1;
    pointer-events: none;
    opacity: 0.85;
    mix-blend-mode: multiply;
  }
  .render-badge {
    position: fixed;
    bottom: 0.75rem;
    right: 0.75rem;
    font-size: 0.72rem;
    color: #8c8477;
    background: rgba(22, 20, 18, 0.85);
    border: 1px solid rgba(80, 70, 55, 0.4);
    backdrop-filter: blur(6px);
    border-radius: 9999px;
    padding: 0.25rem 0.65rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    pointer-events: none;
    z-index: 50;
  }
  .badge-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .badge-dot.webgpu {
    background: #4ade80;
    box-shadow: 0 0 6px rgba(74, 222, 128, 0.6);
  }
  .badge-dot.fallback {
    background: #facc15;
    box-shadow: 0 0 6px rgba(250, 204, 21, 0.6);
  }
</style>
