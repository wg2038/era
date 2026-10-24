<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { MonteCarloDecadeReport, SimulationState } from '@era/era-core';
  import { compiledData } from '@era/data/dist/index.js';
  import { SimulationWorkerClient } from '../lib/workers/sim-client';
  import {
    calculateAdaptiveIntervalMs,
    getPacingDescription,
    type PlaySpeedMode,
  } from '../lib/pacing';
  import InkAtmosphereCanvas from '../lib/components/InkAtmosphereCanvas.svelte';
  import EndingSummary from '../lib/components/EndingSummary.svelte';
  import {
    exportSaveAsJSON,
    importSaveFromJSON,
    loadLatestSave,
    saveGameState,
  } from '../lib/storage/save-manager';

  let client: SimulationWorkerClient | null = null;
  let isReady: boolean = $state(false);
  let simState: SimulationState | null = $state(null);
  let customSeed: string = $state('42');
  let customBirthYear: number | undefined = $state(undefined);
  let mcReports: MonteCarloDecadeReport[] = $state([]);
  let isRunningMc: boolean = $state(false);
  let hasSavedGame: boolean = $state(false);
  let fileInputEl: HTMLInputElement | null = $state(null);

  // 自适应节奏状态 (M3 阶段)
  let isPlaying: boolean = $state(false);
  let pacingMode: PlaySpeedMode = $state('normal');
  let speedMultiplier: number = $state(1);
  let timerId: ReturnType<typeof setInterval> | null = null;

  let pacingInfo = $derived.by(() => {
    const by = simState?.protagonist.birth_year ?? 1930;
    return getPacingDescription(by, pacingMode, speedMultiplier);
  });

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
      const existingSave = await loadLatestSave();
      if (existingSave) {
        hasSavedGame = true;
      }
      simState = await client.init(42n);
      isReady = true;
    } catch (err) {
      console.error('Failed to initialize simulation worker:', err);
    }
  });

  $effect(() => {
    if (simState) {
      saveGameState(simState);
    }
  });

  async function loadSavedGame() {
    const saved = await loadLatestSave();
    if (saved) {
      simState = saved;
    }
  }

  function exportCurrentSave() {
    if (simState) {
      exportSaveAsJSON(simState);
    }
  }

  async function handleImportFile(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) {
      try {
        simState = await importSaveFromJSON(target.files[0]);
      } catch (err) {
        alert(err instanceof Error ? err.message : '导入存档失败');
      }
    }
  }

  onDestroy(() => {
    client?.terminate();
    if (timerId) clearInterval(timerId);
  });

  function stopPacingTimer() {
    if (timerId) {
      clearInterval(timerId);
      timerId = null;
    }
    isPlaying = false;
  }

  function startPacingTimer() {
    stopPacingTimer();
    isPlaying = true;
    const by = simState?.protagonist.birth_year ?? 1930;
    const intervalMs = calculateAdaptiveIntervalMs(by, pacingMode, speedMultiplier);

    timerId = setInterval(async () => {
      if (!client || !simState || simState.is_game_over) {
        stopPacingTimer();
        return;
      }
      // 若当前有必须玩家抉择的事件悬挂，自动暂停播放
      if (simState.pending_choice) {
        stopPacingTimer();
        return;
      }

      if (pacingMode === 'fast') {
        simState = await client.stepAuto();
      } else {
        simState = await client.step();
      }

      if (simState.is_game_over || simState.pending_choice) {
        stopPacingTimer();
      }
    }, intervalMs);
  }

  function togglePlay() {
    if (isPlaying) {
      stopPacingTimer();
    } else {
      startPacingTimer();
    }
  }

  async function interactRelative(id: number) {
    if (!client) return;
    simState = await client.interactRelative(id);
  }

  async function interactFriend(id: number) {
    if (!client) return;
    simState = await client.interactFriend(id);
  }

  async function startNewGame() {
    if (!client) return;
    const seed = customSeed ? BigInt(customSeed) : BigInt(Date.now());
    simState = await client.init(seed, customBirthYear);
  }

  async function stepYear() {
    if (!client || simState?.is_game_over) return;
    simState = await client.step();
  }

  async function stepAuto() {
    if (!client || simState?.is_game_over) return;
    simState = await client.stepAuto();
  }

  async function makeChoice(choiceId: string) {
    if (!client || !simState?.pending_choice) return;
    simState = await client.makeChoice(choiceId);
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

<InkAtmosphereCanvas year={simState?.current_year ?? 1930} />

<div class="stage-container">
  <header class="header">
    <div class="badge">技术展示型内核 · 15+ 适龄提示 · 纯客户端离线运行</div>
    <h1 class="title">《Era》（时代）</h1>
    <p class="subtitle">中国百年人生文字模拟 · WebAssembly 确定性内核 · WebGPU 宣纸水墨 · 终局档案 (M4)</p>
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
      <button class="btn" class:btn-primary={!isPlaying} class:btn-accent={isPlaying} onclick={togglePlay} disabled={!isReady || simState?.is_game_over}>
        {isPlaying ? '⏸ 暂停自动推进' : '▶ 自适应自动推演'}
      </button>
      <button class="btn btn-secondary" onclick={stepYear} disabled={!isReady || simState?.is_game_over || isPlaying}>
        推进一年 (Step Year)
      </button>
      <button class="btn btn-secondary" onclick={stepAuto} disabled={!isReady || simState?.is_game_over || isPlaying}>
        快进一年 (Auto Step)
      </button>
      <button class="btn btn-accent" onclick={runToEnd} disabled={!isReady || simState?.is_game_over}>
        推演至终局 (Run To End)
      </button>
      <button class="btn btn-outline" onclick={triggerMonteCarlo} disabled={isRunningMc}>
        {isRunningMc ? '蒙特卡洛计算中...' : '运行全年代历史对齐压测'}
      </button>
    </div>

    <!-- 自适应时间节奏状态条 (M3 核心) -->
    <div class="pacing-bar">
      <div class="pacing-desc">
        ⏱ <strong>自适应时间预算</strong>：{pacingInfo.label}
      </div>
      <div class="pacing-options">
        <label class="radio-label">
          <input type="radio" name="pacingMode" value="normal" bind:group={pacingMode} onchange={() => { if (isPlaying) startPacingTimer(); }} />
          沉浸游玩 (30分钟预算)
        </label>
        <label class="radio-label">
          <input type="radio" name="pacingMode" value="fast" bind:group={pacingMode} onchange={() => { if (isPlaying) startPacingTimer(); }} />
          极速快进 (5分钟预算)
        </label>
        <select class="speed-select" bind:value={speedMultiplier} onchange={() => { if (isPlaying) startPacingTimer(); }}>
          <option value={1}>1.0x 基准速率</option>
          <option value={2}>2.0x 速率</option>
          <option value={5}>5.0x 速率</option>
          <option value={10}>10.0x 超高速</option>
        </select>
      </div>
    </div>

    <!-- 存档与存储管理工具条 (M5 核心) -->
    <div class="storage-bar">
      <div class="storage-info">
        💾 <strong>存档体系</strong>：IndexedDB 逐年自动存档已激活
      </div>
      <div class="storage-actions">
        {#if hasSavedGame}
          <button class="btn btn-tiny-action" onclick={loadSavedGame}>
            📂 读取最近存档
          </button>
        {/if}
        <button class="btn btn-tiny-action" onclick={exportCurrentSave} disabled={!simState}>
          📤 导出存档 (JSON)
        </button>
        <button class="btn btn-tiny-action" onclick={() => fileInputEl?.click()}>
          📥 导入存档 (JSON)
        </button>
        <input
          bind:this={fileInputEl}
          type="file"
          accept=".json"
          style="display: none;"
          onchange={handleImportFile}
        />
      </div>
    </div>
  </section>

  {#if simState && simState.is_game_over && !simState.can_switch_peer}
    <EndingSummary simState={simState} onRestart={startNewGame} />
  {:else if simState && activePerson}
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
          <div class="currency-badge">
            <span class="currency-amount">💰 {simState.current_currency.formatted_amount}</span>
            <span class="currency-desc">（{simState.current_currency.era_description}）</span>
          </div>
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

      <!-- 待玩家抉择事件交互区 (M2 核心) -->
      {#if simState.pending_choice}
        <div class="choice-event-card" class:crossroads={simState.pending_choice.is_crossroads}>
          <div class="choice-header">
            <span class="event-tier-tag" class:tier-era={simState.pending_choice.tier === 'era'}>
              {simState.pending_choice.is_crossroads ? '★ 重大人生关口' : simState.pending_choice.tier === 'era' ? '宏观时代事件' : '人生分支抉择'}
            </span>
            <h3 class="choice-event-title">{simState.pending_choice.title}</h3>
          </div>
          <p class="choice-event-narrative">{simState.pending_choice.narrative}</p>

          <div class="choices-container">
            {#each simState.pending_choice.choices as opt}
              <button class="choice-btn" onclick={() => makeChoice(opt.id)}>
                <div class="choice-text">{opt.text}</div>
                <div class="effects-row">
                  {#if opt.dh !== 0}<span class="effect-tag" class:gain={opt.dh > 0}>健康 {opt.dh > 0 ? `+${opt.dh}` : opt.dh}</span>{/if}
                  {#if opt.dk !== 0}<span class="effect-tag" class:gain={opt.dk > 0}>学识 {opt.dk > 0 ? `+${opt.dk}` : opt.dk}</span>{/if}
                  {#if opt.dm !== 0}<span class="effect-tag" class:gain={opt.dm > 0}>心情 {opt.dm > 0 ? `+${opt.dm}` : opt.dm}</span>{/if}
                  {#if opt.di !== 0}<span class="effect-tag" class:gain={opt.di > 0}>见识 {opt.di > 0 ? `+${opt.di}` : opt.di}</span>{/if}
                  {#if opt.d_wealth !== 0}<span class="effect-tag" class:gain={opt.d_wealth > 0}>购买力 {opt.d_wealth > 0 ? `+${opt.d_wealth}` : opt.d_wealth}</span>{/if}
                  {#if opt.risk}<span class="effect-tag risk-tag">风险 {Math.round(opt.risk * 100)}%</span>{/if}
                </div>
              </button>
            {/each}
          </div>
        </div>
      {/if}

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

      <!-- 家族谱系树 (M3 核心) -->
      <div class="social-section mt-4">
        <div class="social-header">
          <h3 class="section-title">家族亲族谱系（存亡与亲情羁绊）</h3>
          <span class="social-hint">尊亲生老病死随时代演进 · 侍奉可增进亲密与心情</span>
        </div>
        <div class="kinship-grid">
          {#each activePerson.family_tree as member}
            <div class="kinship-card" class:member-dead={!member.is_alive}>
              <div class="kinship-top">
                <span class="kinship-relation">{member.name}</span>
                <span class="kinship-status" class:alive={member.is_alive}>
                  {member.is_alive ? `${member.age} 岁 · 健在` : `已故（享年 ${member.age} 岁）`}
                </span>
              </div>
              <div class="kinship-occ">{member.occupation}</div>
              <div class="intimacy-bar-wrapper">
                <div class="intimacy-label">
                  <span>亲密度</span>
                  <span>{member.intimacy}/100</span>
                </div>
                <div class="intimacy-track">
                  <div class="intimacy-fill" style="width: {member.intimacy}%"></div>
                </div>
              </div>
              {#if member.is_alive}
                <button class="btn btn-tiny mt-2" onclick={() => interactRelative(member.id)}>
                  问安奉养 (-3购买力)
                </button>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- 朋友圈与朋辈社交 (M3 核心) -->
      <div class="social-section mt-4">
        <div class="social-header">
          <h3 class="section-title">朋辈知交圈（不维系随岁月自然淡化）</h3>
          <span class="social-hint">⚠️ 若无聚谈往来，每年亲密自然衰减 2 点</span>
        </div>
        {#if activePerson.friends.length === 0}
          <div class="social-empty">幼年尚无相识知己，待 7 岁发小、16 岁同窗之时结交……</div>
        {:else}
          <div class="kinship-grid">
            {#each activePerson.friends as friend}
              <div class="kinship-card" class:member-dead={!friend.is_alive}>
                <div class="kinship-top">
                  <span class="kinship-relation">{friend.name}</span>
                  <span class="friend-tag">{friend.tag}</span>
                </div>
                <div class="kinship-occ">
                  {friend.is_alive ? `${friend.age} 岁 · 结识于 ${friend.acquaintance_year} 年` : '故人已逝'}
                </div>
                <div class="intimacy-bar-wrapper">
                  <div class="intimacy-label">
                    <span>交谊羁绊</span>
                    <span class:text-warning={friend.intimacy < 30}>{friend.intimacy}/100</span>
                  </div>
                  <div class="intimacy-track">
                    <div class="intimacy-fill friend-fill" style="width: {friend.intimacy}%"></div>
                  </div>
                </div>
                {#if friend.is_alive}
                  <button class="btn btn-tiny mt-2" onclick={() => interactFriend(friend.id)}>
                    拜会小聚 (-2购买力)
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>

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
  .currency-badge {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }
  .currency-amount {
    color: #eab308;
    font-weight: 600;
  }
  .currency-desc {
    color: #8c8477;
    font-size: 0.78rem;
  }

  /* 抉择事件交互卡片 */
  .choice-event-card {
    background: #1f1b17;
    border: 1px solid #5a4933;
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    margin: 1.5rem 0;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }
  .choice-event-card.crossroads {
    border-color: #d97706;
    background: linear-gradient(180deg, #2b1f14 0%, #1f1812 100%);
    box-shadow: 0 0 25px rgba(217, 119, 6, 0.2);
  }
  .choice-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.6rem;
  }
  .event-tier-tag {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    background: #3b2c1f;
    color: #f59e0b;
    font-weight: 600;
  }
  .event-tier-tag.tier-era {
    background: #1e3a5f;
    color: #60a5fa;
  }
  .choice-event-title {
    margin: 0;
    font-size: 1.2rem;
    font-family: var(--font-serif);
    color: #fef08a;
  }
  .choice-event-narrative {
    color: #d1c7b7;
    font-size: 0.95rem;
    line-height: 1.55;
    margin: 0 0 1.25rem 0;
  }
  .choices-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .choice-btn {
    text-align: left;
    background: #28221c;
    border: 1px solid #4a3c2c;
    border-radius: 6px;
    padding: 0.85rem 1rem;
    color: #f3efe6;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .choice-btn:hover {
    background: #362e26;
    border-color: #c79a5b;
    transform: translateY(-1px);
  }
  .choice-text {
    font-size: 0.95rem;
    font-weight: 500;
    margin-bottom: 0.4rem;
  }
  .effects-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .effect-tag {
    font-size: 0.75rem;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    background: #332724;
    color: #f87171;
  }
  .effect-tag.gain {
    background: #203524;
    color: #4ade80;
  }
  .effect-tag.risk-tag {
    background: #422006;
    color: #fb923c;
  }
  /* 自适应节奏控制条 */
  .pacing-bar {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    background: #1c1916;
    border: 1px solid #3d352b;
    border-radius: 6px;
    padding: 0.6rem 0.85rem;
    margin-top: 0.85rem;
    gap: 0.75rem;
  }
  .pacing-desc {
    font-size: 0.85rem;
    color: #e6dfd3;
  }
  .pacing-options {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }
  .radio-label {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.8rem;
    color: #b0a99c;
    cursor: pointer;
  }
  .speed-select {
    background: #28241f;
    color: #c79a5b;
    border: 1px solid #4a3e30;
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
    font-size: 0.8rem;
  }
  .storage-bar {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    background: #141210;
    border: 1px dashed #3d352b;
    border-radius: 6px;
    padding: 0.5rem 0.85rem;
    margin-top: 0.6rem;
    gap: 0.5rem;
  }
  .storage-info {
    font-size: 0.78rem;
    color: #a8a29e;
  }
  .storage-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .btn-tiny-action {
    background: #201c19;
    border: 1px solid #4a3e2f;
    border-radius: 4px;
    color: #d1c7b7;
    font-size: 0.75rem;
    padding: 0.25rem 0.6rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-tiny-action:hover:not(:disabled) {
    background: #2e2822;
    border-color: #c79a5b;
    color: #f3efe6;
  }

  /* 社交与亲属卡片体系 (M3) */
  .social-section {
    background: #161412;
    border: 1px solid #2e2a25;
    border-radius: 8px;
    padding: 1.25rem;
  }
  .social-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  .social-hint {
    font-size: 0.75rem;
    color: #9c9488;
  }
  .social-empty {
    color: #78716c;
    font-size: 0.85rem;
    font-style: italic;
    padding: 1rem 0;
  }
  .kinship-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 0.85rem;
  }
  .kinship-card {
    background: #211d19;
    border: 1px solid #383127;
    border-radius: 6px;
    padding: 0.85rem;
    transition: all 0.2s ease;
  }
  .kinship-card.member-dead {
    opacity: 0.45;
    border-color: #26221d;
    background: #181614;
  }
  .kinship-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.3rem;
  }
  .kinship-relation {
    font-weight: 600;
    font-size: 0.95rem;
    color: #f3efe6;
  }
  .kinship-status {
    font-size: 0.72rem;
    color: #8c8477;
  }
  .kinship-status.alive {
    color: #4ade80;
  }
  .friend-tag {
    font-size: 0.72rem;
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
    background: #1e293b;
    color: #38bdf8;
  }
  .kinship-occ {
    font-size: 0.78rem;
    color: #a8a29e;
    margin-bottom: 0.5rem;
  }
  .intimacy-bar-wrapper {
    margin-top: 0.35rem;
  }
  .intimacy-label {
    display: flex;
    justify-content: space-between;
    font-size: 0.72rem;
    color: #78716c;
    margin-bottom: 0.2rem;
  }
  .intimacy-track {
    width: 100%;
    height: 5px;
    background: #2e2820;
    border-radius: 9999px;
    overflow: hidden;
  }
  .intimacy-fill {
    height: 100%;
    background: linear-gradient(90deg, #d97706, #f59e0b);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }
  .intimacy-fill.friend-fill {
    background: linear-gradient(90deg, #0284c7, #38bdf8);
  }
  .btn-tiny {
    width: 100%;
    padding: 0.35rem 0.5rem;
    font-size: 0.75rem;
    background: #2b251e;
    color: #fbbf24;
    border: 1px solid #4a3e2f;
    border-radius: 4px;
  }
  .btn-tiny:hover {
    background: #383025;
    border-color: #f59e0b;
  }
  .text-warning {
    color: #f97316;
  }
</style>
