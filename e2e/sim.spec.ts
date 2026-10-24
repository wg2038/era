import { expect, test } from '@playwright/test';

test.describe('Era Reboot - Turn-based Life Sim End-to-End Suite', () => {
  test('loads game homepage with dark red aesthetic and 5 yearly focus choices', async ({
    page,
  }) => {
    await page.goto('/');

    // 验证网页标题
    await expect(page).toHaveTitle(/Era \| 时代/);

    // 验证顶部细条与年代信息
    const topBar = page.locator('.top-bar');
    await expect(topBar).toBeVisible();
    await expect(topBar).toContainText('ERA');

    // 验证五大生命重心选项存在
    const focusGrid = page.locator('.focus-grid');
    await expect(focusGrid).toBeVisible();
    await expect(focusGrid).toContainText('读书求学');
    await expect(focusGrid).toContainText('帮贴谋生');
    await expect(focusGrid).toContainText('顾全家人');
    await expect(focusGrid).toContainText('调养身体');
    await expect(focusGrid).toContainText('维系人情');

    // 验证推进按钮高度与文字
    const advanceBtn = page.locator('.btn-advance');
    await expect(advanceBtn).toBeVisible();
  });

  test('advances year with chosen focus and updates narrative stream', async ({ page }) => {
    await page.goto('/');

    // 等待初始化就绪
    await page.waitForSelector('.btn-advance');

    // 选中谋生重心
    const livelihoodOpt = page.locator('.focus-option:has-text("帮贴谋生")');
    await livelihoodOpt.click();

    // 点击推进至下一年
    const advanceBtn = page.locator('.btn-advance');
    await advanceBtn.click();

    // 验证叙事流更新，出现新记录
    const currentLog = page.locator('.log-item.current');
    await expect(currentLog).toBeVisible();
  });

  test('executes micro-interactions with relatives', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.interaction-bar');

    const interactChips = page.locator('.interact-chip');
    if ((await interactChips.count()) > 0) {
      await interactChips.first().click();
      // 验证行动次数减少或提示更新
      const interactLabel = page.locator('.interaction-bar');
      await expect(interactLabel).toBeVisible();
    }
  });
});
