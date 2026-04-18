import { useMemo, type ReactNode } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  BarChart2,
  Bookmark,
  Brain,
  Building2,
  Eye,
  EyeOff,
  Hash,
  Layers,
  Search,
  ShieldCheck,
  ShieldOff,
  Sparkles,
  Target,
  ThumbsDown,
  TrendingUp,
  Zap,
  type LucideIcon,
} from 'lucide-react';

import {
  getAnalyticsSummary,
  getBehaviorSummary,
  getFunnelSummary,
  getLlmContext,
  getProfileInsights,
  getWeeklyGuidance,
} from '../api';
import type {
  AnalyticsSummary,
  BehaviorSignalCount,
  FunnelSummary,
  LlmContext,
  ProfileInsights,
  WeeklyGuidance,
} from '../api';
import { AIInsightPanel, type AIInsight } from '../components/ui/AIInsightPanel';
import { Badge } from '../components/ui/Badge';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page, PageGrid } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { AnalyticsCard, StatCard } from '../components/ui/StatCard';
import { cn } from '../lib/cn';
import { queryKeys } from '../queryKeys';

function readProfileId() {
  return window.localStorage.getItem('engine_api_profile_id');
}

type Tone = 'primary' | 'success' | 'warning' | 'danger' | 'muted';

const toneClasses: Record<Tone, string> = {
  primary: 'bg-primary/15 text-primary border-primary/25',
  success: 'bg-fit-excellent/15 text-fit-excellent border-fit-excellent/25',
  warning: 'bg-fit-fair/15 text-fit-fair border-fit-fair/25',
  danger: 'bg-destructive/15 text-destructive border-destructive/25',
  muted: 'bg-white/[0.05] text-muted-foreground border-border',
};

const barToneClasses: Record<Tone, string> = {
  primary: 'bg-primary',
  success: 'bg-fit-excellent',
  warning: 'bg-fit-fair',
  danger: 'bg-destructive',
  muted: 'bg-muted-foreground',
};

function Section({
  title,
  description,
  icon: Icon,
  eyebrow,
  action,
  children,
  className,
}: {
  title: string;
  description?: string;
  icon: LucideIcon;
  eyebrow?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <Card className={cn('border-border bg-card', className)}>
      <CardHeader className="gap-3">
        <div className="flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-start gap-3">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
              <Icon className="h-5 w-5" />
            </div>
            <div className="min-w-0">
              {eyebrow ? (
                <p className="m-0 text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                  {eyebrow}
                </p>
              ) : null}
              <CardTitle className="mt-1 text-base font-semibold">{title}</CardTitle>
              {description ? (
                <p className="m-0 mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
              ) : null}
            </div>
          </div>
          {action}
        </div>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

function HeroMetric({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string | number;
  icon: LucideIcon;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.04] px-4 py-3">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-primary/15 bg-primary/10 text-primary">
          <Icon className="h-4 w-4" />
        </div>
        <div>
          <p className="m-0 text-[11px] uppercase tracking-[0.14em] text-muted-foreground">
            {label}
          </p>
          <p className="m-0 mt-1 text-sm font-semibold text-card-foreground">{value}</p>
        </div>
      </div>
    </div>
  );
}

function BarList({
  items,
  emptyMessage,
}: {
  items: { label: string; value: number; tone?: Tone }[];
  emptyMessage: string;
}) {
  const maxValue = Math.max(...items.map((item) => item.value), 1);

  if (items.length === 0) {
    return <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />;
  }

  return (
    <div className="space-y-3">
      {items.map((item) => {
        const width = `${Math.round((item.value / maxValue) * 100)}%`;
        const tone = item.tone ?? 'primary';

        return (
          <div key={item.label} className="grid grid-cols-[minmax(0,1fr)_56px] items-center gap-3">
            <div className="min-w-0 space-y-2">
              <div className="flex items-center justify-between gap-3">
                <p className="m-0 truncate text-sm text-card-foreground">{item.label}</p>
                <span className="text-xs text-muted-foreground">{item.value}</span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-white/[0.05]">
                <div
                  className={cn('h-full rounded-full transition-[width] duration-300', barToneClasses[tone])}
                  style={{ width }}
                />
              </div>
            </div>
            <p className="m-0 text-right text-sm font-semibold text-card-foreground">{item.value}</p>
          </div>
        );
      })}
    </div>
  );
}

function ConversionCard({
  label,
  rate,
  numerator,
  denominator,
  tone,
}: {
  label: string;
  rate: number;
  numerator: number;
  denominator: number;
  tone: Tone;
}) {
  const width = `${Math.max(0, Math.min(rate, 1)) * 100}%`;

  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <div className="mb-2 flex items-center justify-between gap-3">
        <p className="m-0 text-xs text-muted-foreground">{label}</p>
        <span className={cn('text-lg font-bold', tone === 'danger' ? 'text-destructive' : tone === 'warning' ? 'text-fit-fair' : tone === 'success' ? 'text-fit-excellent' : 'text-primary')}>
          {Math.round(rate * 100)}%
        </span>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-white/[0.05]">
        <div
          className={cn('h-full rounded-full transition-[width] duration-300', barToneClasses[tone])}
          style={{ width }}
        />
      </div>
      <p className="m-0 mt-2 text-xs text-muted-foreground">
        {numerator} / {denominator || 0}
      </p>
    </div>
  );
}

