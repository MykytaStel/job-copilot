/* eslint-disable react-refresh/only-export-components */

import type { AnalyticsSummary, FunnelSummary, LlmContext } from '../../api/analytics';
import type { ProfileInsights, WeeklyGuidance } from '../../api/enrichment';
import type { AIInsight } from '../../components/ui/AIInsightPanel';
import { EmptyState } from '../../components/ui/EmptyState';
import { PillCloud, TextList } from './AnalyticsHelpers';

export function LlmContextPanel({ ctx }: { ctx: LlmContext }) {
  const seniorityLabel = ctx.analyzedProfile?.seniority?.trim()
    ? ctx.analyzedProfile.seniority
    : 'Not specified';

  return (
    <div className="space-y-5">
      {ctx.analyzedProfile ? (
        <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
          <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Profile anchor
          </p>
          <div className="mt-3 flex flex-wrap gap-2">
            <span className="inline-flex items-center rounded-full border border-primary/25 bg-primary/12 px-3 py-1.5 text-xs font-semibold text-primary">
              {ctx.analyzedProfile.primaryRole}
            </span>
            <span className="inline-flex items-center rounded-full border border-border bg-white/[0.05] px-3 py-1.5 text-xs text-muted-foreground">
              {seniorityLabel}
            </span>
          </div>
          <p className="m-0 mt-4 text-sm leading-7 text-card-foreground">
            {ctx.analyzedProfile.summary}
          </p>
        </div>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-2">
        <TextList
          title="Positive signals"
          items={ctx.topPositiveEvidence.map(
            (entry) => `${entry.type.replaceAll('_', ' ')}: ${entry.label}`,
          )}
          emptyMessage="No positive signals yet."
          tone="success"
        />
        <TextList
          title="Negative signals"
          items={ctx.topNegativeEvidence.map(
            (entry) => `${entry.type.replaceAll('_', ' ')}: ${entry.label}`,
          )}
          emptyMessage="No negative signals yet."
          tone="danger"
        />
      </div>
    </div>
  );
}

export function ProfileInsightsPanel({ insights }: { insights: ProfileInsights }) {
  return (
    <div className="space-y-5">
      <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
        <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
          Profile summary
        </p>
        <p className="m-0 mt-3 text-sm leading-7 text-card-foreground">
          {insights.profileSummary || 'No summary generated yet.'}
        </p>
        {insights.searchStrategySummary ? (
          <p className="m-0 mt-4 text-sm leading-7 text-muted-foreground">
            {insights.searchStrategySummary}
          </p>
        ) : null}
      </div>

      <div className="grid gap-4 xl:grid-cols-3">
        <TextList
          title="Strengths"
          items={insights.strengths}
          emptyMessage="No strengths highlighted yet."
          tone="success"
        />
        <TextList
          title="Risks"
          items={insights.risks}
          emptyMessage="No risks highlighted yet."
          tone="danger"
        />
        <TextList
          title="Recommended actions"
          items={insights.recommendedActions}
          emptyMessage="No actions suggested yet."
          tone="primary"
        />
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        <PillCloud
          title="Top focus areas"
          items={insights.topFocusAreas}
          emptyMessage="No focus areas suggested yet."
          tone="warning"
        />
        <PillCloud
          title="Search term suggestions"
          items={insights.searchTermSuggestions}
          emptyMessage="No search term suggestions yet."
          tone="primary"
        />
      </div>

      <TextList
        title="Application strategy"
        items={insights.applicationStrategy}
        emptyMessage="No application strategy generated yet."
        tone="success"
      />
    </div>
  );
}

export function WeeklyGuidancePanel({ guidance }: { guidance: WeeklyGuidance }) {
  return (
    <div className="space-y-5">
      <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
        <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
          Weekly summary
        </p>
        <p className="m-0 mt-3 text-sm leading-7 text-card-foreground">
          {guidance.weeklySummary || 'No weekly guidance generated yet.'}
        </p>
      </div>

      <div className="grid gap-4 xl:grid-cols-3">
        <TextList
          title="What is working"
          items={guidance.whatIsWorking}
          emptyMessage="No positive patterns identified yet."
          tone="success"
        />
        <TextList
          title="What is not working"
          items={guidance.whatIsNotWorking}
          emptyMessage="No weak signals identified yet."
          tone="danger"
        />
        <TextList
          title="Funnel bottlenecks"
          items={guidance.funnelBottlenecks}
          emptyMessage="No funnel bottlenecks highlighted yet."
          tone="warning"
        />
      </div>

      <div className="grid gap-4 xl:grid-cols-3">
        <TextList
          title="Search adjustments"
          items={guidance.recommendedSearchAdjustments}
          emptyMessage="No search changes suggested yet."
          tone="primary"
        />
        <TextList
          title="Source moves"
          items={guidance.recommendedSourceMoves}
          emptyMessage="No source shifts suggested yet."
          tone="success"
        />
        <TextList
          title="Role focus"
          items={guidance.recommendedRoleFocus}
          emptyMessage="No role focus changes suggested yet."
          tone="warning"
        />
      </div>

      <TextList
        title="Next week plan"
        items={guidance.nextWeekPlan}
        emptyMessage="No plan generated yet."
        tone="primary"
      />
    </div>
  );
}

export function FunnelBySource({ summary }: { summary: FunnelSummary }) {
  const rows = Array.from(
    new Set([
      ...summary.impressionsBySource.map((entry) => entry.source),
      ...summary.opensBySource.map((entry) => entry.source),
      ...summary.savesBySource.map((entry) => entry.source),
      ...summary.applicationsBySource.map((entry) => entry.source),
    ]),
  ).map((source) => ({
    source,
    impressions: summary.impressionsBySource.find((entry) => entry.source === source)?.count ?? 0,
    opens: summary.opensBySource.find((entry) => entry.source === source)?.count ?? 0,
    saves: summary.savesBySource.find((entry) => entry.source === source)?.count ?? 0,
    applications: summary.applicationsBySource.find((entry) => entry.source === source)?.count ?? 0,
  }));

  if (rows.length === 0) {
    return <EmptyState message="No source funnel data yet." className="px-4 py-4 text-left" />;
  }

  return (
    <div className="space-y-3">
      {rows.map((row) => (
        <div
          key={row.source}
          className="grid grid-cols-[minmax(0,1.2fr)_repeat(4,minmax(56px,1fr))] gap-2 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3 text-xs"
        >
          <span className="truncate font-semibold text-card-foreground">{row.source}</span>
          <span className="text-muted-foreground">Impr. {row.impressions}</span>
          <span className="text-muted-foreground">Open {row.opens}</span>
          <span className="text-muted-foreground">Save {row.saves}</span>
          <span className="text-muted-foreground">Apply {row.applications}</span>
        </div>
      ))}
    </div>
  );
}

export function buildInsights({
  summary,
  profileInsights,
  weeklyGuidance,
}: {
  summary: AnalyticsSummary;
  profileInsights: ProfileInsights | undefined;
  weeklyGuidance: WeeklyGuidance | undefined;
}): AIInsight[] {
  const insights: AIInsight[] = [];

  if (weeklyGuidance?.weeklySummary) {
    insights.push({
      id: 'weekly-summary',
      type: 'trend',
      title: 'Weekly guidance',
      description: weeklyGuidance.weeklySummary,
      action: { label: 'Tune search profile', href: '/profile' },
    });
  }

  if (weeklyGuidance?.recommendedSearchAdjustments[0]) {
    insights.push({
      id: 'search-adjustment',
      type: 'recommendation',
      title: 'Search adjustment',
      description: weeklyGuidance.recommendedSearchAdjustments[0],
      action: { label: 'Open profile', href: '/profile' },
    });
  }

  if (profileInsights?.recommendedActions[0]) {
    insights.push({
      id: 'profile-action',
      type: 'recommendation',
      title: 'Profile action',
      description: profileInsights.recommendedActions[0],
      action: { label: 'Review profile', href: '/profile' },
    });
  }

  if (weeklyGuidance?.funnelBottlenecks[0]) {
    insights.push({
      id: 'funnel-bottleneck',
      type: 'tip',
      title: 'Funnel bottleneck',
      description: weeklyGuidance.funnelBottlenecks[0],
      action: { label: 'Review feedback', href: '/feedback' },
    });
  }

  if (insights.length === 0) {
    insights.push({
      id: 'fallback',
      type: 'tip',
      title: 'Wait for more signal',
      description:
        summary.jobsByLifecycle.total > 0
          ? 'Analytics are live, but richer guidance needs more behavior and feedback history.'
          : 'Create activity in the system first to unlock analytics and guidance.',
      action: { label: 'Go to dashboard', href: '/' },
    });
  }

  return insights.slice(0, 4);
}
