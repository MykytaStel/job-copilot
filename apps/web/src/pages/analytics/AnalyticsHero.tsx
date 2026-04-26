import { Bookmark, Clock, Eye, Layers, Search, Target, ThumbsDown, Zap } from 'lucide-react';
import type { AnalyticsSummary, BehaviorSummary, FunnelSummary, IngestionStats } from '../../api/analytics';

import { Badge } from '../../components/ui/Badge';
import { Card, CardContent } from '../../components/ui/Card';
import { AnalyticsCard } from '../../components/ui/StatCard';

import { HeroMetric } from './AnalyticsHelpers';
import { buildAnalyticsViewModel } from './analytics.view-model';

const heroIcons = {
  layers: Layers,
  search: Search,
  target: Target,
} as const;

const cardIcons = {
  bookmark: Bookmark,
  thumbsDown: ThumbsDown,
  eye: Eye,
  zap: Zap,
} as const;

function formatRelativeTime(isoString: string | null): string {
  if (!isoString) return 'Never';

  const date = new Date(isoString.endsWith('Z') ? isoString : isoString + 'Z');
  const diffMs = Date.now() - date.getTime();
  const diffMin = Math.floor(diffMs / 60_000);

  if (diffMin < 1) return 'Just now';
  if (diffMin < 60) return `${diffMin} min ago`;
  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays}d ago`;
}

export function AnalyticsHero({
  summary,
  behavior,
  funnel,
  ingestionStats,
}: {
  summary: AnalyticsSummary;
  behavior: BehaviorSummary | undefined;
  funnel: FunnelSummary | undefined;
  ingestionStats: IngestionStats | undefined;
}) {
  const viewModel = buildAnalyticsViewModel({ summary, behavior, funnel });

  return (
    <>
      <Card className="overflow-hidden border-border bg-card">
        <CardContent className="p-0">
          <div className="relative">
            <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-primary/10 via-accent/6 to-transparent" />
            <div className="relative flex flex-col gap-6 p-7 lg:flex-row lg:items-end lg:justify-between">
              <div className="max-w-3xl space-y-3">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge
                    variant="default"
                    className="border-0 bg-primary/15 px-2 py-0.5 text-xs text-primary"
                  >
                    Feedback-driven analytics
                  </Badge>
                  <Badge
                    variant="muted"
                    className="px-2 py-0.5 text-[10px] uppercase tracking-wide"
                  >
                    Funnel, source quality, enrichment signals
                  </Badge>
                  <span className="flex items-center gap-1 text-[11px] text-muted-foreground">
                    <Clock className="h-3 w-3 shrink-0" />
                    Last updated:{' '}
                    <span className="font-medium text-foreground">
                      {ingestionStats ? formatRelativeTime(ingestionStats.lastIngestedAt) : '—'}
                    </span>
                  </span>
                </div>
                <h2 className="m-0 text-2xl font-bold text-card-foreground lg:text-3xl">
                  Measure what is actually improving ranking, engagement, and application flow
                </h2>
                <p className="m-0 text-sm leading-7 text-muted-foreground lg:text-base">
                  Analytics combine deterministic search behavior, explicit feedback, and
                  enrichment-ready context so you can tune the profile, sources, and follow-up
                  actions without guessing.
                </p>
              </div>
              <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[460px]">
                {viewModel.heroMetrics.map((metric) => {
                  const Icon = heroIcons[metric.icon];

                  return <HeroMetric key={metric.label} label={metric.label} value={metric.value} icon={Icon} />;
                })}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-2 gap-4 xl:grid-cols-4">
        {viewModel.feedbackCards.map((card) => {
          const Icon = cardIcons[card.icon];

          return <AnalyticsCard key={card.title} title={card.title} value={card.value} icon={Icon} />;
        })}
      </div>
    </>
  );
}
