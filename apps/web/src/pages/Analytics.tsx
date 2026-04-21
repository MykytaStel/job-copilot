import {
  AlertTriangle,
  BarChart2,
  Bookmark,
  Brain,
  Building2,
  Eye,
  EyeOff,
  FileWarning,
  Layers,
  Search,
  Sparkles,
  Target,
  ThumbsDown,
  TrendingDown,
  TrendingUp,
  XCircle,
  Zap,
} from 'lucide-react';

import { AIInsightPanel } from '../components/ui/AIInsightPanel';
import { Badge } from '../components/ui/Badge';
import { Card, CardContent } from '../components/ui/Card';
import { EmptyState } from '../components/ui/EmptyState';
import { Page, PageGrid } from '../components/ui/Page';
import { PageHeader } from '../components/ui/SectionHeader';
import { AnalyticsCard, StatCard } from '../components/ui/StatCard';
import { useAnalyticsPage } from '../features/analytics/useAnalyticsPage';
import {
  BarList,
  ConversionCard,
  HeroMetric,
  PillCloud,
  Section,
  SignalList,
} from './analytics/AnalyticsHelpers';
import {
  FunnelBySource,
  LlmContextPanel,
  ProfileInsightsPanel,
  WeeklyGuidancePanel,
} from './analytics/AnalyticsPanels';

export default function Analytics() {
  const {
    profileId,
    summary,
    behavior,
    funnel,
    llmCtx,
    profileInsights,
    weeklyGuidance,
    aiInsights,
    isLoading,
    insightsLoading,
    insightsError,
    weeklyGuidanceLoading,
    weeklyGuidanceError,
  } = useAnalyticsPage();

  if (!profileId) {
    return (
      <Page>
        <EmptyState message="Create a profile to view analytics." />
      </Page>
    );
  }

  return (
    <Page>
      <PageHeader
        title="Analytics"
        description="Track job-search progress, feedback signals, conversion flow, and enrichment-ready context."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Analytics' }]}
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
              value={
                funnel
                  ? `${Math.round(funnel.conversionRates.openRateFromImpressions * 100)}%`
                  : '0%'
              }
              icon={Eye}
            />
            <AnalyticsCard
              title="Apply Rate"
              value={
                funnel
                  ? `${Math.round(funnel.conversionRates.applicationRateFromSaves * 100)}%`
                  : '0%'
              }
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
                <Section
                  title="Search Quality"
                  description="Signals that explain why current matching can feel weak or noisy."
                  icon={AlertTriangle}
                  eyebrow="Diagnostics"
                >
                  <div className="grid grid-cols-2 gap-3">
                    <StatCard
                      title="Low evidence"
                      value={summary.searchQuality.lowEvidenceJobs}
                      icon={AlertTriangle}
                    />
                    <StatCard
                      title="Weak descriptions"
                      value={summary.searchQuality.weakDescriptionJobs}
                      icon={FileWarning}
                    />
                    <StatCard
                      title="Role mismatch"
                      value={summary.searchQuality.roleMismatchJobs}
                      icon={XCircle}
                    />
                    <StatCard
                      title="Seniority mismatch"
                      value={summary.searchQuality.seniorityMismatchJobs}
                      icon={TrendingDown}
                    />
                  </div>

                  <div className="mt-4">
                    <PillCloud
                      title="Top missing signals"
                      items={summary.searchQuality.topMissingSignals}
                      emptyMessage="No repeated missing signals yet."
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
                    <StatCard
                      title="Applications"
                      value={funnel.applicationCreatedCount}
                      icon={Zap}
                    />
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
                    <EmptyState message="Generating enrichment…" className="px-4 py-4 text-left" />
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
