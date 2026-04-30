import { Code2 } from 'lucide-react';

import type { MarketTechDemand } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

function TechDemandRow({ entry, maxCount }: { entry: MarketTechDemand; maxCount: number }) {
  const width = maxCount > 0 ? Math.max(6, (entry.jobCount / maxCount) * 100) : 0;

  return (
    <div className="grid gap-2 sm:grid-cols-[132px_minmax(0,1fr)_88px] sm:items-center">
      <div className="min-w-0">
        <p className="m-0 truncate text-sm font-semibold text-card-foreground">{entry.skill}</p>
      </div>
      <div className="h-3 overflow-hidden rounded-full bg-surface-muted">
        <div
          className="h-full rounded-full bg-primary"
          style={{ width: `${width}%` }}
          aria-hidden="true"
        />
      </div>
      <p className="m-0 text-left text-xs font-medium text-muted-foreground sm:text-right">
        {formatCount(entry.jobCount)} - {entry.percentage.toFixed(1)}%
      </p>
    </div>
  );
}

export function MarketTechDemandSection({ state }: { state: MarketPageState }) {
  const entries = state.techDemandQuery.data ?? [];
  const maxCount = entries[0]?.jobCount ?? 0;

  return (
    <MarketSection
      title="Tech Skills in Demand"
      description="Top technologies mentioned in active job titles and descriptions seen in the last 30 days."
      icon={Code2}
    >
      {state.techDemandQuery.isPending ? (
        <ListSkeleton rows={8} />
      ) : state.techDemandQuery.isError ? (
        <EmptyState
          message="Unable to load tech demand."
          description="The tech-demand endpoint did not return a usable response."
        />
      ) : entries.length > 0 ? (
        <div className="space-y-3">
          {entries.map((entry) => (
            <TechDemandRow key={entry.skill} entry={entry} maxCount={maxCount} />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No tech demand signals yet."
          description="Tech demand needs active jobs with recognizable technology mentions in the last 30 days."
        />
      )}
    </MarketSection>
  );
}
