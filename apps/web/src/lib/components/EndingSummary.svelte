<script lang="ts">
  import type { SimulationState } from '@era/era-core';

  interface Props {
    simState: SimulationState;
    onRestart: () => void;
  }

  let { simState, onRestart }: Props = $props();

  let activePerson = $derived.by(() => {
    return simState.active_index === 0
      ? simState.protagonist
      : simState.peers[simState.active_index - 1] ?? simState.protagonist;
  });

  let shareCanvas: HTMLCanvasElement | null = $state(null);
  let isCopySuccess: boolean = $state(false);
  let farewellMessage: string = $state('走好，这一生很长也很苦，但你没有退缩，辛苦了。');

  // 种子复现代码
  let shareSeedCode = $derived.by(() => {
    return `ERA#${simState.seed}#${activePerson.birth_year}#${activePerson.birth_location_id}`;
  });

  // 人生最终评价
  let lifeEvaluation = $derived.by(() => {
    const p = activePerson;
    const isOld = (p.death_age ?? p.age) >= 70;
    const isScholar = p.attributes.knowledge >= 60;
    const isWealthy = p.wealth_ppp >= 80;
    const isWitness2025 = p.current_year >= 2025;

    if (isWitness2025) {
      return '跨越百年沧桑，亲历从晚清民国到盛世复兴的历史活化石。';
    }
    if (isScholar && isWealthy) {
      return '儒商兼修，敏锐洞察时代先机，在巨变浪潮中立业立身。';
    }
    if (isScholar) {
      return '尚儒笃学，以笔墨与见识见证时代风云，文骨风雅流传后世。';
    }
    if (isWealthy) {
      return '敢为人先，下海弄潮，凭务实胆魄累积家业，荫庇后代。';
    }
    if (isOld) {
      return '饱经时代风霜，虽无惊天伟业，然寿享天年，福泽绵长。';
    }
    return '生于大时代洪流，于逆旅之中竭尽全力，虽有未竟，无愧此生。';
  });

  function getCauseDescription(peer: typeof activePerson): string {
    if (peer.is_alive && peer.current_year >= 2025) {
      return '平安跨越百年，见证盛世复兴。';
    }
    if (!peer.cause_of_death) {
      return '寿终天年，平淡安详。';
    }
    const c = peer.cause_of_death;
    if (typeof c === 'string') {
      if (c === 'OldAge') return '老衰无疾，享尽天年。';
      if (c === 'SurviveTo2025') return '见证时代至今。';
      return c;
    }
    if ('InfantMortality' in c) return `襁褓早夭：${c.InfantMortality}`;
    if ('ChildMortality' in c) return `幼年夭折：${c.ChildMortality}`;
    if ('Famine' in c) return `遭遇灾荒：${c.Famine}`;
    if ('WartimeCasualty' in c) return `战火兵燹：${c.WartimeCasualty}`;
    if ('Epidemic' in c) return `染疾不治：${c.Epidemic}`;
    if ('Accident' in c) return `意外横祸：${c.Accident}`;
    if ('OrganDisease' in c) return `病逝：${c.OrganDisease}`;
    return '不幸离世。';
  }

  function generateShareCard() {
    if (!shareCanvas) return;
    const ctx = shareCanvas.getContext('2d');
    if (!ctx) return;

    const width = 800;
    const height = 1050;
    shareCanvas.width = width;
    shareCanvas.height = height;

    // 1. 深黛底与暗红渐变
    ctx.fillStyle = '#14090C';
    ctx.fillRect(0, 0, width, height);

    const grad = ctx.createRadialGradient(width / 2, 0, 10, width / 2, 0, 600);
    grad.addColorStop(0, 'rgba(163, 38, 63, 0.35)');
    grad.addColorStop(1, 'rgba(20, 9, 12, 0)');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, width, height);

    // 边框装饰
    ctx.strokeStyle = '#4A2430';
    ctx.lineWidth = 2;
    ctx.strokeRect(32, 32, width - 64, height - 64);
    ctx.strokeStyle = 'rgba(242, 167, 184, 0.3)';
    ctx.lineWidth = 1;
    ctx.strokeRect(38, 38, width - 76, height - 76);

    // 2. 标题区
    ctx.fillStyle = '#F2A7B8';
    ctx.font = 'bold 34px serif';
    ctx.textAlign = 'center';
    ctx.fillText('《Era》（时代）· 百年纪传', width / 2, 100);

    ctx.fillStyle = '#B9959D';
    ctx.font = '15px sans-serif';
    ctx.fillText('中国百年人生文字模拟 · 确定性历史生命档案', width / 2, 135);

    // 3. 人物姓名与生死
    ctx.fillStyle = '#F6E9EC';
    ctx.font = 'bold 32px serif';
    ctx.fillText(activePerson.name, width / 2, 195);

    ctx.fillStyle = '#B9959D';
    ctx.font = '16px sans-serif';
    const birthLocStr = simState.birth_historical_name !== simState.birth_current_name
      ? `${simState.birth_historical_name}（今${simState.birth_current_name}）`
      : simState.birth_current_name;
    ctx.fillText(`${activePerson.birth_year} 年生于 ${birthLocStr}`, width / 2, 230);

    const lifespanStr = activePerson.is_alive && activePerson.current_year >= 2025
      ? `享年 ${activePerson.age} 岁 · 于 2025 年见证新世纪盛世`
      : `享年 ${activePerson.death_age ?? activePerson.age} 岁 (${activePerson.birth_year} - ${activePerson.death_year ?? activePerson.current_year})`;
    ctx.fillStyle = '#A3263F';
    ctx.font = 'bold 20px serif';
    ctx.fillText(lifespanStr, width / 2, 268);

    // 4. 四维属性面板
    ctx.fillStyle = '#221015';
    ctx.fillRect(60, 305, width - 120, 85);
    ctx.strokeStyle = '#4A2430';
    ctx.strokeRect(60, 305, width - 120, 85);

    ctx.fillStyle = '#B9959D';
    ctx.font = '14px sans-serif';
    ctx.fillText('健康', 130, 335);
    ctx.fillText('学识', 270, 335);
    ctx.fillText('心情', 410, 335);
    ctx.fillText('见识', 550, 335);
    ctx.fillText('购买力', 670, 335);

    ctx.fillStyle = '#F6E9EC';
    ctx.font = 'bold 22px serif';
    ctx.fillText(`${activePerson.attributes.health}`, 130, 368);
    ctx.fillText(`${activePerson.attributes.knowledge}`, 270, 368);
    ctx.fillText(`${activePerson.attributes.mood}`, 410, 368);
    ctx.fillText(`${activePerson.attributes.insight}`, 550, 368);
    ctx.fillText(`${activePerson.wealth_ppp} PPP`, 670, 368);

    // 5. 人生总评
    ctx.fillStyle = '#F8D7DE';
    ctx.font = 'italic 17px serif';
    ctx.fillText(`“ ${lifeEvaluation} ”`, width / 2, 430);

    // 6. 人生关键节点心声（展示最多 3 条）
    ctx.fillStyle = '#F2A7B8';
    ctx.font = 'bold 18px serif';
    ctx.textAlign = 'left';
    ctx.fillText('【岁月回声 · 节点留痕】', 60, 480);

    const quotes = activePerson.milestone_quotes.slice(0, 3);
    let quoteY = 515;
    if (quotes.length === 0) {
      ctx.fillStyle = '#B9959D';
      ctx.font = '14px sans-serif';
      ctx.fillText('早年颠沛，未及至留白之期。', 60, quoteY);
      quoteY += 40;
    } else {
      for (const q of quotes) {
        ctx.fillStyle = '#A3263F';
        ctx.font = 'bold 14px sans-serif';
        ctx.fillText(`${q.age} 岁：`, 60, quoteY);

        ctx.fillStyle = '#F6E9EC';
        ctx.font = 'italic 15px serif';
        const text = q.quote.length > 38 ? `${q.quote.slice(0, 38)}...` : q.quote;
        ctx.fillText(`“${text}”`, 130, quoteY);
        quoteY += 38;
      }
    }

    // 7. 道别寄语
    ctx.fillStyle = '#F2A7B8';
    ctx.font = 'bold 18px serif';
    ctx.fillText('【最后道别】', 60, quoteY + 20);

    ctx.fillStyle = '#F6E9EC';
    ctx.font = '15px serif';
    const farewellShort = farewellMessage.length > 40 ? `${farewellMessage.slice(0, 40)}...` : farewellMessage;
    ctx.fillText(`“${farewellShort}”`, 60, quoteY + 52);

    // 8. 底部复现码
    ctx.fillStyle = '#4A2430';
    ctx.fillRect(60, height - 120, width - 120, 50);
    ctx.fillStyle = '#F2A7B8';
    ctx.font = '13px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(shareSeedCode, width / 2, height - 90);
  }

  function copySeedCode() {
    navigator.clipboard.writeText(shareSeedCode).then(() => {
      isCopySuccess = true;
      setTimeout(() => (isCopySuccess = false), 2500);
    });
  }
