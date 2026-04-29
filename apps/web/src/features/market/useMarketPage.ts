import { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';

import {
  getMarketCompanies,
  getMarketCompanyVelocity,
  getMarketOverview,
  getMarketRoles,
  getMarketSalaries,
  type MarketRoleDemand,
  type MarketSalaryTrend,
} from '../../api/market';
import { queryKeys } from '../../queryKeys';

const MARKET_STALE_TIME_MS = 5 * 60_000;

export function deriveMedianSalary(trends: MarketSalaryTrend[]) {
  if (trends.length === 0) {
    return null;
  }

  const ordered = [...trends].sort((left, right) => left.median - right.median);
  const totalWeight = ordered.reduce((sum, item) => sum + item.sampleCount, 0);

  if (totalWeight <= 0) {
    return ordered[Math.floor(ordered.length / 2)]?.median ?? null;
  }

  let cumulativeWeight = 0;
  for (const item of ordered) {
    cumulativeWeight += item.sampleCount;
    if (cumulativeWeight >= totalWeight / 2) {
      return item.median;
    }
  }

  return ordered.at(-1)?.median ?? null;
}

export function sortRoleDemand(roles: MarketRoleDemand[]) {
  return roles
    .slice()
    .sort(
      (left, right) => right.thisPeriod - left.thisPeriod || right.prevPeriod - left.prevPeriod,
    );
}

export function deriveSalaryBounds(trends: MarketSalaryTrend[]) {
  if (trends.length === 0) {
    return { min: 0, max: 0 };
  }

  return {
    min: Math.min(...trends.map((item) => item.p25)),
    max: Math.max(...trends.map((item) => item.p75)),
  };
}

export function useMarketPage() {
  const overviewQuery = useQuery({
    queryKey: queryKeys.market.overview(),
    queryFn: getMarketOverview,
    staleTime: MARKET_STALE_TIME_MS,
  });
  const companiesQuery = useQuery({
    queryKey: queryKeys.market.companies(),
    queryFn: () => getMarketCompanies(12),
    staleTime: MARKET_STALE_TIME_MS,
  });
  const companyVelocityQuery = useQuery({
    queryKey: queryKeys.market.companyVelocity(),
    queryFn: getMarketCompanyVelocity,
    staleTime: MARKET_STALE_TIME_MS,
  });
  const salariesQuery = useQuery({
    queryKey: queryKeys.market.salaries(),
    queryFn: () => getMarketSalaries(),
    staleTime: MARKET_STALE_TIME_MS,
  });
  const rolesQuery = useQuery({
    queryKey: queryKeys.market.roles(),
    queryFn: () => getMarketRoles(30),
    staleTime: MARKET_STALE_TIME_MS,
  });

  const salaryTrends = useMemo(() => salariesQuery.data ?? [], [salariesQuery.data]);
  const roleDemand = useMemo(
    () =>
      sortRoleDemand(
        rolesQuery.data?.filter((role) => role.thisPeriod > 0 || role.prevPeriod > 0) ?? [],
      ),
    [rolesQuery.data],
  );
  const marketMedian = useMemo(() => deriveMedianSalary(salaryTrends), [salaryTrends]);
  const salarySampleCount = useMemo(
    () => salaryTrends.reduce((sum, item) => sum + item.sampleCount, 0),
    [salaryTrends],
  );
  const salaryBounds = useMemo(() => deriveSalaryBounds(salaryTrends), [salaryTrends]);

  return {
    overviewQuery,
    companiesQuery,
    companyVelocityQuery,
    salariesQuery,
    rolesQuery,
    salaryTrends,
    roleDemand,
    marketMedian,
    salarySampleCount,
    salaryMin: salaryBounds.min,
    salaryMax: salaryBounds.max,
  };
}

export type MarketPageState = ReturnType<typeof useMarketPage>;
