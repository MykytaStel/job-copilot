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
        { seniority: 'junior', medianMin: 1000, medianMax: 2000, sampleSize: 2 },
        { seniority: 'mid', medianMin: 2500, medianMax: 3500, sampleSize: 8 },
      ]),
    ).toBe(3000);

    expect(
      deriveSalaryBounds([
        { seniority: 'junior', medianMin: 1000, medianMax: 2000, sampleSize: 2 },
        { seniority: 'mid', medianMin: 2500, medianMax: 3500, sampleSize: 8 },
      ]),
    ).toEqual({ min: 1000, max: 3500 });
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
