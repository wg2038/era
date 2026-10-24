import { compiledData } from '@era/data/dist/index.js';

export type PlaySpeedMode = 'normal' | 'fast' | 'instant';

export interface PacingConfig {
  mode: PlaySpeedMode;
  speedMultiplier: number; // 1x, 2x, 5x, 10x
}

/**
 * 依据年代生命表计算预期寿命，自适应推导“每年秒数 = 总预算 ÷ 预计人生年数”
 * - 完整游玩：总时间预算 30 分钟（1800 秒）
 * - 快进模式：总时间预算 5 分钟（300 秒）
 */
export function calculateAdaptiveIntervalMs(
  birthYear: number,
  mode: PlaySpeedMode,
  multiplier = 1,
): number {
  if (mode === 'instant') return 0;

  // 1. 查询该年代基准预期寿命
  const decade = Math.floor(Math.max(1900, Math.min(1990, birthYear)) / 10) * 10;
  const table = compiledData.lifeTables.find((t) => t.decade === decade);
  const expectedLifeYears = table?.lifeExpectancy ?? 45;

  // 2. 总时间预算分配
  const totalBudgetSeconds = mode === 'fast' ? 300 : 1800;

  // 3. 每年秒数 = 总预算 ÷ 预期人生年数
  const secondsPerYear = totalBudgetSeconds / expectedLifeYears;

  // 4. 结合倍速乘数，计算毫秒间隔（设置合理交互上下限：500ms ~ 30000ms）
  const ms = (secondsPerYear * 1000) / Math.max(1, multiplier);
  return Math.max(400, Math.min(60000, Math.round(ms)));
}

/**
 * 获取用于 UI 呈现的节奏说明文案
 */
export function getPacingDescription(
  birthYear: number,
  mode: PlaySpeedMode,
  multiplier = 1,
): { secondsPerYear: number; label: string } {
  const decade = Math.floor(Math.max(1900, Math.min(1990, birthYear)) / 10) * 10;
  const table = compiledData.lifeTables.find((t) => t.decade === decade);
  const expectedLifeYears = table?.lifeExpectancy ?? 45;

  const totalBudgetSeconds = mode === 'fast' ? 300 : 1800;
  const rawSeconds = totalBudgetSeconds / expectedLifeYears;
  const effectiveSeconds = rawSeconds / Math.max(1, multiplier);

  const label =
    mode === 'fast'
      ? `极速快进 · 每年约 ${effectiveSeconds.toFixed(1)} 秒 (总预算 5 分钟 · 仅关键岔口暂停)`
      : `沉浸沉思 · 每年约 ${effectiveSeconds.toFixed(1)} 秒 (自适应总预算 30 分钟)`;

  return {
    secondsPerYear: effectiveSeconds,
    label,
  };
}
