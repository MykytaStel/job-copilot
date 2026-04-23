import { Page, PageGrid } from '../../components/ui/Page';
import { PageHeader } from '../../components/ui/SectionHeader';
import type { AnalyticsPageState } from '../../features/analytics/useAnalyticsPage';

import { AnalyticsHero } from './AnalyticsHero';
import { AnalyticsMainSections } from './AnalyticsMainSections';
import { AnalyticsSidebar } from './AnalyticsSidebar';

export function AnalyticsContent({ state }: { state: AnalyticsPageState }) {
  const {
    summary,
    behavior,
    funnel,
    aiInsights,
    llmCtx,
    rerankerMetrics,
    weeklyGuidance,
    weeklyGuidanceLoading,
    weeklyGuidanceError,
    profileInsights,
    insightsLoading,
    insightsError,
  } = state;

  if (!summary) {
    return null;
  }

  return (
    <Page>
      <PageHeader
        title="Analytics"
        description="Track job-search progress, feedback signals, conversion flow, and enrichment-ready context."
        breadcrumb={[{ label: 'Dashboard', href: '/' }, { label: 'Analytics' }]}
      />

      <AnalyticsHero summary={summary} behavior={behavior} funnel={funnel} />

      <PageGrid aside={<AnalyticsSidebar summary={summary} aiInsights={aiInsights} />}>
        <AnalyticsMainSections
          summary={summary}
          behavior={behavior}
          funnel={funnel}
          llmCtx={llmCtx}
          rerankerMetrics={rerankerMetrics}
          weeklyGuidance={weeklyGuidance}
          weeklyGuidanceLoading={weeklyGuidanceLoading}
          weeklyGuidanceError={weeklyGuidanceError}
          profileInsights={profileInsights}
          insightsLoading={insightsLoading}
          insightsError={insightsError}
        />
      </PageGrid>
    </Page>
  );
}
