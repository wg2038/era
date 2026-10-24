import fc from 'fast-check';
import { describe, expect, it } from 'vitest';
import { compiledData } from '../dist/index.js';

describe('Data Compiler & Historical Data Integrity', () => {
  it('loads valid compiled data schema', () => {
    expect(compiledData).toBeDefined();
    expect(compiledData.version).toBe('0.1.0');
    expect(compiledData.locations.length).toBeGreaterThan(0);
    expect(compiledData.events.length).toBeGreaterThan(0);
    expect(compiledData.lifeTables.length).toBeGreaterThan(0);
  });

  it('ensures all historical locations have valid confidence and non-empty sources', () => {
    for (const loc of compiledData.locations) {
      expect(['A', 'B', 'C']).toContain(loc.confidence);
      expect(loc.source.trim().length).toBeGreaterThan(0);
      expect(loc.historicalNames.length).toBeGreaterThan(0);

      // Verify that historical names have valid start and end years
      for (const h of loc.historicalNames) {
        expect(h.startYear).toBeLessThanOrEqual(h.endYear);
        expect(h.name.trim().length).toBeGreaterThan(0);
      }
    }
  });

  it('ensures all seed events have valid confidence and non-empty sources', () => {
    for (const evt of compiledData.events) {
      expect(['A', 'B', 'C']).toContain(evt.confidence);
      expect(evt.source.trim().length).toBeGreaterThan(0);
      expect(['destiny', 'era', 'life']).toContain(evt.tier);
    }
  });

  it('property test: any birth year between 1900 and 1999 resolves a historical name for Beijing', () => {
    const beijing = compiledData.locations.find((l) => l.id === 'loc_beijing');
    expect(beijing).toBeDefined();

    fc.assert(
      fc.property(fc.integer({ min: 1900, max: 1999 }), (year) => {
        const entry = beijing?.historicalNames.find(
          (h) => year >= h.startYear && year <= h.endYear,
        );
        expect(entry).toBeDefined();
        if (year <= 1911) {
          expect(entry?.name).toBe('京师顺天府');
        } else if (year >= 1912 && year < 1928) {
          expect(entry?.name).toBe('北京');
        } else if (year >= 1928 && year < 1949) {
          expect(entry?.name).toBe('北平');
        } else {
          expect(entry?.name).toBe('北京');
        }
      }),
    );
  });
});
