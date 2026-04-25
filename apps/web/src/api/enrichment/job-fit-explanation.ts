import { mlRequest } from '../client';
import { fireEvent } from '../events';

import {
  buildAnalyzedProfilePayload,
  buildDeterministicFitPayload,
  buildFeedbackStatePayload,
  buildRankedJobPayload,
  buildSearchProfilePayload,
  type JobFitExplanationRequest,
} from './shared';
import type { JobFitExplanation, MlJobFitExplanationResponse } from './types';

export function buildJobFitExplanationPayload(payload: JobFitExplanationRequest) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
  };
}

export function mapJobFitExplanationResponse(
  response: MlJobFitExplanationResponse,
): JobFitExplanation {
  return {
    fitSummary: response.fit_summary,
    whyItMatches: response.why_it_matches,
    risks: response.risks,
    missingSignals: response.missing_signals,
    recommendedNextStep: response.recommended_next_step,
    applicationAngle: response.application_angle,
  };
}

export async function getJobFitExplanation(
  payload: JobFitExplanationRequest,
): Promise<JobFitExplanation> {
  fireEvent(payload.profileId, {
    eventType: 'fit_explanation_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      primary_role: payload.searchProfile?.primaryRole ?? null,
      has_feedback_state: Boolean(payload.feedbackState),
    },
  });

  const response = await mlRequest<MlJobFitExplanationResponse>(
    '/v1/enrichment/job-fit-explanation',
    {
      method: 'POST',
      body: JSON.stringify(buildJobFitExplanationPayload(payload)),
    },
  );

  return mapJobFitExplanationResponse(response);
}
