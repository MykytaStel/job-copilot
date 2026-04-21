import {
  BriefcaseBusiness,
  Building2,
  CircleDollarSign,
  Wifi,
} from 'lucide-react';

import { StatCard } from '../../components/ui/StatCard';
import type { MarketPageState } from '../../features/market/useMarketPage';
import {
  formatCount,
  formatPercent,
  formatSalary,
} from './market.view-model';
import { StatCardSkeleton } from './MarketSkeletons';

export function MarketHero({ state }: { state: MarketPageState }) {
  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      {state.overviewQuery.isPending ? (
        <>
          <StatCardSkeleton />
          <StatCardSkeleton />
          <StatCardSkeleton />
        </>
      ) : (
        <>
          <StatCard
            title="New jobs this week"
            value={state.overviewQuery.data ? formatCount(state.overviewQuery.data.newJobsThisWeek) : '—'}
            description={
              state.overviewQuery.data
                ? `${formatCount(state.overviewQuery.data.activeJobsCount)} active jobs tracked right now`
                : 'Overview data unavailable'
            }
            icon={BriefcaseBusiness}
          />
          <StatCard
            title="Active companies"
            value={
              state.overviewQuery.data
                ? formatCount(state.overviewQuery.data.activeCompaniesCount)
                : '—'
            }
            description="Companies with at least one active posting"
            icon={Building2}
          />
          <StatCard
            title="Remote share"
            value={
              state.overviewQuery.data
                ? formatPercent(state.overviewQuery.data.remotePercentage)
                : '—'
            }
            description="Share of active jobs explicitly marked remote"
            icon={Wifi}
          />
        </>
      )}

      {state.salariesQuery.isPending ? (
        <StatCardSkeleton />
      ) : (
        <StatCard
          title="Median salary"
          value={state.marketMedian !== null ? formatSalary(state.marketMedian) : '—'}
          description={
            state.salarySampleCount > 0
              ? `Derived from ${formatCount(state.salarySampleCount)} salary-tagged postings`
              : 'No recent salary reports across tracked seniority bands'
          }
          icon={CircleDollarSign}
        />
      )}
    </div>
  );
}