</script>

<div class="ending-container">
  <article class="summary-card">
    <div class="epitaph-header">ERA · 盖棺定论</div>
    <h1 class="epitaph-name">{activePerson.name} 之生平</h1>
    <div class="epitaph-span">
      {activePerson.birth_year} 年生于 {simState.birth_historical_name} ·
      {activePerson.is_alive && activePerson.current_year >= 2025
        ? '跨越百年 · 见证新世纪盛世'
        : `逝于 ${activePerson.death_year ?? activePerson.current_year} 年（${activePerson.death_age ?? activePerson.age} 岁）`}
    </div>

    <div class="narrative-body">
      <p>
        生于风云激荡的近代中国，在宏大的二十世纪波澜中留下了坚实温热的脚印。
      </p>
      <p>
        {lifeEvaluation}
      </p>
    </div>

    <!-- 向他道一句最后的告别 -->
    <div class="farewell-box">
      <div class="farewell-title">向他道一句最后的告别：</div>
      <textarea
        class="farewell-input"
        rows="2"
        bind:value={farewellMessage}
        placeholder="写下你对这位百年生灵的最后道别……"
      ></textarea>
    </div>

    <!-- 岁月留痕 · 你在人生关键节点的心声 -->
    {#if activePerson.milestone_quotes.length > 0}
      <div class="memory-section">
        <div class="memory-title">岁月留痕 · 你在人生关键节点的心声</div>
        <div class="memory-list">
          {#each activePerson.milestone_quotes as mq}
            <div class="memory-card">
              <div class="age">{mq.age} 岁（{mq.year} 年）· {mq.question}</div>
              <div>“{mq.quote}”</div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </article>

  <!-- 同年同县出生的另外四人并排对比（无分数、无排名） -->
  <section class="peers-overview-card">
    <div class="peers-overview-header">
      同年同县（{simState.birth_historical_name}·{activePerson.birth_year}）平行命运对照
    </div>
    <div class="peers-overview-sub">
      同一时刻降生在同一方水土的生灵，各自被时代的浪潮抛向了何方（无排名、无计分）：
    </div>

    <div class="peer-grid">
      <!-- 主角卡片 -->
      <div class="peer-destiny-card protagonist">
        <div class="peer-name">
          <span>{activePerson.name}（主角）</span>
          <span class="peer-status-tag">{activePerson.death_age ?? activePerson.age} 岁</span>
        </div>
        <div class="peer-life-span">
          {activePerson.birth_year} – {activePerson.death_year ?? activePerson.current_year}
        </div>
        <div class="peer-story">{getCauseDescription(activePerson)}</div>
      </div>

      <!-- 同行 4 人卡片 -->
      {#each simState.peers as peer, idx}
        <div class="peer-destiny-card">
          <div class="peer-name">
            <span>{peer.name}</span>
            <span class="peer-status-tag">{peer.death_age ?? peer.age} 岁</span>
          </div>
          <div class="peer-life-span">
            {peer.birth_year} – {peer.death_year ?? peer.current_year}
          </div>
          <div class="peer-story">{getCauseDescription(peer)}</div>
        </div>
      {/each}
    </div>
  </section>

  <!-- 分享卡与重启控制 -->
  <section class="action-footer">
    <div class="footer-btn-row">
      <button class="btn btn-primary" onclick={onRestart} type="button">
        开启全新人生轮回
      </button>
      <button class="btn btn-outline" onclick={generateShareCard} type="button">
        生成纪传分享卡
      </button>
      <button class="btn btn-outline" onclick={copySeedCode} type="button">
        {isCopySuccess ? '✓ 已复制种子代码' : '复制人生种子代码'}
      </button>
    </div>

    <!-- 离屏生成 Canvas -->
    <div class="canvas-wrapper">
      <canvas bind:this={shareCanvas}></canvas>
    </div>
  </section>
</div>

<style>
  .ending-container {
    width: 100%;
    max-width: 920px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 20px 16px 120px;
  }

  .summary-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.3);
  }

  .epitaph-header {
    font-size: 12px;
    color: var(--accent);
    letter-spacing: 0.1em;
    margin-bottom: 8px;
    font-weight: 600;
  }

  h1.epitaph-name {
    font-family: var(--font-serif);
    font-size: clamp(24px, 4vw, 30px);
    line-height: 1.3;
    color: var(--text);
    margin-bottom: 12px;
  }

  .epitaph-span {
    display: inline-block;
    font-size: 13px;
    color: var(--text-muted);
    background: var(--card-highlight);
    border: 1px solid var(--border);
    padding: 4px 10px;
    border-radius: 6px;
    margin-bottom: 18px;
  }

  .narrative-body {
    font-family: var(--font-serif);
    font-size: 17px;
    line-height: 1.8;
    color: var(--text);
    margin-bottom: 20px;
  }

  .narrative-body p + p {
    margin-top: 12px;
  }

  .farewell-box {
    background: var(--card-highlight);
    border: 1.5px solid var(--border);
    border-radius: 10px;
    padding: 16px;
    margin-bottom: 20px;
  }

  .farewell-title {
    font-family: var(--font-serif);
    font-size: 15px;
    color: var(--accent);
    font-weight: 700;
    margin-bottom: 8px;
  }

  .farewell-input {
    width: 100%;
    min-height: 64px;
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
    .farewell-input {
      background: #ffffff;
    }
  }

  .memory-section {
    border-top: 1px dashed var(--border);
    padding-top: 18px;
  }

  .memory-title {
    font-size: 14px;
    font-weight: 700;
    color: var(--accent);
    margin-bottom: 12px;
  }

  .memory-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .memory-card {
    padding: 10px 14px;
    background: rgba(20, 9, 12, 0.4);
    border-left: 3px solid var(--primary);
    border-radius: 6px;
    font-family: var(--font-serif);
    font-size: 14px;
    line-height: 1.6;
  }

  @media (prefers-color-scheme: light) {
    .memory-card {
      background: #f7e4e7;
    }
  }

  .memory-card .age {
    font-size: 12px;
    font-family: var(--font-sans);
    color: var(--accent);
    font-weight: 600;
    margin-bottom: 3px;
  }

  /* 同龄四人平行对照 */
  .peers-overview-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 22px;
  }

  .peers-overview-header {
    font-size: 16px;
    font-family: var(--font-serif);
    font-weight: 700;
    color: var(--text);
    margin-bottom: 6px;
  }

  .peers-overview-sub {
    font-size: 13px;
    color: var(--text-muted);
    margin-bottom: 16px;
    line-height: 1.5;
  }

  .peer-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 12px;
  }

  @media (min-width: 600px) {
    .peer-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  .peer-destiny-card {
    background: var(--card-highlight);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .peer-destiny-card.protagonist {
    border-color: var(--accent);
    background: rgba(163, 38, 63, 0.2);
  }

  .peer-name {
    font-weight: 700;
    font-size: 15px;
    color: var(--text);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .peer-status-tag {
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(163, 38, 63, 0.2);
    color: var(--accent);
  }

  .peer-life-span {
    font-size: 12px;
    color: var(--accent-soft);
  }

  .peer-story {
    font-size: 13px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  /* 底部按钮与分享 */
  .action-footer {
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-items: center;
  }

  .footer-btn-row {
    width: 100%;
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    justify-content: center;
  }

  .btn {
    min-height: 48px;
    padding: 12px 24px;
    font-size: 15px;
    font-weight: 600;
    font-family: var(--font-sans);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease-in-out;
  }

  .btn-primary {
    background: var(--primary);
    color: #ffffff;
    border: none;
    box-shadow: 0 4px 14px rgba(163, 38, 63, 0.35);
  }

  .btn-primary:hover {
    background: var(--primary-hover);
  }

  .btn-outline {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--accent);
  }

  .btn-outline:hover {
    background: var(--card-highlight);
  }

  .canvas-wrapper canvas {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    border: 1px solid var(--border);
    margin-top: 16px;
  }
</style>
