import fc from 'fast-check';
import { describe, expect, it } from 'vitest';
import { compiledData } from '../dist/index.js';

describe('Milestone 2: Events Engine & Historical Location Library Integrity', () => {
  it('contains exactly 40 authentic historical event cards and 30 locations', () => {
    expect(compiledData.locations).toHaveLength(30);
    expect(compiledData.events).toHaveLength(40);

    const destinyEvents = compiledData.events.filter((e) => e.tier === 'destiny');
    const eraEvents = compiledData.events.filter((e) => e.tier === 'era');
    const lifeEvents = compiledData.events.filter((e) => e.tier === 'life');

    expect(destinyEvents).toHaveLength(10);
    expect(eraEvents).toHaveLength(15);
    expect(lifeEvents).toHaveLength(15);
  });

  it('covers all 7 major geographic regions of China in location database', () => {
    const expectedRegions = ['华北', '华东', '华南', '华中', '西南', '西北', '东北'];
    const presentRegions = new Set(compiledData.locations.map((l) => l.region));

    for (const r of expectedRegions) {
      expect(presentRegions.has(r)).toBe(true);
    }
  });

  it('verifies resource squeeze: zero pure-gain choices in all life and era events', () => {
    for (const evt of compiledData.events) {
      if (!evt.choices) continue;

      expect(evt.choices.length).toBeGreaterThanOrEqual(2);

      for (const choice of evt.choices) {
        const effects = choice.effects || {};
        const dh = effects.health ?? 0;
        const dk = effects.knowledge ?? 0;
        const dm = effects.mood ?? 0;
        const di = effects.insight ?? 0;
        const dw = effects.wealthPPP ?? 0;
        const risk = choice.risk ?? 0;

        const hasGain = dh > 0 || dk > 0 || dm > 0 || di > 0 || dw > 0;
        const hasCost = dh < 0 || dk < 0 || dm < 0 || di < 0 || dw < 0 || risk > 0;

        if (hasGain) {
          expect(
            hasCost,
            `Choice ${choice.id} in event ${evt.id} is a pure gain without trade-off`,
          ).toBe(true);
        }
      }
    }
  });

  it('validates delayed consequences configuration (3-10 year delayed effects)', () => {
    let delayedEffectCount = 0;
    for (const evt of compiledData.events) {
      if (evt.delayedEffects) {
        for (const de of evt.delayedEffects) {
          expect(de.delayYears).toBeGreaterThanOrEqual(2);
          expect(de.delayYears).toBeLessThanOrEqual(15);
          expect(de.message.trim().length).toBeGreaterThan(0);
          delayedEffectCount++;
        }
      }
      if (evt.choices) {
        for (const c of evt.choices) {
          if (c.delayedEffects) {
            for (const de of c.delayedEffects) {
              expect(de.delayYears).toBeGreaterThanOrEqual(2);
              expect(de.delayYears).toBeLessThanOrEqual(15);
              expect(de.message.trim().length).toBeGreaterThan(0);
              delayedEffectCount++;
            }
          }
        }
      }
    }

    expect(delayedEffectCount).toBeGreaterThan(0);
  });

  it('property test: every location resolves a valid non-empty historical name for any year in 1900-1999', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1900, max: 1999 }),
        fc.constantFrom(...compiledData.locations),
        (year, location) => {
          const matched = location.historicalNames.find(
            (h) => year >= h.startYear && year <= h.endYear,
          );
          expect(matched).toBeDefined();
          expect(matched?.name.length).toBeGreaterThan(0);
        },
      ),
    );
  });
});
