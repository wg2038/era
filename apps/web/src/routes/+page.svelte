<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { MonteCarloDecadeReport, SimulationState } from '@era/era-core';
  import { compiledData } from '@era/data/dist/index.js';
  import { SimulationWorkerClient } from '../lib/workers/sim-client';

  let client: SimulationWorkerClient | null = null;
  let isReady: boolean = $state(false);
  let simState: SimulationState | null = $state(null);
  let customSeed: string = $state('42');
  let customBirthYear: number | undefined = $state(undefined);
  let mcReports: MonteCarloDecadeReport[] = $state([]);
  let isRunningMc: boolean = $state(false);

  // 计算当前地点历史名称（如：1935年出生在北平）
  let birthLocationName = $derived.by(() => {
    if (!simState) return '';
    const locId = simState.protagonist.birth_location_id;
    const loc = compiledData.locations.find((l) => l.id === locId);
    if (!loc) return '某地';
    const entry = loc.historicalNames.find(
      (h) =>
        simState!.protagonist.birth_year >= h.startYear &&
        simState!.protagonist.birth_year <= h.endYear,
    );
    const histName = entry ? entry.name : loc.currentName;
    return histName === loc.currentName ? loc.currentName : `${histName}（今${loc.currentName}）`;
  });

  // 当前活跃人物（主角或移魂后的同乡）
  let activePerson = $derived.by(() => {
    if (!simState) return null;
    return simState.active_index === 0
      ? simState.protagonist
      : simState.peers[simState.active_index - 1] ?? null;
  });

  onMount(async () => {
    client = new SimulationWorkerClient();
    try {
      simState = await client.init(42n);
      isReady = true;
    } catch (err) {
      console.error('Failed to initialize simulation worker:', err);
    }
  });

  onDestroy(() => {
    client?.terminate();
  });

  async function startNewGame() {
    if (!client) return;
    const seed = customSeed ? BigInt(customSeed) : BigInt(Date.now());
    simState = await client.init(seed, customBirthYear);
  }

  async function stepYear() {
    if (!client || simState?.is_game_over) return;
    simState = await client.step();
  }

  async function runToEnd() {
    if (!client || simState?.is_game_over) return;
    simState = await client.runToEnd();
  }

  async function switchPeer(index: number) {
    if (!client) return;
    simState = await client.switchPeer(index);
  }

  async function triggerMonteCarlo() {
    if (!client || isRunningMc) return;
    isRunningMc = true;
    try {
      mcReports = await client.runMonteCarloAll(5000);
    } catch (err) {
      console.error('Monte carlo failed:', err);
    } finally {
      isRunningMc = false;
    }
  }
</script>

