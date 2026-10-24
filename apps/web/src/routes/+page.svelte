<script lang="ts">
  import { onMount } from 'svelte';
  import initCore, { kernel_version } from '@era/era-core';
  import { compiledData } from '@era/data/dist/index.js';

  let kernelReady = $state(false);
  let kernelVer = $state('');
  let dataVersion = $state(compiledData.version);
  let locationCount = $state(compiledData.locations.length);
  let eventCount = $state(compiledData.events.length);
  let lifeTableCount = $state(compiledData.lifeTables.length);

  onMount(async () => {
    try {
      await initCore();
      kernelVer = kernel_version();
      kernelReady = true;
    } catch (err) {
      console.error('Failed to initialize WASM kernel:', err);
    }
  });
</script>

<div class="stage-container">
  <header class="header">
    <div class="badge">技术展示原型 · 15+ 适龄提示</div>
    <h1 class="title">《Era》（时代）</h1>
    <p class="subtitle">中国百年人生模拟 · 确定性 WebAssembly 内核</p>
  </header>

  <section class="card">
    <h2 class="card-title">M0 架构与集成状态看板</h2>
    <div class="grid">
      <div class="stat-item">
        <span class="stat-label">WASM 模拟内核状态</span>
        <span class="stat-value" class:text-success={kernelReady} class:text-warning={!kernelReady}>
          {kernelReady ? `就绪 (v${kernelVer})` : '加载中...'}
        </span>
      </div>

      <div class="stat-item">
        <span class="stat-label">数据管道校验版本</span>
        <span class="stat-value text-accent">v{dataVersion} (Valibot 严格校验)</span>
      </div>

      <div class="stat-item">
        <span class="stat-label">历史行政区变迁库</span>
        <span class="stat-value">{locationCount} 个地点 (含历史名称沿革)</span>
      </div>

      <div class="stat-item">
        <span class="stat-label">历史事件种子卡片</span>
        <span class="stat-value">{eventCount} 张卡片 (三层事件系统)</span>
      </div>

      <div class="stat-item">
        <span class="stat-label">历史生命表基准年代</span>
        <span class="stat-value">{lifeTableCount} 个年代 (1900–1990)</span>
      </div>
    </div>
  </section>

  <section class="card mt-6">
    <h3 class="card-subtitle">地点历史沿革预览 (以北京为例)</h3>
    <ul class="timeline-list">
      {#each compiledData.locations[0]?.historicalNames ?? [] as item}
        <li>
          <span class="timeline-year">{item.startYear}–{item.endYear}</span>
          <span class="timeline-name">{item.name}</span>
          <span class="timeline-note">{item.note}</span>
        </li>
      {/each}
    </ul>
  </section>
</div>

<style>
  .stage-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 3rem 1.5rem;
  }
  .header {
    text-align: center;
    margin-bottom: 2.5rem;
  }
  .badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
    background-color: #2b2620;
    color: #c79a5b;
    border: 1px solid #4a3e30;
    border-radius: 9999px;
    margin-bottom: 1rem;
    letter-spacing: 0.05em;
  }
  .title {
    font-family: var(--font-serif);
    font-size: 2.5rem;
    letter-spacing: 0.1em;
    color: #f3efe6;
    margin: 0 0 0.5rem 0;
  }
  .subtitle {
    font-size: 1rem;
    color: #9c9488;
    margin: 0;
  }
  .card {
    background: #1a1816;
    border: 1px solid #2e2a25;
    border-radius: 8px;
    padding: 1.5rem;
  }
  .mt-6 {
    margin-top: 1.5rem;
  }
  .card-title {
    font-size: 1.25rem;
    margin-top: 0;
    margin-bottom: 1.25rem;
    color: #e5ded3;
    font-weight: 600;
  }
  .card-subtitle {
    font-size: 1rem;
    margin-top: 0;
    margin-bottom: 1rem;
    color: #c79a5b;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 1rem;
  }
  .stat-item {
    background: #23201c;
    padding: 1rem;
    border-radius: 6px;
    border: 1px solid #332d26;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .stat-label {
    font-size: 0.8rem;
    color: #8c8477;
  }
  .stat-value {
    font-size: 1.05rem;
    font-weight: 600;
    color: #e8e3d8;
  }
  .text-success {
    color: #4ade80;
  }
  .text-warning {
    color: #facc15;
  }
  .text-accent {
    color: #c79a5b;
  }
  .timeline-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .timeline-list li {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    font-size: 0.9rem;
    border-bottom: 1px dashed #2e2a25;
    padding-bottom: 0.5rem;
  }
  .timeline-year {
    color: #c79a5b;
    font-family: monospace;
    font-weight: 600;
    min-width: 90px;
  }
  .timeline-name {
    color: #f3efe6;
    font-weight: 600;
  }
  .timeline-note {
    color: #8c8477;
    font-size: 0.825rem;
  }
</style>
