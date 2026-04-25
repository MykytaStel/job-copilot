import { mlRequest } from '../client';
import { fireEvent } from '../events';

import {
  buildAnalyzedProfilePayload,
  buildDeterministicFitPayload,
  buildFeedbackStatePayload,
  buildJobFitExplanationSummary,
  buildRankedJobPayload,
  buildSearchProfilePayload,
  type ApplicationCoachRequest,
} from './shared';
import type { ApplicationCoach, MlApplicationCoachResponse } from './types';

export function buildApplicationCoachPayload(payload: ApplicationCoachRequest) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    job_fit_explanation: buildJobFitExplanationSummary(payload.jobFitExplanation),
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
    raw_profile_text: payload.rawProfileText ?? null,
  };
}

export function mapApplicationCoachResponse(
  response: MlApplicationCoachResponse,
): ApplicationCoach {
  return {
    applicationSummary: response.application_summary,
    resumeFocusPoints: response.resume_focus_points,
    suggestedBullets: response.suggested_bullets,
    coverLetterAngles: response.cover_letter_angles,
    interviewFocus: response.interview_focus,
    gapsToAddress: response.gaps_to_address,
    redFlags: response.red_flags,
  };
}

export async function getApplicationCoach(
  payload: ApplicationCoachRequest,
): Promise<ApplicationCoach> {
  fireEvent(payload.profileId, {
    eventType: 'application_coach_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      has_fit_explanation: Boolean(payload.jobFitExplanation),
      primary_role: payload.searchProfile?.primaryRole ?? null,
    },
  });

  const response = await mlRequest<MlApplicationCoachResponse>('/v1/enrichment/application-coach', {
    method: 'POST',
    body: JSON.stringify(buildApplicationCoachPayload(payload)),
  });

  return mapApplicationCoachResponse(response);
}