<div class="stage-container">
  <header class="header">
    <div class="badge">技术展示型内核 · 15+ 适龄提示</div>
    <h1 class="title">《Era》（时代）</h1>
    <p class="subtitle">中国百年人生文字模拟 · 确定性 WebAssembly 内核 (M1 阶段)</p>
  </header>

  <!-- 控制与设置卡片 -->
  <section class="card controls-card">
    <div class="input-row">
      <div class="input-group">
        <label for="seed-input">随机数种子 (Seed)</label>
        <input id="seed-input" type="text" bind:value={customSeed} placeholder="任意整型种子" />
      </div>
      <div class="input-group">
        <label for="birth-year-input">指定出生年代 (可选)</label>
        <input
          id="birth-year-input"
          type="number"
          min="1900"
          max="1999"
          bind:value={customBirthYear}
          placeholder="1900-1999 (留空为随机)"
        />
      </div>
      <button class="btn btn-primary" onclick={startNewGame}>开启人生局</button>
    </div>

    <div class="action-row">
      <button class="btn btn-secondary" onclick={stepYear} disabled={!isReady || simState?.is_game_over}>
        推进一年 (Step Year)
      </button>
      <button class="btn btn-accent" onclick={runToEnd} disabled={!isReady || simState?.is_game_over}>
        推演至终局 (Run To End)
      </button>
      <button class="btn btn-outline" onclick={triggerMonteCarlo} disabled={isRunningMc}>
        {isRunningMc ? '蒙特卡洛计算中...' : '运行全年代蒙特卡洛对齐压测'}
      </button>
    </div>
  </section>

  {#if simState && activePerson}
    <!-- 人生主舞台仪表盘 -->
    <section class="card mt-6 stage-main">
      <div class="character-header">
        <div>
          <span class="era-tag">{simState.current_year} 年</span>
          <h2 class="character-title">
            {activePerson.name} · {activePerson.age} 岁
            {#if !activePerson.is_alive}
              <span class="status-dead">已罹难</span>
            {:else if activePerson.current_year >= 2025}
              <span class="status-alive">见证盛世</span>
            {:else}
              <span class="status-normal">在世</span>
            {/if}
          </h2>
          <p class="birth-info">
            {activePerson.birth_year} 年，出生于<strong>{birthLocationName}</strong>
          </p>
        </div>

        <div class="attrs-grid">
          <div class="attr-box">
            <span class="attr-label">健康</span>
            <span class="attr-val" class:text-danger={activePerson.attributes.health < 30}>
              {activePerson.attributes.health}
            </span>
          </div>
          <div class="attr-box">
            <span class="attr-label">学识</span>
            <span class="attr-val">{activePerson.attributes.knowledge}</span>
          </div>
          <div class="attr-box">
            <span class="attr-label">心情</span>
            <span class="attr-val">{activePerson.attributes.mood}</span>
          </div>
          <div class="attr-box">
            <span class="attr-label">见识</span>
            <span class="attr-val">{activePerson.attributes.insight}</span>
          </div>
        </div>
      </div>

      <!-- 早夭移魂提示 -->
      {#if simState.can_switch_peer}
        <div class="banner-switch">
          <div class="banner-title">【早夭移魂通道已开启】</div>
          <p class="banner-desc">
            主角于未成年期（{activePerson.age}岁）遭遇不测离世。你可以移魂至同年同县出生的另外四人之一，继续体验这个时代的另一种可能：
          </p>
          <div class="peer-buttons">
            {#each simState.peers as peer, idx}
              {#if peer.is_alive}
                <button class="btn btn-switch" onclick={() => switchPeer(idx)}>
                  接续为：{peer.name}（{peer.age}岁 · 健在）
                </button>
              {/if}
            {/each}
          </div>
        </div>
      {/if}

      <!-- 同年同县四人命运对照 -->
      <div class="peers-section">
        <h3 class="section-title">同年同县出生的另外四人（平行命运轨迹）</h3>
        <div class="peers-grid">
          {#each simState.peers as peer, idx}
            <div class="peer-card" class:peer-dead={!peer.is_alive}>
              <div class="peer-header">
                <strong>{peer.name}</strong>
                <span class="peer-badge" class:badge-alive={peer.is_alive}>
                  {peer.is_alive ? '健在' : '身故'}
                </span>
              </div>
              <div class="peer-meta">
                {#if peer.is_alive}
                  现年 {peer.age} 岁
                {:else}
                  享年 {peer.death_age ?? 0} 岁（{peer.death_year} 年）
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- 人生年鉴记录流 -->
      <div class="timeline-section">
        <h3 class="section-title">人生纪年史册</h3>
        <ol class="log-list">
          {#each activePerson.log.slice().reverse() as entry}
            <li class="log-item">
              <div class="log-time">{entry.year} 年 ({entry.age} 岁)</div>
              <div class="log-body">
                <div class="log-title">{entry.title}</div>
                <div class="log-content">{entry.content}</div>
              </div>
            </li>
          {/each}
        </ol>
      </div>
    </section>
  {/if}

  <!-- 蒙特卡洛压测报告表 -->
  {#if mcReports.length > 0}
    <section class="card mt-6">
      <h3 class="card-title">蒙特卡洛 1900–1990 全年代历史人口学对齐报告</h3>
      <div class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th>出生年代</th>
              <th>模拟均寿</th>
              <th>史料基准均寿</th>
              <th>均寿偏差</th>
              <th>婴儿死亡率</th>
              <th>史料婴儿死亡率</th>
              <th>活至60岁比例</th>
              <th>见证2025比例</th>
            </tr>
          </thead>
          <tbody>
            {#each mcReports as r}
              <tr>
                <td><strong>{r.decade}s</strong></td>
                <td>{r.avg_lifespan.toFixed(2)} 岁</td>
                <td>{r.benchmark_expected_lifespan.toFixed(2)} 岁</td>
                <td class:text-success={Math.abs(r.lifespan_deviation_pct) < 5.0} class:text-danger={Math.abs(r.lifespan_deviation_pct) >= 5.0}>
                  {r.lifespan_deviation_pct > 0 ? '+' : ''}{r.lifespan_deviation_pct.toFixed(2)}%
                </td>
                <td>{(r.infant_mortality_rate * 100).toFixed(2)}%</td>
                <td>{(r.benchmark_imr * 100).toFixed(2)}%</td>
                <td>{r.reached_60_pct.toFixed(1)}%</td>
                <td>{r.reached_2025_pct.toFixed(1)}%</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</div>

<style>
  .stage-container {
    max-width: 900px;
    margin: 0 auto;
    padding: 2.5rem 1.5rem;
  }
  .header {
    text-align: center;
    margin-bottom: 2rem;
  }
  .badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
    background-color: #2b2620;
    color: #c79a5b;
    border: 1px solid #4a3e30;
    border-radius: 9999px;
    margin-bottom: 0.75rem;
  }
  .title {
    font-family: var(--font-serif);
    font-size: 2.5rem;
    letter-spacing: 0.08em;
    color: #f3efe6;
    margin: 0 0 0.5rem 0;
  }
  .subtitle {
    font-size: 0.95rem;
    color: #9c9488;
    margin: 0;
  }
  .card {
    background: #181614;
    border: 1px solid #2e2a25;
    border-radius: 8px;
    padding: 1.5rem;
  }
  .mt-6 {
    margin-top: 1.5rem;
  }
  .controls-card {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }
  .input-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: flex-end;
  }
  .input-group {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
    min-width: 180px;
  }
  .input-group label {
    font-size: 0.8rem;
    color: #9c9488;
  }
  .input-group input {
    background: #23201c;
    border: 1px solid #3d352b;
    border-radius: 6px;
    padding: 0.6rem 0.8rem;
    color: #f3efe6;
    font-size: 0.95rem;
  }
  .input-group input:focus {
    outline: none;
    border-color: #c79a5b;
  }
  .action-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .btn {
    padding: 0.6rem 1.25rem;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid transparent;
  }
  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn-primary {
    background: #c79a5b;
    color: #121110;
  }
  .btn-primary:hover:not(:disabled) {
    background: #d8aa6b;
  }
  .btn-secondary {
    background: #2b2620;
    color: #f3efe6;
    border-color: #4a3e30;
  }
  .btn-secondary:hover:not(:disabled) {
    background: #38322a;
  }
  .btn-accent {
    background: #963d32;
    color: #fcefe9;
  }
  .btn-accent:hover:not(:disabled) {
    background: #aa473b;
  }
  .btn-outline {
    background: transparent;
    color: #c79a5b;
    border-color: #c79a5b;
  }
  .btn-outline:hover:not(:disabled) {
    background: rgba(199, 154, 91, 0.1);
  }
  .btn-switch {
    background: #2b392a;
    color: #86efac;
    border-color: #3b573a;
  }
  .btn-switch:hover {
    background: #364834;
  }
  .character-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 1.5rem;
    border-bottom: 1px solid #2e2a25;
    padding-bottom: 1.25rem;
  }
  .era-tag {
    font-size: 0.85rem;
    font-family: monospace;
    color: #c79a5b;
    font-weight: 600;
  }
  .character-title {
    margin: 0.25rem 0 0.5rem 0;
    font-size: 1.75rem;
    font-family: var(--font-serif);
    color: #f3efe6;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .status-normal {
    font-size: 0.75rem;
    background: #23201c;
    color: #86efac;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #2e4a2c;
  }
  .status-alive {
    font-size: 0.75rem;
    background: #4338ca;
    color: #c7d2fe;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }
  .status-dead {
    font-size: 0.75rem;
    background: #7f1d1d;
    color: #fca5a5;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }
  .birth-info {
    color: #9c9488;
    margin: 0;
    font-size: 0.9rem;
  }
  .attrs-grid {
    display: flex;
    gap: 0.75rem;
  }
  .attr-box {
    background: #23201c;
    border: 1px solid #332d26;
    border-radius: 6px;
    padding: 0.5rem 0.85rem;
    text-align: center;
    min-width: 60px;
  }
  .attr-label {
    display: block;
    font-size: 0.7rem;
    color: #8c8477;
  }
  .attr-val {
    font-size: 1.15rem;
    font-weight: 600;
    color: #f3efe6;
  }
  .text-danger {
    color: #f87171;
  }
  .text-success {
    color: #4ade80;
  }
  .banner-switch {
    background: #1c261b;
    border: 1px solid #2d452b;
    border-radius: 6px;
    padding: 1rem 1.25rem;
    margin-top: 1.25rem;
  }
  .banner-title {
    color: #86efac;
    font-weight: 600;
    margin-bottom: 0.35rem;
  }
  .banner-desc {
    font-size: 0.875rem;
    color: #bbf7d0;
    margin: 0 0 0.75rem 0;
  }
  .peer-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .peers-section, .timeline-section {
    margin-top: 1.5rem;
  }
  .section-title {
    font-size: 0.95rem;
    color: #c79a5b;
    margin: 0 0 0.85rem 0;
  }
  .peers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 0.75rem;
  }
  .peer-card {
    background: #23201c;
    border: 1px solid #332d26;
    border-radius: 6px;
    padding: 0.75rem;
  }
  .peer-dead {
    opacity: 0.6;
    border-style: dashed;
  }
  .peer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
  }
  .peer-badge {
    font-size: 0.7rem;
    color: #ef4444;
  }
  .badge-alive {
    color: #4ade80;
  }
  .peer-meta {
    font-size: 0.75rem;
    color: #8c8477;
    margin-top: 0.35rem;
  }
  .log-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .log-item {
    display: flex;
    gap: 1rem;
    font-size: 0.9rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px dashed #28241f;
  }
  .log-time {
    color: #c79a5b;
    font-family: monospace;
    font-weight: 600;
    min-width: 105px;
    font-size: 0.85rem;
  }
  .log-title {
    color: #f3efe6;
    font-weight: 600;
    margin-bottom: 0.2rem;
  }
  .log-content {
    color: #b0a99c;
    font-size: 0.85rem;
    line-height: 1.4;
  }
  .table-container {
    overflow-x: auto;
  }
  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
    text-align: right;
  }
  .data-table th, .data-table td {
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid #2e2a25;
  }
  .data-table th:first-child, .data-table td:first-child {
    text-align: left;
  }
  .data-table th {
    color: #8c8477;
    font-weight: 500;
  }
</style>
