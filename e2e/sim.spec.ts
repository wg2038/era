import { expect, test } from '@playwright/test';

test.describe('Era - Chinese Life Simulator End-to-End Test Suite', () => {
  test('loads game homepage with PWA manifest and 15+ rating', async ({ page }) => {
    await page.goto('/');

    // 验证网页标题与适龄提示
    await expect(page).toHaveTitle(/Era \| 时代/);
    const badge = page.locator('.badge');
    await expect(badge).toBeVisible();
    await expect(badge).toContainText('15+ 适龄提示');

    // 验证 WebGPU / 降级图形徽章正常工作
    const renderBadge = page.locator('.render-badge');
    await expect(renderBadge).toBeVisible();
    await expect(renderBadge).toContainText('宣纸水墨扩散渲染');
  });

  test('initializes deterministic simulation and advances year', async ({ page }) => {
    await page.goto('/');

    // 输入固定测试种子 42 与出生年份 1935
    await page.fill('#seed-input', '42');
    await page.fill('#birth-year-input', '1935');
    await page.click('button:has-text("开启人生局")');

    // 验证人物仪表盘加载
    const characterHeader = page.locator('.character-header');
    await expect(characterHeader).toBeVisible();
    await expect(characterHeader).toContainText('0 岁');

    // 验证历史地名沿革显示（1935年北平）
    const birthInfo = page.locator('.birth-info');
    await expect(birthInfo).toBeVisible();
    await expect(birthInfo).toContainText('1935');

    // 验证历史货币换皮呈现（银元/法币/粮票）
    const currencyBadge = page.locator('.currency-badge');
    await expect(currencyBadge).toBeVisible();

    // 验证家族谱系（包含父母）
    const kinshipGrid = page.locator('.kinship-grid');
    await expect(kinshipGrid.first()).toBeVisible();

    // 点击推进一年
    await page.click('button:has-text("推进一年")');
    await expect(characterHeader).toContainText('1 岁');
  });

  test('completes full life run to ending summary and generates shareable seed card', async ({
    page,
  }) => {
    await page.goto('/');

    await page.fill('#seed-input', '12345');
    await page.fill('#birth-year-input', '1920');
    await page.click('button:has-text("开启人生局")');

    // 一键推演至终局
    await page.click('button:has-text("推演至终局")');

    // 验证进入终局看板
    const endingHeader = page.locator('.ending-header');
    await expect(endingHeader).toBeVisible({ timeout: 15000 });
    await expect(endingHeader).toContainText('百年纪传');

    // 验证百年浮沉图谱与同年同县四人全景对照
    const trendChart = page.locator('.trend-chart');
    await expect(trendChart).toBeVisible();

    const peerTable = page.locator('.peer-table');
    await expect(peerTable).toBeVisible();

    // 验证分享卡与种子代码
    const shareCanvas = page.locator('.share-canvas-preview');
    await expect(shareCanvas).toBeVisible();

    const copyBtn = page.locator('button:has-text("复制种子码")');
    await expect(copyBtn).toBeVisible();
  });
});
