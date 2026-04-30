import { CircleDollarSign } from 'lucide-react';

import type { MarketSalaryBySeniority } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount, formatSalary } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

function seniorityLabel(seniority: string) {
  switch (seniority) {
    case 'junior':
      return 'Junior';
    case 'mid':
      return 'Mid';
    case 'senior':
      return 'Senior';
    case 'lead_staff':
      return 'Lead/Staff';
    default:
      return seniority.charAt(0).toUpperCase() + seniority.slice(1);
  }
}

function SalaryRow({
  salary,
  minValue,
  maxValue,
}: {
  salary: MarketSalaryBySeniority;
  minValue: number;
  maxValue: number;
}) {
  const domain = Math.max(maxValue - minValue, 1);
  const rangeStart = ((salary.medianMin - minValue) / domain) * 100;
  const rangeWidth = ((salary.medianMax - salary.medianMin) / domain) * 100;
  const boundedRangeStart = Math.min(100, Math.max(0, rangeStart));
  const boundedRangeWidth = Math.min(100 - boundedRangeStart, Math.max(rangeWidth, 4));

  return (
    <div className="rounded-2xl border border-border/70 bg-surface-muted px-4 py-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <p className="m-0 text-sm font-semibold text-card-foreground">
            {seniorityLabel(salary.seniority)}
          </p>
          <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
            Based on {formatCount(salary.sampleSize)} recent postings with salary and seniority data
          </p>
        </div>
        <div className="text-left sm:text-right">
          <p className="m-0 text-sm font-semibold text-card-foreground">
            {formatSalary(salary.medianMin, 'USD')} - {formatSalary(salary.medianMax, 'USD')}
          </p>
          <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
            Median posted salary range, normalized to USD/month
          </p>
        </div>
      </div>
      <div className="mt-4">
        <div className="relative h-3 rounded-full bg-white-a05">
          <div
            className="absolute top-0 h-full rounded-full bg-primary/25"
            style={{ left: `${boundedRangeStart}%`, width: `${boundedRangeWidth}%` }}
          />
        </div>
        <div className="mt-2 flex items-center justify-between text-[11px] uppercase tracking-[0.12em] text-muted-foreground">
          <span>{formatSalary(minValue, 'USD')}</span>
          <span>{formatSalary(maxValue, 'USD')}</span>
        </div>
      </div>
    </div>
  );
}

export function MarketSalarySection({ state }: { state: MarketPageState }) {
  return (
    <MarketSection
      title="Salary by Seniority"
      description="Median posted salary ranges from recent active postings with inferred seniority. Amounts are normalized to USD/month."
      icon={CircleDollarSign}
    >
      {state.salariesQuery.isPending ? (
        <ListSkeleton rows={4} />
      ) : state.salariesQuery.isError ? (
        <EmptyState
          message="Unable to load salary ranges."
          description="The salary analytics endpoint returned an error."
        />
      ) : state.salaryBySeniority.length > 0 ? (
        <div className="space-y-3">
          {state.salaryBySeniority.map((salary) => (
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
          description="Salary ranges appear here when at least 10 recent postings share a seniority bucket and structured compensation."
        />
      )}
    </MarketSection>
  );
}