function SignalList({
  title,
  description,
  items,
  tone,
}: {
  title: string;
  description: string;
  items: BehaviorSignalCount[];
  tone: Tone;
}) {
  if (items.length === 0) {
    return (
      <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
        <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
        <p className="m-0 mt-1 text-xs leading-6 text-muted-foreground">{description}</p>
        <EmptyState message="No signal data yet." className="px-4 py-4 text-left" />
      </div>
    );
  }

  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
      <p className="m-0 mt-1 text-xs leading-6 text-muted-foreground">{description}</p>
      <div className="mt-4 space-y-3">
        {items.slice(0, 6).map((item) => (
          <div
            key={item.key}
            className="flex items-start justify-between gap-3 rounded-xl border border-border/60 bg-background/60 px-3 py-3"
          >
            <div className="min-w-0">
              <p className="m-0 truncate text-sm font-medium text-card-foreground">{item.key}</p>
              <div className="mt-2 flex flex-wrap gap-2">
                <span className="rounded-full border border-border bg-white/[0.04] px-2 py-1 text-[11px] text-muted-foreground">
                  saves {item.saveCount}
                </span>
                <span className="rounded-full border border-border bg-white/[0.04] px-2 py-1 text-[11px] text-muted-foreground">
                  applies {item.applicationCreatedCount}
                </span>
                <span className="rounded-full border border-border bg-white/[0.04] px-2 py-1 text-[11px] text-muted-foreground">
                  bad fit {item.badFitCount}
                </span>
              </div>
            </div>
            <span className={cn('shrink-0 rounded-full border px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.14em]', toneClasses[tone])}>
              net {item.netScore}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

function PillCloud({
  title,
  items,
  emptyMessage,
  tone,
}: {
  title: string;
  items: string[];
  emptyMessage: string;
  tone: Tone;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
      {items.length === 0 ? (
        <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />
      ) : (
        <div className="mt-4 flex flex-wrap gap-2">
          {items.map((item) => (
            <span
              key={item}
              className={cn(
                'inline-flex items-center rounded-full border px-3 py-1.5 text-xs',
                toneClasses[tone],
              )}
            >
              {item}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

function TextList({
  title,
  items,
  emptyMessage,
  tone,
}: {
  title: string;
  items: string[];
  emptyMessage: string;
  tone: Tone;
}) {
  return (
    <div className="rounded-2xl border border-border/70 bg-white/[0.03] p-4">
      <p className="m-0 text-sm font-semibold text-card-foreground">{title}</p>
      {items.length === 0 ? (
        <EmptyState message={emptyMessage} className="px-4 py-4 text-left" />
      ) : (
        <div className="mt-4 space-y-3">
          {items.map((item) => (
            <div key={item} className="flex items-start gap-3 rounded-xl border border-border/60 bg-background/60 px-3 py-3">
              <span className={cn('mt-1 h-2 w-2 shrink-0 rounded-full', barToneClasses[tone])} />
              <p className="m-0 text-sm leading-6 text-card-foreground">{item}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function LlmContextPanel({ ctx }: { ctx: LlmContext }) {
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
              {ctx.analyzedProfile.seniority}
            </span>
          </div>
          <p className="m-0 mt-4 text-sm leading-7 text-card-foreground">{ctx.analyzedProfile.summary}</p>
        </div>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-2">
        <TextList
          title="Positive signals"
          items={ctx.topPositiveEvidence.map((entry) => `${entry.type.replaceAll('_', ' ')}: ${entry.label}`)}
          emptyMessage="No positive signals yet."
          tone="success"
        />
        <TextList
          title="Negative signals"
          items={ctx.topNegativeEvidence.map((entry) => `${entry.type.replaceAll('_', ' ')}: ${entry.label}`)}
          emptyMessage="No negative signals yet."
          tone="danger"
        />
      </div>
    </div>
  );
}

function ProfileInsightsPanel({ insights }: { insights: ProfileInsights }) {
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

function WeeklyGuidancePanel({ guidance }: { guidance: WeeklyGuidance }) {
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

function FunnelBySource({ summary }: { summary: FunnelSummary }) {
  const rows = Array.from(
    new Set([
      ...summary.impressionsBySource.map((entry) => entry.source),
      ...summary.opensBySource.map((entry) => entry.source),
      ...summary.savesBySource.map((entry) => entry.source),
      ...summary.applicationsBySource.map((entry) => entry.source),
    ]),
  ).map((source) => ({
    source,
    impressions:
      summary.impressionsBySource.find((entry) => entry.source === source)?.count ?? 0,
    opens: summary.opensBySource.find((entry) => entry.source === source)?.count ?? 0,
    saves: summary.savesBySource.find((entry) => entry.source === source)?.count ?? 0,
    applications:
      summary.applicationsBySource.find((entry) => entry.source === source)?.count ?? 0,
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

function buildInsights({
  summary,
  funnel,
  profileInsights,
  weeklyGuidance,
}: {
  summary: AnalyticsSummary;
  funnel: FunnelSummary | undefined;
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

export default function Analytics() {
  const profileId = readProfileId();

  const { data: summary, isLoading: summaryLoading } = useQuery({
    queryKey: queryKeys.analytics.summary(profileId ?? ''),
    queryFn: () => getAnalyticsSummary(profileId!),
    enabled: !!profileId,
  });

  const { data: behavior, isLoading: behaviorLoading } = useQuery({
    queryKey: queryKeys.analytics.behavior(profileId ?? ''),
    queryFn: () => getBehaviorSummary(profileId!),
    enabled: !!profileId,
  });

  const { data: funnel, isLoading: funnelLoading } = useQuery({
    queryKey: queryKeys.analytics.funnel(profileId ?? ''),
    queryFn: () => getFunnelSummary(profileId!),
    enabled: !!profileId,
  });

  const { data: llmCtx, isLoading: ctxLoading } = useQuery({
    queryKey: queryKeys.analytics.llmContext(profileId ?? ''),
    queryFn: () => getLlmContext(profileId!),
    enabled: !!profileId,
  });

  const enrichmentContextVersion = llmCtx ? JSON.stringify(llmCtx) : '';

  const {
    data: profileInsights,
    isLoading: insightsLoading,
    error: insightsError,
  } = useQuery({
    queryKey: queryKeys.analytics.profileInsights(profileId ?? '', enrichmentContextVersion),
    queryFn: () => getProfileInsights(llmCtx!),
    enabled: !!profileId && !!llmCtx,
    retry: 0,
  });

  const weeklyGuidanceContextVersion =
    summary && behavior && funnel && llmCtx
      ? JSON.stringify({ summary, behavior, funnel, llmCtx })
      : '';

  const {
    data: weeklyGuidance,
    isLoading: weeklyGuidanceLoading,
    error: weeklyGuidanceError,
  } = useQuery({
    queryKey: queryKeys.analytics.weeklyGuidance(profileId ?? '', weeklyGuidanceContextVersion),
    queryFn: () =>
      getWeeklyGuidance({
        profileId: profileId!,
        analyticsSummary: summary!,
        behaviorSummary: behavior!,
        funnelSummary: funnel!,
        llmContext: llmCtx!,
      }),
    enabled: !!profileId && !!summary && !!behavior && !!funnel && !!llmCtx,
    retry: 0,
  });

  const aiInsights = useMemo(
    () =>
      summary
        ? buildInsights({
            summary,
            funnel,
            profileInsights,
            weeklyGuidance,
          })
        : [],
    [summary, funnel, profileInsights, weeklyGuidance],
  );

  if (!profileId) {
    return (
      <Page>
        <EmptyState message="Create a profile to view analytics." />
      </Page>
    );
  }

  const isLoading = summaryLoading || behaviorLoading || funnelLoading || ctxLoading;

  return (
    <Page>
      <PageHeader
        title="Analytics"
        description="Track job-search progress, feedback signals, conversion flow, and enrichment-ready context."
        breadcrumb={[
          { label: 'Dashboard', href: '/' },
          { label: 'Analytics' },
        ]}
      />

      {isLoading ? (
        <EmptyState message="Loading analytics…" />
      ) : summary ? (
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
                    <HeroMetric
                      label="Indexed jobs"
                      value={summary.jobsByLifecycle.total}
                      icon={Layers}
                    />
                    <HeroMetric
                      label="Search runs"
                      value={behavior?.searchRunCount ?? 0}
                      icon={Search}
                    />
                    <HeroMetric
                      label="Applications"
                      value={funnel?.applicationCreatedCount ?? 0}
                      icon={Target}
                    />
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="grid grid-cols-2 gap-4 xl:grid-cols-4">
            <AnalyticsCard
              title="Saved Jobs"
              value={summary.feedback.savedJobsCount}
              icon={Bookmark}
            />
            <AnalyticsCard
              title="Bad Fit"
              value={summary.feedback.badFitJobsCount}
              icon={ThumbsDown}
            />
            <AnalyticsCard
              title="Open Rate"
              value={funnel ? `${Math.round(funnel.conversionRates.openRateFromImpressions * 100)}%` : '0%'}
              icon={Eye}
            />
            <AnalyticsCard
              title="Apply Rate"
              value={funnel ? `${Math.round(funnel.conversionRates.applicationRateFromSaves * 100)}%` : '0%'}
              icon={Zap}
            />
          </div>

          <PageGrid
            aside={
              <>
                <AIInsightPanel insights={aiInsights} title="AI Guidance" />
                <Section
                  title="Match Surface"
                  description="Top deterministic dimensions currently shaping ranked jobs."
                  icon={Sparkles}
                  eyebrow="Current signal"
                >
                  <div className="space-y-4">
                    <PillCloud
                      title="Matched roles"
                      items={summary.topMatchedRoles}
                      emptyMessage="No matched roles yet."
                      tone="primary"
                    />
                    <PillCloud
                      title="Matched skills"
                      items={summary.topMatchedSkills}
                      emptyMessage="No matched skills yet."
                      tone="success"
                    />
                    <PillCloud
                      title="Matched keywords"
                      items={summary.topMatchedKeywords}
                      emptyMessage="No matched keywords yet."
                      tone="warning"
                    />
                  </div>
                </Section>
              </>
            }
          >
            <div className="space-y-8">
              {funnel ? (
                <Section
                  title="Search Funnel"
                  description="Follow the conversion path from impressions to applications and see where the loop breaks."
                  icon={Target}
                  eyebrow="Conversion"
                >
                  <div className="grid grid-cols-2 gap-4 xl:grid-cols-4">
                    <StatCard title="Impressions" value={funnel.impressionCount} icon={Eye} />
                    <StatCard title="Opens" value={funnel.openCount} icon={BarChart2} />
                    <StatCard title="Saves" value={funnel.saveCount} icon={Bookmark} />
                    <StatCard title="Applications" value={funnel.applicationCreatedCount} icon={Zap} />
                  </div>

                  <div className="mt-6 grid gap-4 xl:grid-cols-3">
                    <ConversionCard
                      label="Open from impressions"
                      rate={funnel.conversionRates.openRateFromImpressions}
                      numerator={funnel.openCount}
                      denominator={funnel.impressionCount}
                      tone="primary"
                    />
                    <ConversionCard
                      label="Save from opens"
                      rate={funnel.conversionRates.saveRateFromOpens}
                      numerator={funnel.saveCount}
                      denominator={funnel.openCount}
                      tone="warning"
                    />
                    <ConversionCard
                      label="Apply from saves"
                      rate={funnel.conversionRates.applicationRateFromSaves}
                      numerator={funnel.applicationCreatedCount}
                      denominator={funnel.saveCount}
                      tone="success"
                    />
                  </div>

                  <div className="mt-6 grid grid-cols-2 gap-4 xl:grid-cols-3">
                    <StatCard title="Hidden" value={funnel.hideCount} icon={EyeOff} />
                    <StatCard title="Bad Fit" value={funnel.badFitCount} icon={ThumbsDown} />
                    <StatCard
                      title="Fit Explainers"
                      value={funnel.fitExplanationRequestedCount}
                      icon={Brain}
                    />
                    <StatCard
                      title="Coach Runs"
                      value={funnel.applicationCoachRequestedCount}
                      icon={Brain}
                    />
                    <StatCard
                      title="Cover Letters"
                      value={funnel.coverLetterDraftRequestedCount}
                      icon={Layers}
                    />
                    <StatCard
                      title="Interview Prep"
                      value={funnel.interviewPrepRequestedCount}
                      icon={Zap}
                    />
                  </div>
                </Section>
              ) : null}

              <div className="grid gap-6 xl:grid-cols-2">
                <Section
                  title="Jobs by Source"
                  description="Where the indexed opportunity volume is currently coming from."
                  icon={BarChart2}
                  eyebrow="Supply"
                >
                  <BarList
                    items={summary.jobsBySource.map((entry, index) => ({
                      label: entry.source,
                      value: entry.count,
                      tone: index % 2 === 0 ? 'primary' : 'warning',
                    }))}
                    emptyMessage="No source data yet."
                  />
                </Section>

                <Section
                  title="Lifecycle Coverage"
                  description="How much of the feed is active, inactive, or reactivated."
                  icon={TrendingUp}
                  eyebrow="Feed health"
                >
                  <BarList
                    items={[
                      { label: 'Active', value: summary.jobsByLifecycle.active, tone: 'success' },
                      { label: 'Inactive', value: summary.jobsByLifecycle.inactive, tone: 'muted' },
                      {
                        label: 'Reactivated',
                        value: summary.jobsByLifecycle.reactivated,
                        tone: 'primary',
                      },
                    ]}
                    emptyMessage="No lifecycle data yet."
                  />
                  <div className="mt-5 rounded-2xl border border-border/70 bg-white/[0.03] px-4 py-3">
                    <p className="m-0 text-xs uppercase tracking-[0.14em] text-muted-foreground">
                      Total indexed
                    </p>
                    <p className="m-0 mt-2 text-2xl font-bold text-card-foreground">
                      {summary.jobsByLifecycle.total}
                    </p>
                  </div>
                </Section>
              </div>

              {behavior ? (
                <Section
                  title="Behavior Signals"
                  description="See which sources and role families consistently produce positive or negative outcomes."
                  icon={TrendingUp}
                  eyebrow="Learning loop"
                >
                  <div className="grid gap-4 xl:grid-cols-2">
                    <SignalList
                      title="Positive sources"
                      description="Sources with the strongest positive behavior history."
                      items={behavior.topPositiveSources}
                      tone="success"
                    />
                    <SignalList
                      title="Negative sources"
                      description="Sources associated with poor fit, hides, or low-quality outcomes."
                      items={behavior.topNegativeSources}
                      tone="danger"
                    />
                    <SignalList
                      title="Positive role families"
                      description="Role groups with strong saves and applications."
                      items={behavior.topPositiveRoleFamilies}
                      tone="success"
                    />
                    <SignalList
                      title="Negative role families"
                      description="Role groups repeatedly marked as weak or irrelevant."
                      items={behavior.topNegativeRoleFamilies}
                      tone="danger"
                    />
                  </div>
                </Section>
              ) : null}

              {funnel ? (
                <Section
                  title="Funnel by Source"
                  description="Break down the conversion path per source to see where quality actually comes from."
                  icon={Building2}
                  eyebrow="Source quality"
                >
                  <FunnelBySource summary={funnel} />
                </Section>
              ) : null}

              {llmCtx ? (
                <Section
                  title="LLM Context Preview"
                  description="Deterministic payload prepared for enrichment and coaching layers."
                  icon={Brain}
                  eyebrow="Enrichment context"
                >
                  <LlmContextPanel ctx={llmCtx} />
                </Section>
              ) : null}

              {llmCtx ? (
                <Section
                  title="Weekly Guidance"
                  description="ML-generated weekly review based on analytics, funnel behavior, and current context."
                  icon={Brain}
                  eyebrow="Weekly readout"
                >
                  {weeklyGuidanceLoading ? (
                    <EmptyState
                      message="Generating weekly guidance…"
                      className="px-4 py-4 text-left"
                    />
                  ) : weeklyGuidanceError ? (
                    <EmptyState
                      message={
                        (weeklyGuidanceError as Error).message ||
                        'Weekly guidance is unavailable right now.'
                      }
                      className="px-4 py-4 text-left"
                    />
                  ) : weeklyGuidance ? (
                    <WeeklyGuidancePanel guidance={weeklyGuidance} />
                  ) : (
                    <EmptyState
                      message="No weekly guidance available yet."
                      className="px-4 py-4 text-left"
                    />
                  )}
                </Section>
              ) : null}

              {llmCtx ? (
                <Section
                  title="LLM Enrichment"
                  description="Profile-specific strengths, risks, and action guidance produced from the current deterministic context."
                  icon={Sparkles}
                  eyebrow="Profile intelligence"
                >
                  {insightsLoading ? (
                    <EmptyState
                      message="Generating enrichment…"
                      className="px-4 py-4 text-left"
                    />
                  ) : insightsError ? (
                    <EmptyState
                      message={
                        (insightsError as Error).message ||
                        'ML enrichment is unavailable right now.'
                      }
                      className="px-4 py-4 text-left"
                    />
                  ) : profileInsights ? (
                    <ProfileInsightsPanel insights={profileInsights} />
                  ) : (
                    <EmptyState
                      message="No enrichment available yet."
                      className="px-4 py-4 text-left"
                    />
                  )}
                </Section>
              ) : null}
            </div>
          </PageGrid>
        </>
      ) : (
        <EmptyState message="No analytics data available." />
      )}
    </Page>
  );
}
