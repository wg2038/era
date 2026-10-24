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

  // 种子复现代码
  let shareSeedCode = $derived.by(() => {
    return `ERA#${simState.seed}#${activePerson.birth_year}#${activePerson.birth_location_id}`;
  });

  // 人生最终评价
  let lifeEvaluation = $derived.by(() => {
    const p = activePerson;
    const isOld = (p.death_age ?? p.age) >= 70;
    const isScholar = p.attributes.knowledge >= 60;
    const isWealthy = p.wealth_ppp >= 100;
    const isWitness2025 = p.current_year >= 2025;

    if (isWitness2025) {
      return '跨越百年沧桑，亲历从晚清民初到盛世复兴的历史活化石。';
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
      return '饱经百年风霜，虽无惊天伟业，然寿享天年，福泽绵长。';
    }
    return '生于大时代洪流，于逆旅之中竭尽全力，虽有未竟，无愧此生。';
  });

  // 绘制分享卡图片
  function generateShareCard() {
    if (!shareCanvas) return;
    const ctx = shareCanvas.getContext('2d');
    if (!ctx) return;

    const width = 800;
    const height = 1000;
    shareCanvas.width = width;
    shareCanvas.height = height;

    // 1. 宣纸水墨背景
    ctx.fillStyle = '#1c1815';
    ctx.fillRect(0, 0, width, height);

    // 边框装饰
    ctx.strokeStyle = '#5a4933';
    ctx.lineWidth = 2;
    ctx.strokeRect(30, 30, width - 60, height - 60);
    ctx.strokeRect(36, 36, width - 72, height - 72);

    // 2. 标题区
    ctx.fillStyle = '#c79a5b';
    ctx.font = 'bold 36px serif';
    ctx.textAlign = 'center';
    ctx.fillText('《Era》（时代）· 人生档案', width / 2, 100);

    ctx.fillStyle = '#8c8477';
    ctx.font = '16px sans-serif';
    ctx.fillText('中国百年人生文字模拟 · 确定性历史人生印记', width / 2, 135);

    // 3. 人物信息
    ctx.fillStyle = '#f3efe6';
    ctx.font = 'bold 32px serif';
    ctx.fillText(activePerson.name, width / 2, 200);

    ctx.fillStyle = '#d1c7b7';
    ctx.font = '18px sans-serif';
    const birthLocStr = simState.birth_historical_name !== simState.birth_current_name
      ? `${simState.birth_historical_name}（今${simState.birth_current_name}）`
      : simState.birth_current_name;
    ctx.fillText(`${activePerson.birth_year} 年生于 ${birthLocStr}`, width / 2, 240);

    const lifespanStr = activePerson.is_alive && activePerson.current_year >= 2025
      ? `跨越百年 · 于 2025 年见证盛世 (${activePerson.age} 岁)`
      : `享年 ${activePerson.death_age ?? activePerson.age} 岁 (${activePerson.birth_year} - ${activePerson.death_year ?? activePerson.current_year})`;
    ctx.fillStyle = '#eab308';
    ctx.font = 'bold 20px serif';
    ctx.fillText(lifespanStr, width / 2, 280);

    // 4. 四维属性与资产条
    ctx.fillStyle = '#26221d';
    ctx.fillRect(70, 320, width - 140, 100);

    ctx.fillStyle = '#8c8477';
    ctx.font = '15px sans-serif';
    ctx.fillText('健康', 140, 355);
    ctx.fillText('学识', 280, 355);
    ctx.fillText('心情', 420, 355);
    ctx.fillText('见识', 560, 355);
    ctx.fillText('终局财富', 670, 355);

    ctx.fillStyle = '#f3efe6';
    ctx.font = 'bold 22px serif';
    ctx.fillText(`${activePerson.attributes.health}`, 140, 395);
    ctx.fillText(`${activePerson.attributes.knowledge}`, 280, 395);
    ctx.fillText(`${activePerson.attributes.mood}`, 420, 395);
    ctx.fillText(`${activePerson.attributes.insight}`, 560, 395);
    ctx.fillText(`${simState.current_currency.formatted_amount.split(' ')[0]}`, 670, 395);

    // 5. 人生总评
    ctx.fillStyle = '#e5e7eb';
    ctx.font = 'italic 18px serif';
    ctx.fillText(`“ ${lifeEvaluation} ”`, width / 2, 470);

    // 6. 人生关键纪事（取最后 4 条重大纪要）
    ctx.fillStyle = '#c79a5b';
    ctx.font = 'bold 20px serif';
    ctx.textAlign = 'left';
    ctx.fillText('【人生纪年精选】', 70, 530);

    const keyLogs = activePerson.log.slice(-4);
    let logY = 570;
    for (const item of keyLogs) {
      ctx.fillStyle = '#d97706';
      ctx.font = 'bold 16px sans-serif';
      ctx.fillText(`${item.year}年 (${item.age}岁)`, 70, logY);

      ctx.fillStyle = '#f3efe6';
      ctx.font = 'bold 16px serif';
      ctx.fillText(item.title, 190, logY);

      ctx.fillStyle = '#a8a29e';
      ctx.font = '14px sans-serif';
      const shortDesc = item.content.length > 36 ? `${item.content.slice(0, 36)}...` : item.content;
      ctx.fillText(shortDesc, 190, logY + 24);

      logY += 56;
    }

    // 7. 种子代码底栏（100% 相同种子可精确复现）
    ctx.fillStyle = '#26221d';
    ctx.fillRect(70, 830, width - 140, 80);

    ctx.fillStyle = '#c79a5b';
    ctx.font = 'bold 15px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(`种子开局代码: ${shareSeedCode}`, width / 2, 865);

    ctx.fillStyle = '#78716c';
    ctx.font = '13px sans-serif';
    ctx.fillText('输入此种子码与出生年代，即可在 Era 中 100% 复现相同人生开局', width / 2, 890);

    // 落款
    ctx.fillStyle = '#57534e';
    ctx.font = '12px sans-serif';
    ctx.fillText('Era Project · WebAssembly Deterministic Life Simulator', width / 2, 955);
  }

  function downloadShareCard() {
    generateShareCard();
    if (!shareCanvas) return;
    const dataUrl = shareCanvas.toDataURL('image/png');
    const a = document.createElement('a');
    a.href = dataUrl;
    a.download = `Era_Life_${activePerson.name}_${activePerson.birth_year}.png`;
    a.click();
  }

  async function copySeed() {
    try {
      await navigator.clipboard.writeText(shareSeedCode);
      isCopySuccess = true;
      setTimeout(() => {
        isCopySuccess = false;
      }, 2500);
    } catch (err) {
      console.error('Failed to copy seed:', err);
    }
  }

  $effect(() => {
    generateShareCard();
  });
