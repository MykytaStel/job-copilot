import { describe, expect, it } from 'vitest';

import {
  deriveMedianSalary,
  deriveSalaryBounds,
  sortRoleDemand,
} from '../src/features/market/useMarketPage';

describe('market page helpers', () => {
  it('derives weighted salary median and empty bounds safely', () => {
    expect(deriveMedianSalary([])).toBeNull();
    expect(deriveSalaryBounds([])).toEqual({ min: 0, max: 0 });

    expect(
      deriveMedianSalary([
        { seniority: 'junior', p25: 1000, median: 1500, p75: 2000, sampleCount: 2 },
        { seniority: 'middle', p25: 2500, median: 3000, p75: 3500, sampleCount: 8 },
      ]),
    ).toBe(3000);
  });

  it('sorts role demand by current period and previous period fallback', () => {
    expect(
      sortRoleDemand([
        { roleGroup: 'QA', thisPeriod: 5, prevPeriod: 9, trend: 'down' },
        { roleGroup: 'Backend', thisPeriod: 12, prevPeriod: 2, trend: 'up' },
        { roleGroup: 'Frontend', thisPeriod: 5, prevPeriod: 10, trend: 'down' },
      ]).map((role) => role.roleGroup),
    ).toEqual(['Backend', 'Frontend', 'QA']);
  });
});
