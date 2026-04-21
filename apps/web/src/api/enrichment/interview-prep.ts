import { mlRequest } from '../client';
import { logUserEvent } from '../events';

import {
  buildAnalyzedProfilePayload,
  buildApplicationCoachSummary,
  buildCoverLetterDraftSummary,
  buildDeterministicFitPayload,
  buildFeedbackStatePayload,
  buildJobFitExplanationSummary,
  buildRankedJobPayload,
  buildSearchProfilePayload,
  type InterviewPrepRequest,
} from './shared';
import type { InterviewPrep, MlInterviewPrepResponse } from './types';

export function buildInterviewPrepPayload(payload: InterviewPrepRequest) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    job_fit_explanation: buildJobFitExplanationSummary(payload.jobFitExplanation),
    application_coach: buildApplicationCoachSummary(payload.applicationCoach),
    cover_letter_draft: buildCoverLetterDraftSummary(payload.coverLetterDraft),
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
    raw_profile_text: payload.rawProfileText ?? null,
  };
}

export function mapInterviewPrepResponse(
  response: MlInterviewPrepResponse,
): InterviewPrep {
  return {
    prepSummary: response.prep_summary,
    likelyTopics: response.likely_topics,
    technicalFocus: response.technical_focus,
    behavioralFocus: response.behavioral_focus,
    storiesToPrepare: response.stories_to_prepare,
    questionsToAsk: response.questions_to_ask,
    riskAreas: response.risk_areas,
    followUpPlan: response.follow_up_plan,
  };
}

export async function getInterviewPrep(
  payload: InterviewPrepRequest,
): Promise<InterviewPrep> {
  void logUserEvent(payload.profileId, {
    eventType: 'interview_prep_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      has_fit_explanation: Boolean(payload.jobFitExplanation),
      has_application_coach: Boolean(payload.applicationCoach),
      has_cover_letter_draft: Boolean(payload.coverLetterDraft),
      has_raw_profile_text: Boolean(payload.rawProfileText),
    },
  }).catch(() => null);

  const response = await mlRequest<MlInterviewPrepResponse>('/v1/enrichment/interview-prep', {
    method: 'POST',
    body: JSON.stringify(buildInterviewPrepPayload(payload)),
  });

  return mapInterviewPrepResponse(response);
}
