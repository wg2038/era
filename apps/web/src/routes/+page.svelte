<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { SimulationState } from '@era/era-core';
  import { compiledData } from '@era/data/dist/index.js';
  import { SimulationWorkerClient } from '../lib/workers/sim-client';
  import EndingSummary from '../lib/components/EndingSummary.svelte';
  import {
    exportSaveAsJSON,
    importSaveFromJSON,
    loadLatestSave,
    saveGameState,
  } from '../lib/storage/save-manager';

  let client: SimulationWorkerClient | null = null;
  let isReady: boolean = $state(false);
  let isAdvancing: boolean = $state(false);
  let simState: SimulationState | null = $state(null);

  // 玩家当前选中的年度重心（读书 / 谋生 / 家人 / 身体 / 人情）
  let selectedFocus: 'Study' | 'Livelihood' | 'Family' | 'Health' | 'Social' = $state('Study');

  // 岔路口选中项
  let selectedChoiceId: string | null = $state(null);

  // 人生节点反思心声输入
  let milestoneQuoteInput: string = $state('');

  // 自动模式开关（默认关闭）
  let isAutoMode: boolean = $state(false);
  let autoTimer: ReturnType<typeof setInterval> | null = null;

  // 自定义种子与存档
  let customSeed: string = $state('42');
  let customBirthYear: number | undefined = $state(undefined);
  let hasSavedGame: boolean = $state(false);
  let fileInputEl: HTMLInputElement | null = $state(null);

  // 当前活跃人物（主角或移魂后的同伴）
  let activePerson = $derived.by(() => {
    if (!simState) return null;
    return simState.active_index === 0
      ? simState.protagonist
      : simState.peers[simState.active_index - 1] ?? simState.protagonist;
  });

  // 当前地点历史名称
  let birthLocationName = $derived.by(() => {
    if (!simState || !activePerson) return '';
    const locId = activePerson.birth_location_id;
    const loc = compiledData.locations.find((l) => l.id === locId);
    if (!loc) return simState.birth_current_name;
    const entry = loc.historicalNames.find(
      (h) =>
        activePerson!.birth_year >= h.startYear &&
        activePerson!.birth_year <= h.endYear,
    );
    const histName = entry ? entry.name : loc.currentName;
    return histName === loc.currentName ? loc.currentName : `${histName}（今${loc.currentName}）`;
  });

  // 当触发岔路口事件时，默认选中第一项
  $effect(() => {
    if (simState?.pending_choice) {
      if (!selectedChoiceId || !simState.pending_choice.choices.some((c) => c.id === selectedChoiceId)) {
        selectedChoiceId = simState.pending_choice.choices[0]?.id ?? null;
      }
    } else {
      selectedChoiceId = null;
    }
  });

  // 当触发人生节点自省时，预填默认心声
  $effect(() => {
    if (simState?.active_milestone) {
      milestoneQuoteInput = simState.active_milestone.default_quote;
    }
  });

  // 自动存档
  $effect(() => {
    if (simState) {
      saveGameState(simState);
    }
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

  onDestroy(() => {
    if (autoTimer) clearInterval(autoTimer);
    client?.terminate();
  });

  // 回合制推进一年（按所选生命重心）
  async function handleAdvanceYear() {
    if (!client || isAdvancing || !simState || simState.is_game_over) return;
    isAdvancing = true;
    try {
      simState = await client.advanceYear(selectedFocus);
    } catch (err) {
      console.error('Advance year failed:', err);
    } finally {
      isAdvancing = false;
    }
  }

  // 提交重大岔路口抉择
  async function handleCommitChoice() {
    if (!client || isAdvancing || !selectedChoiceId) return;
    isAdvancing = true;
    try {
      simState = await client.makeChoice(selectedChoiceId);
    } catch (err) {
      console.error('Make choice failed:', err);
    } finally {
      isAdvancing = false;
    }
  }

  // 封存人生节点心声
  async function handleSubmitMilestone() {
    if (!client || isAdvancing) return;
    isAdvancing = true;
    try {
      simState = await client.submitMilestone(milestoneQuoteInput);
    } catch (err) {
      console.error('Submit milestone failed:', err);
    } finally {
      isAdvancing = false;
    }
  }

  // 亲友微互动
  async function handleInteractRelative(relativeId: number, action: string) {
    if (!client || isAdvancing) return;
    try {
      simState = await client.interactRelativeAction(relativeId, action);
    } catch (err) {
      console.warn('Relative action failed:', err);
    }
  }

  async function handleInteractFriend(friendId: number, action: string) {
    if (!client || isAdvancing) return;
    try {
      simState = await client.interactFriendAction(friendId, action);
    } catch (err) {
      console.warn('Friend action failed:', err);
    }
  }

  // 早夭移魂切换至同乡
  async function handleSwitchPeer(peerIndex: number) {
    if (!client || !simState?.can_switch_peer) return;
    try {
      simState = await client.switchPeer(peerIndex);
    } catch (err) {
      console.error('Switch peer failed:', err);
    }
  }

  // 自动模式开关切换
  function toggleAutoMode() {
    isAutoMode = !isAutoMode;
    if (isAutoMode) {
      autoTimer = setInterval(async () => {
        if (!client || isAdvancing || !simState || simState.is_game_over) {
          if (simState?.is_game_over && autoTimer) {
            clearInterval(autoTimer);
            isAutoMode = false;
          }
          return;
        }
        // 遇到岔路口或人生节点时，自动模式暂停等待玩家决断
        if (simState.pending_choice?.is_crossroads || simState.active_milestone) {
          return;
        }
        isAdvancing = true;
        try {
          simState = await client.stepAuto();
        } catch (err) {
          console.error('Auto step error:', err);
        } finally {
          isAdvancing = false;
        }
      }, 1200);
    } else if (autoTimer) {
      clearInterval(autoTimer);
      autoTimer = null;
    }
  }

  // 重启新游戏
  async function handleRestart(seedOverride?: bigint | number, birthYearOverride?: number) {
    if (!client) return;
    if (autoTimer) {
      clearInterval(autoTimer);
      autoTimer = null;
      isAutoMode = false;
    }
    const seed = seedOverride ?? (customSeed ? BigInt(customSeed) : BigInt(Date.now()));
    isAdvancing = true;
    try {
      simState = await client.init(seed, birthYearOverride ?? customBirthYear);
    } finally {
      isAdvancing = false;
    }
  }

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
        const loaded = await importSaveFromJSON(target.files[0]);
        simState = loaded;
      } catch (err) {
        alert(err instanceof Error ? err.message : '存档解析失败');
      }
    }
  }
