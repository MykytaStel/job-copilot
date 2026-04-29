import { MapPinned } from 'lucide-react';

import type { MarketRegionDemand } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

function RegionRow({ entry, maxCount }: { entry: MarketRegionDemand; maxCount: number }) {
  const width = maxCount > 0 ? Math.max(6, (entry.jobCount / maxCount) * 100) : 0;
  const roleLabel = entry.topRoles.length > 0 ? entry.topRoles.join(', ') : 'No dominant roles yet';

  return (
    <div className="grid gap-3 rounded-2xl border border-border/70 bg-surface-muted px-4 py-4 sm:grid-cols-[144px_minmax(0,1fr)_88px] sm:items-center">
      <div className="min-w-0">
        <p className="m-0 truncate text-sm font-semibold text-card-foreground">{entry.region}</p>
        <p className="m-0 mt-1 truncate text-xs text-muted-foreground">{roleLabel}</p>
      </div>
      <div className="h-3 overflow-hidden rounded-full bg-background">
        <div
          className="h-full rounded-full bg-primary"
          style={{ width: `${width}%` }}
          aria-hidden="true"
        />
      </div>
      <p className="m-0 text-left text-xs font-medium text-muted-foreground sm:text-right">
        {formatCount(entry.jobCount)} jobs
      </p>
    </div>
  );
}

export function MarketRegionBreakdownSection({ state }: { state: MarketPageState }) {
  const entries = state.regionBreakdownQuery.data ?? [];
  const maxCount = Math.max(0, ...entries.map((entry) => entry.jobCount));

  return (
    <MarketSection
      title="Role Demand by Region"
      description="Active job volume by region bucket, with the most common role groups inside each bucket."
      icon={MapPinned}
    >
      {state.regionBreakdownQuery.isPending ? (
        <ListSkeleton rows={5} />
      ) : state.regionBreakdownQuery.isError ? (
        <EmptyState
          message="Unable to load region demand."
          description="The region-breakdown endpoint did not return a usable response."
        />
      ) : entries.length > 0 ? (
        <div className="space-y-3">
          {entries.map((entry) => (
            <RegionRow key={entry.region} entry={entry} maxCount={maxCount} />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No region signals yet."
          description="Region demand needs active jobs with location or remote-mode data."
        />
      )}
    </MarketSection>
  );
}
