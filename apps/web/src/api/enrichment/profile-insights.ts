import type { LlmContext } from '../analytics';
import { json, request } from '../client';

import { buildLlmContextPayload } from './shared';
import type { MlProfileInsightsResponse, ProfileInsights } from './types';

export function buildProfileInsightsPayload(context: LlmContext) {
  return buildLlmContextPayload(context);
}

export function mapProfileInsightsResponse(response: MlProfileInsightsResponse): ProfileInsights {
  return {
    profileSummary: response.profile_summary,
    searchStrategySummary: response.search_strategy_summary,
    strengths: response.strengths,
    risks: response.risks,
    recommendedActions: response.recommended_actions,
    topFocusAreas: response.top_focus_areas,
    searchTermSuggestions: response.search_term_suggestions,
    applicationStrategy: response.application_strategy,
  };
}

export async function getProfileInsights(context: LlmContext): Promise<ProfileInsights> {
  const response = await request<MlProfileInsightsResponse>(
    '/api/v1/enrichment/profile-insights',
    json('POST', buildProfileInsightsPayload(context)),
  );

  return mapProfileInsightsResponse(response);
}
