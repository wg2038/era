import { chromium } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const prototypeDir = path.join(rootDir, 'prototype');
const shotsDir = path.join(prototypeDir, 'shots');

if (!fs.existsSync(shotsDir)) {
  fs.mkdirSync(shotsDir, { recursive: true });
}

const prototypes = [
  '01-birth.html',
  '02-annual.html',
  '03-crossroads.html',
  '04-milestone.html',
  '05-ending.html',
];

const viewports = [
  { name: '360x640', width: 360, height: 640 },
  { name: '390x844', width: 390, height: 844 },
  { name: '768x1024', width: 768, height: 1024 },
  { name: '1280x800', width: 1280, height: 800 },
  { name: '1920x1080', width: 1920, height: 1080 },
];

async function run() {
  const customLdPath = '/home/cicada/.local/lib/playwright-deps/usr/lib/aarch64-linux-gnu';
  process.env.LD_LIBRARY_PATH = customLdPath + (process.env.LD_LIBRARY_PATH ? ':' + process.env.LD_LIBRARY_PATH : '');
  process.env.FONTCONFIG_PATH = '/home/cicada/.local/lib/playwright-deps/etc/fonts';

  const browser = await chromium.launch({
    headless: true,
    args: ['--disable-gpu', '--no-sandbox'],
    env: {
      ...process.env,
      LD_LIBRARY_PATH: process.env.LD_LIBRARY_PATH,
      FONTCONFIG_PATH: process.env.FONTCONFIG_PATH,
    },
  });

  console.log('Browser launched successfully. Capturing prototype screenshots...');

  for (const proto of prototypes) {
    const protoPath = 'file://' + path.join(prototypeDir, proto);
    const baseName = proto.replace('.html', '');

    for (const vp of viewports) {
      const context = await browser.newContext({
        viewport: { width: vp.width, height: vp.height },
        deviceScaleFactor: 1,
      });

      const page = await context.newPage();
      await page.goto(protoPath, { waitUntil: 'load' });

      // 检查是否有异常横向溢出
      const hasHorizontalScroll = await page.evaluate(() => {
        return document.documentElement.scrollWidth > document.documentElement.clientWidth;
      });

      if (hasHorizontalScroll) {
        console.warn(`[WARNING] Horizontal scroll detected in ${proto} at ${vp.name}`);
      }

      const shotPath = path.join(shotsDir, `${baseName}-${vp.name}.png`);
      await page.screenshot({ path: shotPath, fullPage: false });
      console.log(`Saved screenshot: ${baseName}-${vp.name}.png (overflow: ${hasHorizontalScroll})`);

      await context.close();
    }
  }

  await browser.close();
  console.log('All screenshots captured successfully in prototype/shots/');
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
