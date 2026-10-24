import * as v from 'valibot';

/**
 * 置信度评级 (Confidence Rating)
 * A: 确凿普查/公报数据
 * B: 学术学术推算与重构估算
 * C: 模糊区间或邻近推导
 */
export const ConfidenceSchema = v.enum({
  A: 'A',
  B: 'B',
  C: 'C',
});
export type Confidence = v.InferOutput<typeof ConfidenceSchema>;

/**
 * 历史地名沿革区间定义
 */
export const HistoricalNameEntrySchema = v.object({
  startYear: v.number(),
  endYear: v.number(),
  name: v.string(),
  note: v.optional(v.string()),
});
export type HistoricalNameEntry = v.InferOutput<typeof HistoricalNameEntrySchema>;

/**
 * 历史地点实体定义
 */
export const LocationSchema = v.object({
  id: v.string(),
  currentName: v.string(),
  region: v.string(), // 华北 / 华东 / 华南 / 华中 / 西南 / 西北 / 东北
  populationWeight: v.record(v.string(), v.number()), // "1900": 1.2, "1953": 1.8...
  historicalNames: v.array(HistoricalNameEntrySchema),
  climate: v.enum({
    temperate: 'temperate',
    subtropical: 'subtropical',
    tropical: 'tropical',
    arid: 'arid',
    cold: 'cold',
  }),
  source: v.string(),
  confidence: ConfidenceSchema,
});
export type Location = v.InferOutput<typeof LocationSchema>;

/**
 * 属性变更定义 (健康/学识/心情/见识/购买力)
 */
export const AttributeEffectsSchema = v.object({
  health: v.optional(v.number()),
  knowledge: v.optional(v.number()),
  mood: v.optional(v.number()),
  insight: v.optional(v.number()),
  wealthPPP: v.optional(v.number()),
});
export type AttributeEffects = v.InferOutput<typeof AttributeEffectsSchema>;

/**
 * 延迟后果生效项
 */
export const DelayedEffectSchema = v.object({
  delayYears: v.number(),
  effects: AttributeEffectsSchema,
  message: v.string(),
});
export type DelayedEffect = v.InferOutput<typeof DelayedEffectSchema>;

/**
 * 人生层选项定义
 */
export const ChoiceOptionSchema = v.object({
  id: v.string(),
  text: v.string(),
  effects: AttributeEffectsSchema,
  delayedEffects: v.optional(v.array(DelayedEffectSchema)),
  risk: v.optional(v.number()), // 0.0 - 1.0
  source: v.optional(v.string()),
});
export type ChoiceOption = v.InferOutput<typeof ChoiceOptionSchema>;

/**
 * 事件层级枚举
 */
export const EventTierSchema = v.enum({
  destiny: 'destiny', // 命运层
  era: 'era', // 时代层
  life: 'life', // 人生层
});
export type EventTier = v.InferOutput<typeof EventTierSchema>;

/**
 * 事件触发条件
 */
export const TriggerConditionsSchema = v.object({
  minYear: v.optional(v.number()),
  maxYear: v.optional(v.number()),
  minAge: v.optional(v.number()),
  maxAge: v.optional(v.number()),
  regions: v.optional(v.array(v.string())),
  requiresTrait: v.optional(v.array(v.string())),
});
export type TriggerConditions = v.InferOutput<typeof TriggerConditionsSchema>;

/**
 * 事件卡片全量定义
 */
export const EventCardSchema = v.object({
  id: v.string(),
  tier: EventTierSchema,
  title: v.string(),
  narrative: v.string(),
  triggers: TriggerConditionsSchema,
  choices: v.optional(v.array(ChoiceOptionSchema)), // 人生层通常带 2-4 个选项；命运与时代层可无选项直接结算
  effects: v.optional(AttributeEffectsSchema),
  delayedEffects: v.optional(v.array(DelayedEffectSchema)),
  source: v.string(),
  confidence: ConfidenceSchema,
});
export type EventCard = v.InferOutput<typeof EventCardSchema>;

/**
 * 生命表年份区间死因与粗死亡率
 */
export const LifeTableDecadeSchema = v.object({
  decade: v.number(), // 1900, 1910, ..., 1990
  infantMortalityRate: v.number(), // 0.0 - 1.0
  childMortalityRate: v.number(), // 1-5岁死亡率
  baseCrudeDeathRate: v.number(), // 粗死亡率 (‰)
  lifeExpectancy: v.number(),
  source: v.string(),
  confidence: ConfidenceSchema,
});
export type LifeTableDecade = v.InferOutput<typeof LifeTableDecadeSchema>;
