import { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';

import {
  getMarketCompanies,
  getMarketCompanyVelocity,
  getMarketFreezeSignals,
  getMarketOverview,
  getMarketRegionBreakdown,
  getMarketRoles,
  getMarketSalaryBySeniority,
  getMarketTechDemand,
  type MarketRoleDemand,
  type MarketSalaryBySeniority,
} from '../../api/market';
import { queryKeys } from '../../queryKeys';

const MARKET_STALE_TIME_MS = 5 * 60_000;

export function deriveMedianSalary(entries: MarketSalaryBySeniority[]) {
  if (entries.length === 0) {
    return null;
  }

  const ordered = [...entries].sort(
    (left, right) =>
      (left.medianMin + left.medianMax) / 2 - (right.medianMin + right.medianMax) / 2,
  );
  const totalWeight = ordered.reduce((sum, item) => sum + item.sampleSize, 0);

  if (totalWeight <= 0) {
    const medianEntry = ordered[Math.floor(ordered.length / 2)];
    return medianEntry ? (medianEntry.medianMin + medianEntry.medianMax) / 2 : null;
  }

  let cumulativeWeight = 0;
  for (const item of ordered) {
    cumulativeWeight += item.sampleSize;
    if (cumulativeWeight >= totalWeight / 2) {
      return (item.medianMin + item.medianMax) / 2;
    }
  }

  const lastEntry = ordered.at(-1);
  return lastEntry ? (lastEntry.medianMin + lastEntry.medianMax) / 2 : null;
}

export function sortRoleDemand(roles: MarketRoleDemand[]) {
  return roles
    .slice()
    .sort(
      (left, right) => right.thisPeriod - left.thisPeriod || right.prevPeriod - left.prevPeriod,
    );
}

export function deriveSalaryBounds(entries: MarketSalaryBySeniority[]) {
  if (entries.length === 0) {
    return { min: 0, max: 0 };
  }

  return {
    min: Math.min(...entries.map((item) => item.medianMin)),
    max: Math.max(...entries.map((item) => item.medianMax)),
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
  const freezeSignalsQuery = useQuery({
    queryKey: queryKeys.market.freezeSignals(),
    queryFn: getMarketFreezeSignals,
    staleTime: MARKET_STALE_TIME_MS,
  });
  const salariesQuery = useQuery({
    queryKey: queryKeys.market.salaries(),
    queryFn: getMarketSalaryBySeniority,
    staleTime: MARKET_STALE_TIME_MS,
  });
  const rolesQuery = useQuery({
    queryKey: queryKeys.market.roles(),
    queryFn: () => getMarketRoles(30),
    staleTime: MARKET_STALE_TIME_MS,
  });
  const regionBreakdownQuery = useQuery({
    queryKey: queryKeys.market.regionBreakdown(),
    queryFn: getMarketRegionBreakdown,
    staleTime: MARKET_STALE_TIME_MS,
  });
  const techDemandQuery = useQuery({
    queryKey: queryKeys.market.techDemand(),
    queryFn: getMarketTechDemand,
    staleTime: MARKET_STALE_TIME_MS,
  });

  const salaryBySeniority = useMemo(() => salariesQuery.data ?? [], [salariesQuery.data]);
  const roleDemand = useMemo(
    () =>
      sortRoleDemand(
        rolesQuery.data?.filter((role) => role.thisPeriod > 0 || role.prevPeriod > 0) ?? [],
      ),
    [rolesQuery.data],
  );
  const marketMedian = useMemo(() => deriveMedianSalary(salaryBySeniority), [salaryBySeniority]);
  const salarySampleCount = useMemo(
    () => salaryBySeniority.reduce((sum, item) => sum + item.sampleSize, 0),
    [salaryBySeniority],
  );
  const salaryBounds = useMemo(() => deriveSalaryBounds(salaryBySeniority), [salaryBySeniority]);

  return {
    overviewQuery,
    companiesQuery,
    companyVelocityQuery,
    freezeSignalsQuery,
    salariesQuery,
    rolesQuery,
    regionBreakdownQuery,
    techDemandQuery,
    salaryBySeniority,
    roleDemand,
    marketMedian,
    salarySampleCount,
    salaryMin: salaryBounds.min,
    salaryMax: salaryBounds.max,
  };
}

export type MarketPageState = ReturnType<typeof useMarketPage>;
