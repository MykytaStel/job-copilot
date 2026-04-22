import {
  BarChart2,
  Brain,
  Bookmark,
  Building2,
  Eye,
  EyeOff,
  Layers,
  Sparkles,
  Target,
  ThumbsDown,
  TrendingUp,
  Zap,
} from 'lucide-react';
import type {
  AnalyticsSummary,
  BehaviorSummary,
  FunnelSummary,
  LlmContext,
} from '../../api/analytics';
import type { ProfileInsights, WeeklyGuidance } from '../../api/enrichment';

import { StatCard } from '../../components/ui/StatCard';

import {
  BarList,
  ConversionCard,
  Section,
  SignalList,
} from './AnalyticsHelpers';
import {
  FunnelBySource,
  LlmContextPanel,
  ProfileInsightsPanel,
  WeeklyGuidancePanel,
} from './AnalyticsPanels';
import { AnalyticsEnrichmentSection } from './AnalyticsEnrichmentSection';

export function AnalyticsMainSections({
  summary,
  behavior,
  funnel,
  llmCtx,
  weeklyGuidance,
  weeklyGuidanceLoading,
  weeklyGuidanceError,
  profileInsights,
  insightsLoading,
  insightsError,
}: {
  summary: AnalyticsSummary;
  behavior: BehaviorSummary | undefined;
  funnel: FunnelSummary | undefined;
  llmCtx: LlmContext | undefined;
  weeklyGuidance: WeeklyGuidance | undefined;
  weeklyGuidanceLoading: boolean;
  weeklyGuidanceError: unknown;
  profileInsights: ProfileInsights | undefined;
  insightsLoading: boolean;
  insightsError: unknown;
}) {
  return (
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
            <StatCard title="Fit Explainers" value={funnel.fitExplanationRequestedCount} icon={Brain} />
            <StatCard title="Coach Runs" value={funnel.applicationCoachRequestedCount} icon={Brain} />
            <StatCard title="Cover Letters" value={funnel.coverLetterDraftRequestedCount} icon={Layers} />
            <StatCard title="Interview Prep" value={funnel.interviewPrepRequestedCount} icon={Zap} />
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
              { label: 'Reactivated', value: summary.jobsByLifecycle.reactivated, tone: 'primary' },
            ]}
            emptyMessage="No lifecycle data yet."
          />
          <div className="mt-5 rounded-2xl border border-border/70 bg-surface-muted px-4 py-3">
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
        <AnalyticsEnrichmentSection
          title="Weekly Guidance"
          description="ML-generated weekly review based on analytics, funnel behavior, and current context."
          icon={Brain}
          eyebrow="Weekly readout"
          isLoading={weeklyGuidanceLoading}
          error={weeklyGuidanceError}
          loadingMessage="Generating weekly guidance…"
          errorMessage="Weekly guidance is unavailable right now."
          emptyMessage="No weekly guidance available yet."
        >
          {weeklyGuidance ? <WeeklyGuidancePanel guidance={weeklyGuidance} /> : undefined}
        </AnalyticsEnrichmentSection>
      ) : null}

      {llmCtx ? (
        <AnalyticsEnrichmentSection
          title="LLM Enrichment"
          description="Profile-specific strengths, risks, and action guidance produced from the current deterministic context."
          icon={Sparkles}
          eyebrow="Profile intelligence"
          isLoading={insightsLoading}
          error={insightsError}
          loadingMessage="Generating enrichment…"
          errorMessage="ML enrichment is unavailable right now."
          emptyMessage="No enrichment available yet."
        >
          {profileInsights ? <ProfileInsightsPanel insights={profileInsights} /> : undefined}
        </AnalyticsEnrichmentSection>
      ) : null}
    </div>
  );
}
