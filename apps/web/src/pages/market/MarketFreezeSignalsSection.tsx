import { AlertTriangle } from 'lucide-react';

import type { MarketFreezeSignal } from '../../api/market';
import { EmptyState } from '../../components/ui/EmptyState';
import { semanticBadgeClass, semanticPanelClass } from '../../components/ui/semanticTone';
import { cn } from '../../lib/cn';
import type { MarketPageState } from '../../features/market/useMarketPage';
import { formatCount } from './market.view-model';
import { ListSkeleton } from './MarketSkeletons';
import { MarketSection } from './MarketSection';

const dateFormatter = new Intl.DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  year: 'numeric',
});

function formatLastPostedAt(value: string) {
  const parsed = new Date(value);

  if (Number.isNaN(parsed.getTime())) {
    return value;
  }

  return dateFormatter.format(parsed);
}

function FreezeSignalRow({ signal }: { signal: MarketFreezeSignal }) {
  return (
    <div
      className={cn(
        'grid gap-3 rounded-2xl border px-4 py-4 md:grid-cols-[minmax(0,1fr)_150px_170px] md:items-center',
        semanticPanelClass.warning,
      )}
    >
      <div className="min-w-0">
        <p className="m-0 truncate text-sm font-semibold text-card-foreground">
          {signal.company}
        </p>
        <p className="m-0 mt-1 text-xs leading-5 text-muted-foreground">
          Last new posting {formatLastPostedAt(signal.lastPostedAt)}
        </p>
      </div>

      <div className="flex items-center justify-between gap-3 md:block md:text-right">
        <span className="text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
          60 day posts
        </span>
        <p className="m-0 mt-1 text-lg font-semibold text-card-foreground">
          {formatCount(signal.historicalCount)}
        </p>
      </div>

      <div className="flex items-center justify-between gap-3 md:justify-end">
        <span className="text-xs text-muted-foreground">Silent for</span>
        <span
          className={cn(
            'inline-flex h-8 items-center justify-center rounded-full border px-2.5 text-xs font-medium',
            semanticBadgeClass.warning,
          )}
        >
          {formatCount(signal.daysSinceLastPost)} days
        </span>
      </div>
    </div>
  );
}

export function MarketFreezeSignalsSection({ state }: { state: MarketPageState }) {
  const entries = state.freezeSignalsQuery.data ?? [];

  return (
    <MarketSection
      title="Hiring Paused"
      description="Companies with at least 5 jobs first seen in the last 60 days, but no new postings in the last 14 days."
      icon={AlertTriangle}
    >
      {state.freezeSignalsQuery.isPending ? (
        <ListSkeleton rows={5} />
      ) : state.freezeSignalsQuery.isError ? (
        <EmptyState
          message="Unable to load hiring pause signals."
          description="The freeze signals endpoint did not return a usable response."
        />
      ) : entries.length > 0 ? (
        <div className="space-y-3">
          {entries.map((signal) => (
            <FreezeSignalRow key={signal.company} signal={signal} />
          ))}
        </div>
      ) : (
        <EmptyState
          message="No hiring pause signals yet."
          description="This section needs companies with at least 5 postings in 60 days and no postings in the last 14 days."
        />
      )}
    </MarketSection>
  );
}
