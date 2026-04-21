import type { BehaviorSignalCount } from '../analytics';
import { mlRequest } from '../client';

import {
  buildFeedbackSummaryPayload,
  buildLlmContextPayload,
  type WeeklyGuidanceRequest,
} from './shared';
import type { MlWeeklyGuidanceResponse, WeeklyGuidance } from './types';

function buildBehaviorSignalPayload(signal: BehaviorSignalCount) {
  return {
    key: signal.key,
    save_count: signal.saveCount,
    hide_count: signal.hideCount,
    bad_fit_count: signal.badFitCount,
    application_created_count: signal.applicationCreatedCount,
    positive_count: signal.positiveCount,
    negative_count: signal.negativeCount,
    net_score: signal.netScore,
  };
}

export function buildWeeklyGuidancePayload(payload: WeeklyGuidanceRequest) {
  return {
    profile_id: payload.profileId,
    analytics_summary: {
      feedback: buildFeedbackSummaryPayload(payload.analyticsSummary.feedback),
      jobs_by_source: payload.analyticsSummary.jobsBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      jobs_by_lifecycle: {
        total: payload.analyticsSummary.jobsByLifecycle.total,
        active: payload.analyticsSummary.jobsByLifecycle.active,
        inactive: payload.analyticsSummary.jobsByLifecycle.inactive,
        reactivated: payload.analyticsSummary.jobsByLifecycle.reactivated,
      },
      top_matched_roles: payload.analyticsSummary.topMatchedRoles,
      top_matched_skills: payload.analyticsSummary.topMatchedSkills,
      top_matched_keywords: payload.analyticsSummary.topMatchedKeywords,
    },
    behavior_summary: {
      search_run_count: payload.behaviorSummary.searchRunCount,
      top_positive_sources: payload.behaviorSummary.topPositiveSources.map(buildBehaviorSignalPayload),
      top_negative_sources: payload.behaviorSummary.topNegativeSources.map(buildBehaviorSignalPayload),
      top_positive_role_families:
        payload.behaviorSummary.topPositiveRoleFamilies.map(buildBehaviorSignalPayload),
      top_negative_role_families:
        payload.behaviorSummary.topNegativeRoleFamilies.map(buildBehaviorSignalPayload),
      source_signal_counts: payload.behaviorSummary.sourceSignalCounts.map(buildBehaviorSignalPayload),
      role_family_signal_counts:
        payload.behaviorSummary.roleFamilySignalCounts.map(buildBehaviorSignalPayload),
    },
    funnel_summary: {
      impression_count: payload.funnelSummary.impressionCount,
      open_count: payload.funnelSummary.openCount,
      save_count: payload.funnelSummary.saveCount,
      hide_count: payload.funnelSummary.hideCount,
      bad_fit_count: payload.funnelSummary.badFitCount,
      application_created_count: payload.funnelSummary.applicationCreatedCount,
      fit_explanation_requested_count: payload.funnelSummary.fitExplanationRequestedCount,
      application_coach_requested_count: payload.funnelSummary.applicationCoachRequestedCount,
      cover_letter_draft_requested_count: payload.funnelSummary.coverLetterDraftRequestedCount,
      interview_prep_requested_count: payload.funnelSummary.interviewPrepRequestedCount,
      conversion_rates: {
        open_rate_from_impressions: payload.funnelSummary.conversionRates.openRateFromImpressions,
        save_rate_from_opens: payload.funnelSummary.conversionRates.saveRateFromOpens,
        application_rate_from_saves: payload.funnelSummary.conversionRates.applicationRateFromSaves,
      },
      impressions_by_source: payload.funnelSummary.impressionsBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      opens_by_source: payload.funnelSummary.opensBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      saves_by_source: payload.funnelSummary.savesBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
      applications_by_source: payload.funnelSummary.applicationsBySource.map((entry) => ({
        source: entry.source,
        count: entry.count,
      })),
    },
    llm_context: buildLlmContextPayload(payload.llmContext, {
      includeProfileId: false,
    }),
  };
}

export function mapWeeklyGuidanceResponse(response: MlWeeklyGuidanceResponse): WeeklyGuidance {
  return {
    weeklySummary: response.weekly_summary,
    whatIsWorking: response.what_is_working,
    whatIsNotWorking: response.what_is_not_working,
    recommendedSearchAdjustments: response.recommended_search_adjustments,
    recommendedSourceMoves: response.recommended_source_moves,
    recommendedRoleFocus: response.recommended_role_focus,
    funnelBottlenecks: response.funnel_bottlenecks,
    nextWeekPlan: response.next_week_plan,
  };
}

export async function getWeeklyGuidance(
  payload: WeeklyGuidanceRequest,
): Promise<WeeklyGuidance> {
  const response = await mlRequest<MlWeeklyGuidanceResponse>('/v1/enrichment/weekly-guidance', {
    method: 'POST',
    body: JSON.stringify(buildWeeklyGuidancePayload(payload)),
  });

  return mapWeeklyGuidanceResponse(response);
}
