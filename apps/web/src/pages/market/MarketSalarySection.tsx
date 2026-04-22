import { CircleDollarSign } from 'lucide-react';

import type { MarketSalaryTrend } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import {
  formatCount,
  formatSalary,
  titleCase,
} from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

function SalaryRow({
  salary,
  minValue,
  maxValue,
}: {
  salary: MarketSalaryTrend;
  minValue: number;
  maxValue: number;
}) {
  const domain = Math.max(maxValue - minValue, 1);
  const rangeStart = ((salary.p25 - minValue) / domain) * 100;
  const rangeWidth = ((salary.p75 - salary.p25) / domain) * 100;
  const medianPosition = ((salary.median - minValue) / domain) * 100;

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <p className="m-0 text-sm font-semibold text-card-foreground">
            {titleCase(salary.seniority)}
          </p>
          <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
            Based on {formatCount(salary.sampleCount)} active postings with salary data
          </p>
        </div>
        <div className="text-left sm:text-right">
          <p className="m-0 text-sm font-semibold text-card-foreground">
            Median {formatSalary(salary.median)}
          </p>
          <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
            {formatSalary(salary.p25)} - {formatSalary(salary.p75)} p25-p75 range
          </p>
        </div>
      </div>
      <div className="mt-4">
        <div className="relative h-3 rounded-full bg-white-a05">
          <div
            className="absolute top-0 h-full rounded-full bg-primary/25"
            style={{ left: `${rangeStart}%`, width: `${Math.max(rangeWidth, 3)}%` }}
          />
          <div
            className="absolute top-1/2 h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-background bg-primary shadow-[0_0_0_4px_rgba(90,132,255,0.18)]"
            style={{ left: `${medianPosition}%` }}
          />
        </div>
        <div className="mt-2 flex items-center justify-between text-[11px] uppercase tracking-[0.12em] text-muted-foreground">
          <span>{formatSalary(minValue)}</span>
          <span>{formatSalary(maxValue)}</span>
        </div>
      </div>
    </div>
  );
}

export function MarketSalarySection({ state }: { state: MarketPageState }) {
  return (
    <MarketSection
      title="Salary by Seniority"
      description="P25-p75 ranges with the median marker for the recent active salary sample in each seniority bucket."
      icon={CircleDollarSign}
    >
      {state.salariesQuery.isPending ? (
        <ListSkeleton rows={4} />
      ) : state.salariesQuery.isError ? (
        <EmptyState
          message="Unable to load salary ranges."
          description="The salary analytics endpoint returned an error."
        />
      ) : state.salaryTrends.length > 0 ? (
        <div className="space-y-3">
          {state.salaryTrends.map((salary) => (
            <SalaryRow
              key={salary.seniority}
              salary={salary}
              minValue={state.salaryMin}
              maxValue={state.salaryMax}
            />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No recent salary distribution data."
          description="Salary ranges appear here when active postings include structured compensation."
        />
      )}
    </MarketSection>
  );
}
