import type { MarketTrend } from '../../api/market';
import { cn } from '../../lib/cn';
import { getTrendMeta } from './market.view-model';

export function MarketTrendBadge({ trend }: { trend: MarketTrend | number }) {
  const meta = getTrendMeta(trend);
  const Icon = meta.icon;

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-xs font-medium',
        meta.className,
      )}
    >
      <Icon className="h-3.5 w-3.5" />
      {meta.label}
    </span>
  );
}
