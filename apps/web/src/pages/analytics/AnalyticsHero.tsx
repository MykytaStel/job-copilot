import { Bookmark, Eye, Layers, Search, Target, ThumbsDown, Zap } from 'lucide-react';
import type { AnalyticsSummary, BehaviorSummary, FunnelSummary } from '../../api/analytics';

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

export function AnalyticsHero({
  summary,
  behavior,
  funnel,
}: {
  summary: AnalyticsSummary;
  behavior: BehaviorSummary | undefined;
  funnel: FunnelSummary | undefined;
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
                <div className="flex flex-wrap gap-2">
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
