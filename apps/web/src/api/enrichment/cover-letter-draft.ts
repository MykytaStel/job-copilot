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
  type CoverLetterDraftRequest,
} from './shared';
import type { CoverLetterDraft, MlCoverLetterDraftResponse } from './types';

export function buildCoverLetterDraftPayload(payload: CoverLetterDraftRequest) {
  return {
    profile_id: payload.profileId,
    analyzed_profile: buildAnalyzedProfilePayload(payload.analyzedProfile),
    search_profile: buildSearchProfilePayload(payload.searchProfile),
    ranked_job: buildRankedJobPayload(payload.rankedJob),
    deterministic_fit: buildDeterministicFitPayload(payload.deterministicFit),
    job_fit_explanation: buildJobFitExplanationSummary(payload.jobFitExplanation),
    application_coach: buildApplicationCoachSummary(payload.applicationCoach),
    feedback_state: buildFeedbackStatePayload(payload.feedbackState),
    raw_profile_text: payload.rawProfileText ?? null,
  };
}

export function mapCoverLetterDraftResponse(
  response: MlCoverLetterDraftResponse,
): CoverLetterDraft {
  return {
    draftSummary: response.draft_summary,
    openingParagraph: response.opening_paragraph,
    bodyParagraphs: response.body_paragraphs,
    closingParagraph: response.closing_paragraph,
    keyClaimsUsed: response.key_claims_used,
    evidenceGaps: response.evidence_gaps,
    toneNotes: response.tone_notes,
  };
}

export async function getCoverLetterDraft(
  payload: CoverLetterDraftRequest,
): Promise<CoverLetterDraft> {
  void logUserEvent(payload.profileId, {
    eventType: 'cover_letter_draft_requested',
    jobId: payload.rankedJob.id,
    payloadJson: {
      surface: 'profile_page',
      deterministic_fit_score: payload.deterministicFit.score,
      has_fit_explanation: Boolean(payload.jobFitExplanation),
      has_application_coach: Boolean(payload.applicationCoach),
      has_raw_profile_text: Boolean(payload.rawProfileText),
    },
  }).catch(() => null);

  const response = await mlRequest<MlCoverLetterDraftResponse>(
    '/v1/enrichment/cover-letter-draft',
    {
      method: 'POST',
      body: JSON.stringify(buildCoverLetterDraftPayload(payload)),
    },
  );

  return mapCoverLetterDraftResponse(response);
}