</script>

{#if !isReady || !simState || !activePerson}
  <div class="loading-screen">
    <div class="loading-spinner"></div>
    <div class="loading-text">正在推演百年历史与人生星历……</div>
  </div>
{:else if simState.is_game_over}
  <EndingSummary {simState} onRestart={() => handleRestart()} />
{:else}
  <!-- 顶部细状态条 -->
  <header class="top-bar">
    <div class="timeline-info">
      <span class="brand">ERA</span>
      <span>·</span>
      <span class="year">{activePerson.current_year} 年</span>
      <span>·</span>
      <span>{activePerson.age} 岁</span>
      <span>·</span>
      <span>{birthLocationName}</span>
    </div>
    <div class="currency-badge">
      {simState.current_currency.formatted_amount}
    </div>
  </header>

  <main class="layout-grid">
    <!-- 主叙事流 -->
    <section class="narrative-stream">
      <!-- 过去的叙事：淡出降噪（展示前 2 条） -->
      {#if activePerson.log.length > 1}
        {#each activePerson.log.slice(-3, -1) as pastLog}
          <div class="log-item past">
            <div class="log-header">{pastLog.year} 年（{pastLog.age} 岁）· {pastLog.title}</div>
            <p>{pastLog.content}</p>
          </div>
        {/each}
      {/if}

      <!-- 当年的叙事焦点卡片 -->
      {#if activePerson.log.length > 0}
        {@const currentLog = activePerson.log[activePerson.log.length - 1]}
        {#if currentLog}
          <div class="log-item current">
            <div class="log-header">
              {currentLog.year} 年（{currentLog.age} 岁）· {currentLog.title}
            </div>
            <p>{currentLog.content}</p>
          </div>
        {/if}
      {/if}

      <!-- 情况 A：重大岔路口事件（升学/择业/迁徙等） -->
      {#if simState.pending_choice}
        <article class="decision-card crossroads-card">
          <div class="card-badge">重大岔路口 · 不可逆抉择</div>
          <h2 class="card-title">{simState.pending_choice.title}</h2>
          <div class="narrative-text">
            <p>{simState.pending_choice.narrative}</p>
          </div>

          <div class="choice-list">
            {#each simState.pending_choice.choices as opt}
              <label
                class="choice-card"
                class:selected={selectedChoiceId === opt.id}
              >
                <div class="choice-header">
                  <span class="choice-name">{opt.text}</span>
                  <input
                    type="radio"
                    name="crossroads_opt"
                    value={opt.id}
                    checked={selectedChoiceId === opt.id}
                    onchange={() => (selectedChoiceId = opt.id)}
                  />
                </div>
                <div class="choice-cost">
                  <strong>代价与影响：</strong>
                  {#if opt.dh !== 0}健康 {opt.dh > 0 ? `+${opt.dh}` : opt.dh} {/if}
                  {#if opt.dk !== 0}学识 {opt.dk > 0 ? `+${opt.dk}` : opt.dk} {/if}
                  {#if opt.dm !== 0}心情 {opt.dm > 0 ? `+${opt.dm}` : opt.dm} {/if}
                  {#if opt.di !== 0}见识 {opt.di > 0 ? `+${opt.di}` : opt.di} {/if}
                  {#if opt.d_wealth !== 0}购买力 {opt.d_wealth > 0 ? `+${opt.d_wealth}` : opt.d_wealth} PPP{/if}
                  {#if opt.dh === 0 && opt.dk === 0 && opt.dm === 0 && opt.di === 0 && opt.d_wealth === 0}
                    随遇而安，静待时代激荡
                  {/if}
                </div>
              </label>
            {/each}
          </div>

          <div class="action-row">
            <button
              class="btn-action"
              onclick={handleCommitChoice}
              disabled={isAdvancing || !selectedChoiceId}
              type="button"
            >
              {isAdvancing ? '抉择生效中……' : '立下决断，承担因果'}
            </button>
          </div>
        </article>

      <!-- 情况 B：人生节点反思提问（5、15、30、50、70 岁） -->
      {:else if simState.active_milestone}
        <article class="decision-card milestone-card">
          <div class="card-badge">人生节点暂停 · {simState.active_milestone.age} 岁</div>
          <h2 class="card-title">{simState.active_milestone.title}</h2>
          <div class="narrative-text">
            <p>{simState.active_milestone.narrative}</p>
          </div>

          <div class="prompt-box">
            <div class="prompt-question">{simState.active_milestone.question}</div>
            <div class="prompt-hint">
              请写下一句话。这句话将被岁月尘封，直至终局寿终盖棺之时，重新与你相认。
            </div>

            <textarea
              class="quote-input"
              rows="3"
              bind:value={milestoneQuoteInput}
              placeholder="写下你内心的回答……"
            ></textarea>
            <div class="char-count">{milestoneQuoteInput.length} 字</div>
          </div>

          <!-- 昔年回响 -->
          {#if activePerson.milestone_quotes.length > 0}
            <div class="past-quotes">
              <div class="past-title">昔年回响：</div>
              {#each activePerson.milestone_quotes as mq}
                <div class="past-item">
                  <strong>{mq.age} 岁：</strong>“{mq.quote}”
                </div>
              {/each}
            </div>
          {/if}

          <div class="action-row">
            <button
              class="btn-action"
              onclick={handleSubmitMilestone}
              disabled={isAdvancing}
              type="button"
            >
              {isAdvancing ? '封存中……' : '封存这句话，继续迈步'}
            </button>
          </div>
        </article>

      <!-- 情况 C：年度核心回合（选择本年度生命重心） -->
      {:else}
        <div class="decision-card focus-card">
          <div class="decision-title">
            <span>请决定 {activePerson.current_year} 年的生命重心</span>
            <span class="focus-subtitle">（每年必选一项，改变属性与后续因果）</span>
          </div>

          <div class="focus-grid">
            <label class="focus-option" class:selected={selectedFocus === 'Study'}>
              <div>
                <div class="focus-name">📖 读书求学</div>
                <div class="focus-desc">研读经典与新知，学识+5，开销家资 5 PPP，略微耗神</div>
              </div>
              <input
                type="radio"
                name="focus"
                value="Study"
                checked={selectedFocus === 'Study'}
                onchange={() => (selectedFocus = 'Study')}
              />
            </label>

            <label class="focus-option" class:selected={selectedFocus === 'Livelihood'}>
              <div>
                <div class="focus-name">💼 帮贴谋生</div>
                <div class="focus-desc">奔波做工经商，购买力+10 PPP，体魄操劳（健康-2），学识滞后</div>
              </div>
              <input
                type="radio"
                name="focus"
                value="Livelihood"
                checked={selectedFocus === 'Livelihood'}
                onchange={() => (selectedFocus = 'Livelihood')}
              />
            </label>

            <label class="focus-option" class:selected={selectedFocus === 'Family'}>
              <div>
                <div class="focus-name">🏠 顾全家人</div>
                <div class="focus-desc">陪伴侍奉双亲老小，在世亲族亲密+6，心情舒缓，消耗微量家资</div>
              </div>
              <input
                type="radio"
                name="focus"
                value="Family"
                checked={selectedFocus === 'Family'}
                onchange={() => (selectedFocus = 'Family')}
              />
            </label>

            <label class="focus-option" class:selected={selectedFocus === 'Health'}>
              <div>
                <div class="focus-name">🏃 调养身体</div>
                <div class="focus-desc">避寒节欲强身，健康+6，显著提升对时代疫病与饥荒的抵御力</div>
              </div>
              <input
                type="radio"
                name="focus"
                value="Health"
                checked={selectedFocus === 'Health'}
                onchange={() => (selectedFocus = 'Health')}
              />
            </label>

            <label class="focus-option" class:selected={selectedFocus === 'Social'}>
              <div>
                <div class="focus-name">🍵 维系人情</div>
                <div class="focus-desc">与乡邻好友走动，朋友亲密+8，见识+3，消耗微量礼数</div>
              </div>
              <input
                type="radio"
                name="focus"
                value="Social"
                checked={selectedFocus === 'Social'}
                onchange={() => (selectedFocus = 'Social')}
              />
            </label>
          </div>

          <!-- 亲友微互动通道 -->
          <div class="interaction-bar">
            <span class="interact-label">
              亲友行动（年限2次，余{activePerson.actions_remaining_this_year}次）：
            </span>

            {#if activePerson.actions_remaining_this_year > 0}
              {#each activePerson.family_tree.filter((m) => m.is_alive).slice(0, 2) as rel}
                <button
                  class="interact-chip"
                  onclick={() => handleInteractRelative(rel.id, 'Visit')}
                  type="button"
                >
                  探望{rel.name} (亲密+18)
                </button>
                <button
                  class="interact-chip"
                  onclick={() => handleInteractRelative(rel.id, 'Remit')}
                  type="button"
                >
                  寄资{rel.name} (-5 PPP)
                </button>
              {/each}

              {#each activePerson.friends.filter((f) => f.is_alive).slice(0, 1) as friend}
                <button
                  class="interact-chip"
                  onclick={() => handleInteractFriend(friend.id, 'Visit')}
                  type="button"
                >
                  拜会好友{friend.name} (亲密+20)
                </button>
              {/each}
            {:else}
              <span class="interact-muted">今年亲友互动额度已用尽</span>
            {/if}
          </div>
        </div>

        <!-- 手机端折叠抽屉：属性、家谱、同龄人 -->
        <details class="mobile-drawer">
          <summary>
            <span>查看属性面板、家族亲眷与同龄四人</span>
            <span>▼</span>
          </summary>
          <div class="drawer-content">
            <!-- 四维属性 -->
            <div class="stats-grid">
              <div class="stat-box">
                <span class="stat-name">健康</span>
                <span class="stat-value">{activePerson.attributes.health}</span>
              </div>
              <div class="stat-box">
                <span class="stat-name">学识</span>
                <span class="stat-value">{activePerson.attributes.knowledge}</span>
              </div>
              <div class="stat-box">
                <span class="stat-name">心情</span>
                <span class="stat-value">{activePerson.attributes.mood}</span>
              </div>
              <div class="stat-box">
                <span class="stat-name">见识</span>
                <span class="stat-value">{activePerson.attributes.insight}</span>
              </div>
            </div>

            <!-- 家眷状态 -->
            <div class="drawer-section">
              <div class="section-title">家族亲眷</div>
              <div class="family-list">
                {#each activePerson.family_tree as rel}
                  <div class="family-item" class:deceased={!rel.is_alive}>
                    <span>{rel.relation}: {rel.name} ({rel.age}岁)</span>
                    <span>{rel.is_alive ? `亲密 ${rel.intimacy}` : '已故'}</span>
                  </div>
                {/each}
              </div>
            </div>

            <!-- 同行四人 -->
            <div class="drawer-section">
              <div class="section-title">同年同县（{simState.birth_historical_name}）同龄人</div>
              <div class="peer-list">
                {#each simState.peers as peer}
                  <div class="peer-item" class:deceased={!peer.is_alive}>
                    <span class="name">{peer.name}</span>
                    <span class="destiny">{peer.is_alive ? `${peer.age} 岁·在世` : `逝于 ${peer.death_age ?? peer.age} 岁`}</span>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        </details>

        <!-- 底部固定操作栏（手机固定在底部拇指区，高度 >= 48px；电脑端就地显示） -->
        <section class="action-container">
          <div class="action-row">
            <button
              class="btn-advance"
              onclick={handleAdvanceYear}
              disabled={isAdvancing}
              type="button"
            >
              {isAdvancing ? '岁月流转中……' : `推进至 ${activePerson.current_year + 1} 年`}
            </button>
            <div class="auto-toggle-row">
              <label class="auto-switch-label">
                <input
                  type="checkbox"
                  checked={isAutoMode}
                  onchange={toggleAutoMode}
                />
                <span>自动推演模式（按性格自决，遇关口暂停）</span>
              </label>
            </div>
          </div>
        </section>
      {/if}

      <!-- 早夭移魂救赎提示 -->
      {#if simState.can_switch_peer}
        <div class="switch-peer-card">
          <div class="switch-title">早夭移魂 · 命运之门未绝</div>
          <p class="switch-desc">
            你在此世未成年（{activePerson.age}岁）不幸早夭，但同一方水土同年出生的同乡尚有健在者。你可选择借魂续命，继续体验这段岁月：
          </p>
          <div class="switch-buttons">
            {#each simState.peers as peer, idx}
              {#if peer.is_alive}
                <button
                  class="btn-peer-switch"
                  onclick={() => handleSwitchPeer(idx)}
                  type="button"
                >
                  转生为：{peer.name} ({peer.age}岁)
                </button>
              {/if}
            {/each}
          </div>
        </div>
      {/if}
    </section>

    <!-- 桌面端常驻右侧栏（>= 900px） -->
    <aside class="desktop-sidebar">
      <!-- 四维属性 -->
      <div class="side-card">
        <h3>身心属性</h3>
        <div class="stats-grid">
          <div class="stat-box">
            <span class="stat-name">健康</span>
            <span class="stat-value">{activePerson.attributes.health}</span>
          </div>
          <div class="stat-box">
            <span class="stat-name">学识</span>
            <span class="stat-value">{activePerson.attributes.knowledge}</span>
          </div>
          <div class="stat-box">
            <span class="stat-name">心情</span>
            <span class="stat-value">{activePerson.attributes.mood}</span>
          </div>
          <div class="stat-box">
            <span class="stat-name">见识</span>
            <span class="stat-value">{activePerson.attributes.insight}</span>
          </div>
        </div>
        <div class="wealth-info">
          购买力储备：<strong>{activePerson.wealth_ppp} PPP</strong>
        </div>
      </div>

      <!-- 家族亲眷 -->
      <div class="side-card">
        <h3>家族羁绊</h3>
        <div class="family-list">
          {#each activePerson.family_tree as rel}
            <div class="family-item" class:deceased={!rel.is_alive}>
              <span>{rel.name}（{rel.occupation}）</span>
              <span>{rel.is_alive ? `亲密 ${rel.intimacy}` : '已故'}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- 同行四人 -->
      <div class="side-card">
        <h3>同年同县同行者</h3>
        <div class="peer-list">
          {#each simState.peers as peer}
            <div class="peer-item" class:deceased={!peer.is_alive}>
              <span class="name">{peer.name}</span>
              <span class="destiny">{peer.is_alive ? `${peer.age} 岁` : `逝于 ${peer.death_age ?? peer.age} 岁`}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- 存档与复现控制 -->
      <div class="side-card">
        <h3>轮回与存档</h3>
        <div class="archive-row">
          <button class="btn-sm" onclick={exportCurrentSave} type="button">导出存档</button>
          <button class="btn-sm" onclick={() => fileInputEl?.click()} type="button">导入存档</button>
          {#if hasSavedGame}
            <button class="btn-sm" onclick={loadSavedGame} type="button">读取本地存档</button>
          {/if}
          <input
            bind:this={fileInputEl}
            type="file"
            accept=".json"
            style="display: none;"
            onchange={handleImportFile}
          />
        </div>
        <div class="seed-input-row">
          <input
            class="input-seed"
            type="text"
            bind:value={customSeed}
            placeholder="种子 Seed"
          />
          <button class="btn-sm btn-seed" onclick={() => handleRestart()} type="button">
            指定种子重启
          </button>
        </div>
      </div>
    </aside>
  </main>
{/if}

<style>
  /* 顶部紧凑状态条 */
  .top-bar {
    position: sticky;
    top: 0;
    z-index: 30;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    background: rgba(20, 9, 12, 0.88);
    border-bottom: 1px solid var(--border);
    padding: 10px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    letter-spacing: 0.05em;
  }
  @media (prefers-color-scheme: light) {
    .top-bar {
      background: rgba(251, 239, 241, 0.9);
    }
  }

  .top-bar .brand {
    font-weight: 700;
    color: var(--accent);
    letter-spacing: 0.1em;
  }

  .top-bar .timeline-info {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-muted);
  }

  .top-bar .timeline-info .year {
    color: var(--text);
    font-weight: 600;
  }

  .top-bar .currency-badge {
    background: rgba(163, 38, 63, 0.2);
    border: 1px solid var(--border);
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 12px;
    color: var(--accent-soft);
  }

  /* 主网格容器 */
  .layout-grid {
    flex: 1;
    width: 100%;
    max-width: 980px;
    margin: 0 auto;
    display: grid;
    grid-template-columns: 1fr;
    padding: 18px 16px 200px;
    gap: 20px;
  }

  @media (min-width: 900px) {
    .layout-grid {
      grid-template-columns: 560px 340px;
      padding: 28px 16px 40px;
      align-items: start;
    }
  }

  /* 叙事历史流 */
  .narrative-stream {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 560px;
  }

  .log-item {
    padding: 14px 18px;
    border-radius: 8px;
    font-family: var(--font-serif);
    font-size: 16px;
    line-height: 1.75;
    transition: all 0.2s ease;
  }

  .log-item.past {
    background: transparent;
    border-left: 2px solid var(--border);
    color: var(--text-muted);
    opacity: 0.65;
  }

  .log-item.current {
    background: var(--card);
    border: 1px solid var(--border);
    border-left: 4px solid var(--primary);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    color: var(--text);
    font-size: 17px;
  }

  .log-header {
    font-size: 13px;
    font-family: var(--font-sans);
    color: var(--accent);
    margin-bottom: 6px;
    font-weight: 600;
  }

  /* 决策卡片通用 */
  .decision-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
  }

  .card-badge {
    display: inline-block;
    font-size: 11px;
    font-weight: 600;
    color: #ffffff;
    background: var(--primary);
    padding: 2px 8px;
    border-radius: 4px;
    margin-bottom: 10px;
  }

  .card-title {
    font-family: var(--font-serif);
    font-size: 20px;
    font-weight: 700;
    color: var(--text);
    margin-bottom: 12px;
  }

  .narrative-text {
    font-family: var(--font-serif);
    font-size: 16px;
    line-height: 1.7;
    color: var(--text);
    margin-bottom: 16px;
  }

  /* 重心决策面板 */
  .focus-card {
    background: var(--card-highlight);
  }

  .decision-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--accent);
    margin-bottom: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .focus-subtitle {
    font-size: 12px;
    font-weight: normal;
    color: var(--text-muted);
  }

  .focus-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 10px;
  }

  .focus-option {
    min-height: 52px;
    padding: 10px 14px;
    border-radius: 8px;
    border: 1.5px solid var(--border);
    background: var(--card);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: all 0.15s ease-in-out;
  }

  .focus-option:hover {
    border-color: var(--accent);
    background: rgba(163, 38, 63, 0.15);
  }

  .focus-option.selected {
    border-color: var(--accent);
    background: rgba(163, 38, 63, 0.25);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .focus-name {
    font-weight: 600;
    font-size: 15px;
    color: var(--text);
  }

  .focus-desc {
    font-size: 12px;
    color: var(--text-muted);
    margin-top: 2px;
  }

  /* 亲友微互动 */
  .interaction-bar {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px dashed var(--border);
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .interact-label {
    font-size: 12px;
    color: var(--accent);
    font-weight: 600;
  }

  .interact-chip {
    font-size: 12px;
    padding: 6px 10px;
    background: rgba(20, 9, 12, 0.5);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    cursor: pointer;
    transition: all 0.15s;
  }

  .interact-chip:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .interact-muted {
    font-size: 12px;
    color: var(--text-muted);
  }

  /* 岔路口选项列表 */
  .choice-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 16px;
  }

  .choice-card {
    padding: 12px 14px;
    border-radius: 8px;
    border: 1.5px solid var(--border);
    background: var(--card-highlight);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .choice-card:hover {
    border-color: var(--accent);
  }

  .choice-card.selected {
    border-color: var(--accent);
    background: rgba(163, 38, 63, 0.2);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .choice-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 600;
    font-size: 15px;
    color: var(--text);
    margin-bottom: 6px;
  }

  .choice-cost {
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  /* 人生节点反思自省 */
  .prompt-box {
    background: var(--card-highlight);
    border: 1px dashed var(--border);
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 16px;
  }

  .prompt-question {
    font-family: var(--font-serif);
    font-size: 16px;
    font-weight: 700;
    color: var(--accent);
    margin-bottom: 8px;
  }

  .prompt-hint {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 12px;
    line-height: 1.5;
  }

  .quote-input {
    width: 100%;
    min-height: 72px;
    padding: 10px 12px;
    font-family: var(--font-serif);
    font-size: 15px;
    line-height: 1.6;
    background: rgba(20, 9, 12, 0.6);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    resize: vertical;
  }
  @media (prefers-color-scheme: light) {
    .quote-input {
      background: #ffffff;
    }
  }

  .char-count {
    font-size: 12px;
    color: var(--text-muted);
    text-align: right;
    margin-top: 4px;
  }

  .past-quotes {
    border-top: 1px dashed var(--border);
    padding-top: 12px;
    margin-bottom: 16px;
  }

  .past-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: 6px;
  }

  .past-item {
    font-family: var(--font-serif);
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  /* 底部固定操作栏（手机固定在底部，高度 >= 48px；桌面端就地居中） */
  .action-container {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 25;
    background: rgba(20, 9, 12, 0.94);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-top: 1px solid var(--border);
    padding: 12px 16px calc(12px + env(safe-area-inset-bottom, 0px));
  }
  @media (prefers-color-scheme: light) {
    .action-container {
      background: rgba(251, 239, 241, 0.94);
    }
  }

  @media (min-width: 900px) {
    .action-container {
      position: static;
      background: transparent;
      backdrop-filter: none;
      border-top: none;
      padding: 10px 0 0;
    }
  }

  .action-row {
    max-width: 560px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .btn-advance, .btn-action {
    width: 100%;
    min-height: 48px;
    padding: 12px 20px;
    font-size: 16px;
    font-weight: 700;
    font-family: var(--font-sans);
    border-radius: 8px;
    border: none;
    background: var(--primary);
    color: #ffffff;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(163, 38, 63, 0.35);
    transition: all 0.15s ease-in-out;
  }

  .btn-advance:hover, .btn-action:hover {
    background: var(--primary-hover);
  }

  .btn-advance:disabled, .btn-action:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .auto-toggle-row {
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: 13px;
    color: var(--text-muted);
  }

  .auto-switch-label {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  /* 手机端折叠抽屉 */
  .mobile-drawer {
    display: block;
    margin-top: 10px;
  }
  @media (min-width: 900px) {
    .mobile-drawer {
      display: none;
    }
  }

  .mobile-drawer summary {
    cursor: pointer;
    list-style: none;
    padding: 12px 16px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--accent);
    font-size: 13px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .mobile-drawer summary::-webkit-details-marker {
    display: none;
  }

  .mobile-drawer[open] summary {
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
    border-bottom: none;
  }

  .mobile-drawer .drawer-content {
    background: var(--card);
    border: 1px solid var(--border);
    border-top: none;
    border-bottom-left-radius: 8px;
    border-bottom-right-radius: 8px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  /* 桌面端常驻右侧栏 */
  .desktop-sidebar {
    display: none;
  }
  @media (min-width: 900px) {
    .desktop-sidebar {
      display: flex;
      flex-direction: column;
      gap: 16px;
      position: sticky;
      top: 60px;
    }
  }

  .side-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 16px 18px;
  }

  .side-card h3 {
    font-size: 13px;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 10px;
  }

  /* 属性网格 */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }

  .stat-box {
    background: var(--card-highlight);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px;
    text-align: center;
  }

  .stat-name {
    font-size: 11px;
    color: var(--text-muted);
    display: block;
    margin-bottom: 2px;
  }

  .stat-value {
    font-size: 16px;
    font-weight: 700;
    color: var(--text);
  }

  .wealth-info {
    margin-top: 10px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .wealth-info strong {
    color: var(--accent);
  }

  /* 家族与朋辈列表 */
  .family-list, .peer-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .family-item, .peer-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 10px;
    background: rgba(20, 9, 12, 0.4);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 12px;
  }
  @media (prefers-color-scheme: light) {
    .family-item, .peer-item {
      background: #f7e4e7;
    }
  }

  .family-item.deceased, .peer-item.deceased {
    opacity: 0.5;
  }

  /* 早夭移魂提示卡 */
  .switch-peer-card {
    background: var(--card);
    border: 1px solid var(--primary);
    border-radius: 10px;
    padding: 16px;
    margin-top: 10px;
  }

  .switch-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--accent);
    margin-bottom: 6px;
  }

  .switch-desc {
    font-size: 13px;
    color: var(--text);
    margin-bottom: 12px;
    line-height: 1.5;
  }

  .switch-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .btn-peer-switch {
    padding: 8px 14px;
    font-size: 13px;
    background: var(--primary);
    color: #ffffff;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  /* 存档控制 */
  .archive-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }

  .seed-input-row {
    display: flex;
    gap: 6px;
  }

  .input-seed {
    flex: 1;
    padding: 6px 8px;
    font-size: 12px;
    background: rgba(20, 9, 12, 0.5);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text);
  }
  @media (prefers-color-scheme: light) {
    .input-seed {
      background: #ffffff;
    }
  }

  .btn-sm {
    padding: 6px 10px;
    font-size: 12px;
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: 4px;
    cursor: pointer;
  }
  .btn-sm:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .btn-seed {
    background: rgba(163, 38, 63, 0.2);
    border-color: var(--primary);
  }

  /* Loading */
  .loading-screen {
    min-height: 100dvh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 16px;
    color: var(--text-muted);
  }

  .loading-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
