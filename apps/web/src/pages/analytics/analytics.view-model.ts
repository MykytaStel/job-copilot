import type { AnalyticsSummary, BehaviorSummary, FunnelSummary } from '../../api/analytics';

export function buildAnalyticsViewModel({
  summary,
  behavior,
  funnel,
}: {
  summary: AnalyticsSummary;
  behavior: BehaviorSummary | undefined;
  funnel: FunnelSummary | undefined;
}) {
  return {
    heroMetrics: [
      { label: 'Indexed jobs', value: summary.jobsByLifecycle.total, icon: 'layers' as const },
      { label: 'Search runs', value: behavior?.searchRunCount ?? 0, icon: 'search' as const },
      { label: 'Applications', value: funnel?.applicationCreatedCount ?? 0, icon: 'target' as const },
    ],
    feedbackCards: [
      { title: 'Saved Jobs', value: summary.feedback.savedJobsCount, icon: 'bookmark' as const },
      { title: 'Bad Fit', value: summary.feedback.badFitJobsCount, icon: 'thumbsDown' as const },
      {
        title: 'Open Rate',
        value: funnel ? `${Math.round(funnel.conversionRates.openRateFromImpressions * 100)}%` : '0%',
        icon: 'eye' as const,
      },
      {
        title: 'Apply Rate',
        value: funnel
          ? `${Math.round(funnel.conversionRates.applicationRateFromSaves * 100)}%`
          : '0%',
        icon: 'zap' as const,
      },
    ],
  };
}
