import { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';

import {
  getAnalyticsSummary,
  getBehaviorSummary,
  getFunnelSummary,
  getIngestionStats,
  getLlmContext,
  getRerankerMetrics,
} from '../../api/analytics';
import { getProfileInsights, getWeeklyGuidance } from '../../api/enrichment';
import { readProfileId } from '../../lib/profileSession';
import { queryKeys } from '../../queryKeys';
import { buildInsights } from '../../pages/analytics/AnalyticsPanels';

export function useAnalyticsPage() {
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

  const { data: rerankerMetrics, isLoading: rerankerMetricsLoading } = useQuery({
    queryKey: queryKeys.analytics.rerankerMetrics(profileId ?? ''),
    queryFn: () => getRerankerMetrics(profileId!),
    enabled: !!profileId,
  });

  const { data: ingestionStats } = useQuery({
    queryKey: queryKeys.analytics.ingestionStats(),
    queryFn: () => getIngestionStats(),
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
            profileInsights,
            weeklyGuidance,
          })
        : [],
    [summary, profileInsights, weeklyGuidance],
  );

  return {
    profileId,
    summary,
    behavior,
    funnel,
    llmCtx,
    rerankerMetrics,
    ingestionStats,
    profileInsights,
    weeklyGuidance,
    aiInsights,
    isLoading:
      summaryLoading ||
      behaviorLoading ||
      funnelLoading ||
      ctxLoading ||
      rerankerMetricsLoading,
    insightsLoading,
    insightsError,
    weeklyGuidanceLoading,
    weeklyGuidanceError,
  };
}

export type AnalyticsPageState = ReturnType<typeof useAnalyticsPage>;
