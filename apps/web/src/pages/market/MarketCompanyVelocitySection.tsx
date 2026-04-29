import { BarChart3 } from 'lucide-react';

import type { MarketCompanyVelocity, MarketCompanyVelocityTrend } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import { semanticBadgeClass } from '../../components/ui/semanticTone';
import { cn } from '../../lib/cn';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

const trendMeta: Record<
  MarketCompanyVelocityTrend,
  {
    label: string;
    className: string;
  }
> = {
  growing: {
    label: '↑ growing',
    className: semanticBadgeClass.success,
  },
  stable: {
    label: 'stable',
    className: semanticBadgeClass.muted,
  },
  declining: {
    label: '↓ declining',
    className: semanticBadgeClass.danger,
  },
};

function CompanyVelocityRow({
  entry,
  maxJobCount,
}: {
  entry: MarketCompanyVelocity;
  maxJobCount: number;
}) {
  const width = maxJobCount > 0 ? Math.max(8, Math.round((entry.jobCount / maxJobCount) * 100)) : 0;
  const meta = trendMeta[entry.trend];

  return (
    <div className="grid gap-2 md:grid-cols-[minmax(140px,220px)_minmax(0,1fr)_112px] md:items-center">
      <p className="m-0 min-w-0 truncate text-sm font-semibold text-card-foreground">
        {entry.company}
      </p>
      <div className="min-w-0">
        <div className="h-8 overflow-hidden rounded-[var(--radius-md)] border border-border bg-background/70">
          <div
            className="flex h-full items-center justify-end bg-primary/70 px-2 text-xs font-semibold text-primary-foreground"
            style={{ width: `${width}%` }}
          >
            {formatCount(entry.jobCount)}
          </div>
        </div>
      </div>
      <span
        className={cn(
          'inline-flex h-8 items-center justify-center rounded-full border px-2.5 text-xs font-medium',
          meta.className,
        )}
      >
        {meta.label}
      </span>
    </div>
  );
}

export function MarketCompanyVelocitySection({ state }: { state: MarketPageState }) {
  const entries = state.companyVelocityQuery.data ?? [];
  const maxJobCount = Math.max(0, ...entries.map((entry) => entry.jobCount));

  return (
    <MarketSection
      title="Company Hiring Velocity"
      description="Top companies by new postings first seen in the last 30 days, filtered to companies with at least 3 recent jobs."
      icon={BarChart3}
    >
      {state.companyVelocityQuery.isPending ? (
        <ListSkeleton rows={6} />
      ) : state.companyVelocityQuery.isError ? (
        <EmptyState
          message="Unable to load hiring velocity."
          description="The company velocity endpoint did not return a usable response."
        />
      ) : entries.length > 0 ? (
        <div className="space-y-3">
          {entries.map((entry) => (
            <CompanyVelocityRow
              key={entry.company}
              entry={entry}
              maxJobCount={maxJobCount}
            />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No company velocity signals yet."
          description="This section needs companies with at least 3 jobs first seen in the last 30 days."
        />
      )}
    </MarketSection>
  );
}