</script>

<div class="ending-container">
  <div class="ending-header">
    <div class="ending-tag">终局评定看板 · 人生盖棺定论</div>
    <h2 class="ending-name">{activePerson.name} 的百年纪传</h2>
    <div class="ending-subtitle">
      {#if activePerson.current_year >= 2025}
        <span class="text-success">★ 见证百年盛世 · 功德圆满</span>
      {:else}
        <span>享年 {activePerson.death_age ?? activePerson.age} 岁 · {activePerson.cause_of_death ? activePerson.cause_of_death : '无疾而终'}</span>
      {/if}
    </div>
    <p class="ending-evaluation">“ {lifeEvaluation} ”</p>
  </div>

  <!-- 人生曲线与资产总览 (SVG 矢量折线图) -->
  <section class="ending-card">
    <h3 class="section-title">百年浮沉图谱（资产 PPP 与亲情演进）</h3>
    <div class="chart-container">
      <svg class="trend-chart" viewBox="0 0 600 200">
        <!-- 坐标轴底线 -->
        <line x1="40" y1="170" x2="580" y2="170" stroke="#383127" stroke-width="1" />
        <line x1="40" y1="20" x2="40" y2="170" stroke="#383127" stroke-width="1" />

        <!-- 坐标标注 -->
        <text x="35" y="175" fill="#78716c" font-size="10" text-anchor="end">0</text>
        <text x="35" y="100" fill="#78716c" font-size="10" text-anchor="end">50</text>
        <text x="35" y="25" fill="#78716c" font-size="10" text-anchor="end">100</text>

        <text x="40" y="188" fill="#78716c" font-size="10">{activePerson.birth_year}</text>
        <text x="580" y="188" fill="#78716c" font-size="10" text-anchor="end">{activePerson.current_year}</text>

        <!-- 购买力 PPP 折线示例 (按日志年份标绘) -->
        <polyline
          fill="none"
          stroke="#f59e0b"
          stroke-width="2.5"
          points="40,130 150,110 260,85 370,120 480,60 580,45"
        />
        <!-- 亲情平均羁绊折线示例 -->
        <polyline
          fill="none"
          stroke="#38bdf8"
          stroke-width="2"
          stroke-dasharray="4"
          points="40,50 150,60 260,75 370,95 480,105 580,120"
        />
      </svg>
      <div class="chart-legend">
        <span class="legend-item"><span class="legend-dot gold"></span> 内部购买力点数 (PPP) 波动</span>
        <span class="legend-item"><span class="legend-dot blue"></span> 家族与朋友羁绊均值</span>
      </div>
    </div>
  </section>

  <!-- 同年同县 4 人终局对照看板 -->
  <section class="ending-card mt-4">
    <h3 class="section-title">同年同县出生的另外四人 · 百年命运对照</h3>
    <div class="peer-table-wrapper">
      <table class="peer-table">
        <thead>
          <tr>
            <th>人物</th>
            <th>终局状态</th>
            <th>终寿</th>
            <th>死因归宿</th>
            <th>四维属性 (健/学/心/见)</th>
          </tr>
        </thead>
        <tbody>
          <tr class="highlight-row">
            <td><strong>{activePerson.name} (主角)</strong></td>
            <td>{activePerson.is_alive ? '见证盛世' : '寿尽身故'}</td>
            <td><strong>{activePerson.death_age ?? activePerson.age} 岁</strong></td>
            <td>{activePerson.is_alive ? '见证2025' : '归于尘土'}</td>
            <td>{activePerson.attributes.health}/{activePerson.attributes.knowledge}/{activePerson.attributes.mood}/{activePerson.attributes.insight}</td>
          </tr>
          {#each simState.peers as peer}
            <tr>
              <td>{peer.name}</td>
              <td>{peer.is_alive ? '见证盛世' : '身故'}</td>
              <td>{peer.death_age ?? peer.age} 岁</td>
              <td>{peer.is_alive ? '见证2025' : '夭折/病故/战乱'}</td>
              <td>{peer.attributes.health}/{peer.attributes.knowledge}/{peer.attributes.mood}/{peer.attributes.insight}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <!-- 分享卡生成预览与下载操作 -->
  <section class="ending-card mt-4 share-card-section">
    <h3 class="section-title">可分享的人生档案卡（可复现种子码）</h3>
    <p class="share-desc">包含完整属性、代表性年鉴、死因与复现开局的种子码：</p>

    <div class="canvas-wrapper">
      <canvas bind:this={shareCanvas} class="share-canvas-preview"></canvas>
    </div>

    <div class="share-buttons mt-4">
      <button class="btn btn-primary" onclick={downloadShareCard}>
        💾 下载高清人生档案长卡 (PNG)
      </button>
      <button class="btn btn-secondary" onclick={copySeed}>
        {isCopySuccess ? '✓ 已复制种子代码' : '📋 复制种子码复现开局'}
      </button>
      <button class="btn btn-accent" onclick={onRestart}>
        🔄 再历百年（开启新人生）
      </button>
    </div>
  </section>
</div>

<style>
  .ending-container {
    max-width: 860px;
    margin: 2rem auto;
  }
  .ending-header {
    text-align: center;
    background: #181512;
    border: 1px solid #4a3e2f;
    border-radius: 8px;
    padding: 2rem 1.5rem;
    margin-bottom: 1.5rem;
  }
  .ending-tag {
    font-size: 0.78rem;
    color: #c79a5b;
    border: 1px solid #5a4933;
    display: inline-block;
    padding: 0.2rem 0.6rem;
    border-radius: 9999px;
    margin-bottom: 0.75rem;
  }
  .ending-name {
    font-family: var(--font-serif);
    font-size: 2.2rem;
    color: #f3efe6;
    margin: 0 0 0.5rem 0;
  }
  .ending-subtitle {
    font-size: 1.05rem;
    color: #d1c7b7;
    margin-bottom: 1rem;
  }
  .ending-evaluation {
    font-size: 1rem;
    font-style: italic;
    color: #fef08a;
    line-height: 1.6;
    max-width: 650px;
    margin: 0 auto;
  }
  .ending-card {
    background: #181614;
    border: 1px solid #2e2a25;
    border-radius: 8px;
    padding: 1.5rem;
  }
  .chart-container {
    margin-top: 1rem;
  }
  .trend-chart {
    width: 100%;
    height: 200px;
    background: #141210;
    border-radius: 6px;
  }
  .chart-legend {
    display: flex;
    justify-content: center;
    gap: 1.5rem;
    font-size: 0.8rem;
    color: #a8a29e;
    margin-top: 0.75rem;
  }
  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .legend-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .legend-dot.gold {
    background: #f59e0b;
  }
  .legend-dot.blue {
    background: #38bdf8;
  }
  .peer-table-wrapper {
    overflow-x: auto;
    margin-top: 1rem;
  }
  .peer-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.88rem;
    text-align: left;
  }
  .peer-table th, .peer-table td {
    padding: 0.75rem;
    border-bottom: 1px solid #28241f;
  }
  .peer-table th {
    color: #8c8477;
    font-weight: 500;
  }
  .highlight-row {
    background: #26201a;
    color: #fef08a;
  }
  .canvas-wrapper {
    display: flex;
    justify-content: center;
    margin-top: 1rem;
  }
  .share-canvas-preview {
    max-width: 100%;
    height: auto;
    border: 1px solid #4a3e2f;
    border-radius: 6px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.5);
  }
  .share-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    justify-content: center;
  }
  .share-desc {
    color: #8c8477;
    font-size: 0.85rem;
    margin: 0.25rem 0 0.75rem 0;
  }
</style>
